//! The parser: tokens → AST.
//!
//! Recursive descent + precedence climbing for expressions. Sets provenance at
//! construction (surface authors in the candid register; the OFFICIAL face is derived
//! by `E`; there is no `E⁻¹`, invariant I2).
//!
//! No wildcard arms in statement dispatch: it enumerates every keyword it accepts and
//! errors on anything else.

use crate::ast::{BinOp, Expr, Program, Stmt, UnOp};
use crate::euphemism;
use crate::lexer::{lex, Tok, Token};

/// A parse failure (loud, never swallowed — §12).
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: u32,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error (line {}): {}", self.line, self.message)
    }
}

impl From<crate::lexer::LexError> for ParseError {
    fn from(e: crate::lexer::LexError) -> Self {
        ParseError {
            message: e.message,
            line: e.line,
        }
    }
}

/// Parse source into a `Program`.
pub fn parse(src: &str) -> Result<Program, ParseError> {
    let toks = lex(src)?;
    Parser { toks, i: 0 }.program()
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.i].tok
    }
    fn line(&self) -> u32 {
        self.toks[self.i].line
    }
    fn la(&self, n: usize) -> &Tok {
        let j = (self.i + n).min(self.toks.len() - 1);
        &self.toks[j].tok
    }
    fn next(&mut self) -> Tok {
        let t = self.toks[self.i].tok.clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        t
    }
    fn err<T>(&self, message: impl Into<String>) -> Result<T, ParseError> {
        Err(ParseError {
            message: message.into(),
            line: self.line(),
        })
    }
    fn eat(&mut self, want: &Tok) -> Result<Tok, ParseError> {
        if self.peek() == want {
            Ok(self.next())
        } else {
            self.err(format!("expected {want:?}, got {:?}", self.peek()))
        }
    }
    fn eat_ident(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.next();
                Ok(s)
            }
            other => self.err(format!("expected identifier, got {other:?}")),
        }
    }
    fn eat_string(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.next();
                Ok(s)
            }
            other => self.err(format!("expected string, got {other:?}")),
        }
    }

    fn program(&mut self) -> Result<Program, ParseError> {
        self.eat(&Tok::Operation)?;
        self.eat(&Tok::LParen)?;
        let op_name = self.eat_string()?;
        self.eat(&Tok::RParen)?;
        let mut body = Vec::new();
        while *self.peek() != Tok::Eof {
            body.push(self.stmt()?);
        }
        Ok(Program { op_name, body })
    }

    /// A `{ … }` block.
    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.eat(&Tok::LBrace)?;
        let mut body = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unterminated block");
            }
            body.push(self.stmt()?);
        }
        self.eat(&Tok::RBrace)?;
        Ok(body)
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Tok::Ident(kw) => match kw.as_str() {
                "if" => self.if_stmt(),
                "while" => self.while_stmt(),
                "func" => self.func_def(),
                "return" => self.return_stmt(),
                "declare" | "assert" => self.declare(),
                "hasbara" => self.hasbara(),
                _ => match self.la(1) {
                    Tok::Eq => self.assign(),
                    Tok::LParen => self.call_or_action(),
                    other => self.err(format!(
                        "unexpected token after identifier `{kw}`: {other:?}"
                    )),
                },
            },
            other => self.err(format!("unexpected token at statement start: {other:?}")),
        }
    }

    fn assign(&mut self) -> Result<Stmt, ParseError> {
        let var = self.eat_ident()?;
        self.eat(&Tok::Eq)?;
        let value = self.expr()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Assign { var, value })
    }

    /// `name ( args ) ;` — a sanctioned/plain **action** (single target), else a call.
    fn call_or_action(&mut self) -> Result<Stmt, ParseError> {
        let name = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let args = self.arg_list()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;

        if euphemism::is_action_verb(&name) {
            // Actions take a single entity target (spike shape `verb(target)`).
            match args.as_slice() {
                [Expr::Var(target)] => Ok(Stmt::Action {
                    verb: name,
                    target: target.clone(),
                    self_defense: false,
                }),
                _ => self.err(format!("operation `{name}` expects a single target entity")),
            }
        } else {
            Ok(Stmt::ExprStmt(Expr::Call { name, args }))
        }
    }

    fn arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if *self.peek() == Tok::RParen {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if *self.peek() == Tok::Comma {
                self.next();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'if'
        self.eat(&Tok::LParen)?;
        let cond = self.expr()?;
        self.eat(&Tok::RParen)?;
        let then_body = self.block()?;
        let else_body = if matches!(self.peek(), Tok::Ident(k) if k == "else") {
            self.next();
            self.block()?
        } else {
            Vec::new()
        };
        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'while'
        self.eat(&Tok::LParen)?;
        let cond = self.expr()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::While { cond, body })
    }

    fn func_def(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'func'
        let name = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let mut params = Vec::new();
        if *self.peek() != Tok::RParen {
            loop {
                params.push(self.eat_ident()?);
                if *self.peek() == Tok::Comma {
                    self.next();
                } else {
                    break;
                }
            }
        }
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::FuncDef { name, params, body })
    }

    fn return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'return'
        if *self.peek() == Tok::Semi {
            self.next();
            return Ok(Stmt::Return(None));
        }
        let e = self.expr()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Return(Some(e)))
    }

    fn declare(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'declare' / 'assert'
        self.eat(&Tok::LParen)?;
        let e = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Declare(e))
    }

    fn hasbara(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'hasbara'
        self.eat(&Tok::LParen)?;
        let talking_point = self.eat_string()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::Hasbara {
            talking_point,
            body,
        })
    }

    // ─────────── expressions (precedence climbing) ───────────

    fn expr(&mut self) -> Result<Expr, ParseError> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.and_expr()?;
        while *self.peek() == Tok::PipePipe {
            self.next();
            let rhs = self.and_expr()?;
            lhs = bin(BinOp::Or, lhs, rhs);
        }
        Ok(lhs)
    }

    fn and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.eq_expr()?;
        while *self.peek() == Tok::AmpAmp {
            self.next();
            let rhs = self.eq_expr()?;
            lhs = bin(BinOp::And, lhs, rhs);
        }
        Ok(lhs)
    }

    fn eq_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.cmp_expr()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => BinOp::Eq,
                Tok::Ne => BinOp::Ne,
                _ => break,
            };
            self.next();
            let rhs = self.cmp_expr()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn cmp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.add_expr()?;
        loop {
            let op = match self.peek() {
                Tok::Lt => BinOp::Lt,
                Tok::Le => BinOp::Le,
                Tok::Gt => BinOp::Gt,
                Tok::Ge => BinOp::Ge,
                _ => break,
            };
            self.next();
            let rhs = self.add_expr()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.mul_expr()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.next();
            let rhs = self.mul_expr()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Percent => BinOp::Mod,
                _ => break,
            };
            self.next();
            let rhs = self.unary()?;
            lhs = bin(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Tok::Minus => {
                self.next();
                Ok(Expr::UnOp {
                    op: UnOp::Neg,
                    expr: Box::new(self.unary()?),
                })
            }
            Tok::Bang => {
                self.next();
                Ok(Expr::UnOp {
                    op: UnOp::Not,
                    expr: Box::new(self.unary()?),
                })
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                Ok(Expr::Int(v))
            }
            Tok::Str(s) => {
                self.next();
                Ok(Expr::Str(s))
            }
            Tok::LParen => {
                self.next();
                let e = self.expr()?;
                self.eat(&Tok::RParen)?;
                Ok(e)
            }
            Tok::Ident(name) => {
                self.next();
                match name.as_str() {
                    "true" => Ok(Expr::Bool(true)),
                    "false" => Ok(Expr::Bool(false)),
                    _ => {
                        if *self.peek() == Tok::LParen {
                            self.next();
                            let args = self.arg_list()?;
                            self.eat(&Tok::RParen)?;
                            Ok(Expr::Call { name, args })
                        } else {
                            Ok(Expr::Var(name))
                        }
                    }
                }
            }
            other => self.err(format!("unexpected token in expression: {other:?}")),
        }
    }
}

fn bin(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::BinOp {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

#[cfg(test)]
mod tests;
