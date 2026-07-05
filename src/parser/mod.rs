//! The parser: tokens → AST.
//!
//! Recursive descent over the token stream. It sets provenance at construction (the
//! statements it builds are `AUTHORED_ACTUAL` — the surface authors in the candid
//! register and the OFFICIAL face is derived by `E`; there is no `E⁻¹`, invariant I2).
//!
//! Phase-0 subset (grows per phase with the grammar). No wildcard arms: statement
//! dispatch enumerates every keyword it accepts and errors on anything else.

use crate::ast::{DeclRhs, Program, Stmt};
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
    /// Look ahead `n` tokens (saturating at EOF).
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

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Tok::Ident(kw) => match kw.as_str() {
                "declare" | "assert" => self.declare(),
                "hasbara" => self.hasbara(),
                _ => {
                    // Not a keyword: either an assignment `x = …` or an action `x(t)`.
                    match self.la(1) {
                        Tok::Eq => self.assign(),
                        Tok::LParen => self.action(),
                        other => self.err(format!(
                            "unexpected token after identifier `{kw}`: {other:?}"
                        )),
                    }
                }
            },
            other => self.err(format!("unexpected token at statement start: {other:?}")),
        }
    }

    fn assign(&mut self) -> Result<Stmt, ParseError> {
        let var = self.eat_ident()?;
        self.eat(&Tok::Eq)?;
        let value = self.eat_int()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Assign { var, value })
    }

    fn eat_int(&mut self) -> Result<i64, ParseError> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                Ok(v)
            }
            other => self.err(format!("expected integer, got {other:?}")),
        }
    }

    fn action(&mut self) -> Result<Stmt, ParseError> {
        let verb = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let target = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Action {
            verb,
            target,
            self_defense: false,
        })
    }

    fn declare(&mut self) -> Result<Stmt, ParseError> {
        // `declare` / `assert` (alias) `( ident == (int|ident) ) ;`
        self.next(); // the keyword
        self.eat(&Tok::LParen)?;
        let lhs = self.eat_ident()?;
        self.eat(&Tok::EqEq)?;
        let rhs = match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                DeclRhs::Int(v)
            }
            Tok::Ident(s) => {
                self.next();
                DeclRhs::Var(s)
            }
            other => return self.err(format!("expected int or identifier, got {other:?}")),
        };
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Declare { lhs, rhs })
    }

    fn hasbara(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'hasbara'
        self.eat(&Tok::LParen)?;
        let talking_point = self.eat_string()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::LBrace)?;
        let mut body = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unterminated hasbara block");
            }
            body.push(self.stmt()?);
        }
        self.eat(&Tok::RBrace)?;
        Ok(Stmt::Hasbara {
            talking_point,
            body,
        })
    }
}

#[cfg(test)]
mod tests;
