//! The parser: tokens → AST.
//!
//! Recursive descent + precedence climbing for expressions. Sets provenance at
//! construction (surface authors in the candid register; the OFFICIAL face is derived
//! by `E`; there is no `E⁻¹`, invariant I2).
//!
//! No wildcard arms in statement dispatch: it enumerates every keyword it accepts and
//! errors on anything else.

use crate::ast::{BinOp, Expr, InertKind, Program, Stance, Stmt, UnOp};
use crate::euphemism;
use crate::lexer::{lex, Tok, Token};
use crate::model::{Audience, Clearance, SODI};

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

/// Parse source into a `Program`. Source comments are extracted from the token stream
/// and carried on the `Program` (they never affect parsing or execution).
pub fn parse(src: &str) -> Result<Program, ParseError> {
    let toks = lex(src)?;
    let mut comments = Vec::new();
    let filtered: Vec<Token> = toks
        .into_iter()
        .filter(|t| {
            if let Tok::Comment(c) = &t.tok {
                comments.push(c.clone());
                false
            } else {
                true
            }
        })
        .collect();
    let mut program = Parser {
        toks: filtered,
        i: 0,
    }
    .program()?;
    program.comments = comments;
    Ok(program)
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

    fn eat_int(&mut self) -> Result<i64, ParseError> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                Ok(v)
            }
            other => self.err(format!("expected integer, got {other:?}")),
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
        Ok(Program {
            op_name,
            body,
            comments: Vec::new(),
        })
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
            // `(self_defense) action;` — the universal cast applied to an action (#7).
            Tok::LParen => self.self_defense_action(),
            Tok::Ident(kw) => match kw.as_str() {
                "if" => self.if_stmt(),
                "while" => self.while_stmt(),
                "func" => self.func_def(),
                "return" => self.return_stmt(),
                "declare" | "assert" => self.declare(),
                "hasbara" => self.hasbara(),
                "mossad" => self.mossad(),
                "blame" => self.blame(),
                "let" => self.let_binding(),
                "bribe" => self.bribe(),
                "postpone" => self.postpone(),
                "elections" => self.elections(),
                "raise" => Ok(Stmt::Raise {
                    name: self.kw_one_ident("raise")?,
                }),
                "whatabout" => Ok(Stmt::Whatabout {
                    name: self.kw_one_ident("whatabout")?,
                }),
                "ceasefire" => self.ceasefire(),
                "deeply_concerned" => self.deeply_concerned(),
                "concern" => Ok(Stmt::Concern {
                    who: Some(self.kw_one_ident("concern")?),
                }),
                "criticism" => Ok(Stmt::Criticism {
                    subject: self.kw_one_ident("criticism")?,
                }),
                "antisemitism" => Ok(Stmt::Antisemitism {
                    incident: self.kw_one_ident("antisemitism")?,
                }),
                "access" => Ok(Stmt::Access {
                    entity: self.kw_one_ident("access")?,
                }),
                "timeline" => Ok(Stmt::Timeline {
                    symbol: self.kw_one_ident("timeline")?,
                }),
                "human_shields" => self.human_shields(),
                "proportionate" => Ok(Stmt::Proportionate {
                    claim: self.kw_one_ident("proportionate")?,
                }),
                "disputed" => self.disputed(),
                "deny" => Ok(Stmt::Deny {
                    event: self.kw_one_ident("deny")?,
                }),
                "world_opinion" => {
                    self.kw_no_arg("world_opinion")?;
                    Ok(Stmt::Inert {
                        kind: InertKind::WorldOpinion,
                    })
                }
                "polls" => {
                    self.kw_no_arg("polls")?;
                    Ok(Stmt::Inert {
                        kind: InertKind::Polls,
                    })
                }
                "investigate" => Ok(Stmt::Investigate {
                    subject: self.kw_one_ident("investigate")?,
                }),
                "address_international" => {
                    self.kw_no_arg("address_international")?;
                    Ok(Stmt::AddressInternational)
                }
                "announce" => self.announce(),
                // Feature B — audience dispatch.
                "address" => self.address(),
                "statement" => self.poly_statement(),
                "commit" => self.position(Stance::Commit),
                "foreclose" => self.position(Stance::Foreclose),
                _ => match self.la(1) {
                    Tok::Eq => self.assign(),
                    Tok::LParen => self.call_or_action(),
                    // `name;` — invoke a poly-statement under the current audience (B).
                    // A bare `ident;` is otherwise not a valid statement, so this is
                    // unambiguous (mirrors action-verb vs call disambiguation).
                    Tok::Semi => self.invoke(),
                    other => self.err(format!(
                        "unexpected token after identifier `{kw}`: {other:?}"
                    )),
                },
            },
            other => self.err(format!("unexpected token at statement start: {other:?}")),
        }
    }

    /// `(self_defense) <action>;` — the one statement-level cast prefix (spike shape).
    fn self_defense_action(&mut self) -> Result<Stmt, ParseError> {
        self.eat(&Tok::LParen)?;
        let name = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        if name != "self_defense" {
            return self.err("only (self_defense) is a valid cast prefix on a statement");
        }
        match self.call_or_action()? {
            Stmt::Action { verb, target, .. } => Ok(Stmt::Action {
                verb,
                target,
                self_defense: true,
            }),
            _ => self.err("(self_defense) must prefix a sanctioned action"),
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

    /// A keyword taking a single identifier argument: `kw ( ident ) ;`.
    fn kw_one_ident(&mut self, _kw: &str) -> Result<String, ParseError> {
        self.next(); // the keyword
        self.eat(&Tok::LParen)?;
        let arg = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(arg)
    }

    /// `let name = (allocate|establish_commission|settlement)(what);`
    fn let_binding(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'let'
        let name = self.eat_ident()?;
        self.eat(&Tok::Eq)?;
        let kind = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let what = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        match kind.as_str() {
            "allocate" => Ok(Stmt::Allocate { name, what }),
            "settlement" => Ok(Stmt::Settlement { name, what }),
            "establish_commission" => Ok(Stmt::EstablishCommission {
                name,
                subject: what,
            }),
            other => self.err(format!(
                "`let` binding must be allocate/settlement/establish_commission, got `{other}`"
            )),
        }
    }

    fn ceasefire(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'ceasefire'
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Ceasefire)
    }

    /// `deeply_concerned();` — an ally's no-op with no argument.
    fn deeply_concerned(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'deeply_concerned'
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Concern { who: None })
    }

    /// A keyword taking no arguments: `kw ( ) ;`. The keyword is the current token.
    fn kw_no_arg(&mut self, _kw: &str) -> Result<(), ParseError> {
        self.next(); // the keyword
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(())
    }

    /// `disputed(name, official, actual);` — a contested figure.
    fn disputed(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'disputed'
        self.eat(&Tok::LParen)?;
        let name = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let official = self.eat_int()?;
        self.eat(&Tok::Comma)?;
        let actual = self.eat_int()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Disputed {
            name,
            official,
            actual,
        })
    }

    /// `human_shields(verb(target));` — legalize a wrapped action.
    fn human_shields(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'human_shields'
        self.eat(&Tok::LParen)?;
        let verb = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let target = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::HumanShields { verb, target })
    }

    /// `bribe(name, amount);`
    fn bribe(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'bribe'
        self.eat(&Tok::LParen)?;
        let name = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let amount = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Bribe { name, amount })
    }

    /// `postpone();`
    fn postpone(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'postpone'
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Postpone)
    }

    /// `elections;`
    fn elections(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'elections'
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Elections)
    }

    /// `announce "…";` — the official authoring register (Feature A). The `OFFICIAL`
    /// face is the announced string verbatim; the `ACTUAL` face is `UNAVAILABLE`.
    fn announce(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'announce'
        let text = self.eat_string()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Announce { text })
    }

    /// `commit(subject);` / `foreclose(subject);` — a policy position (Feature B).
    fn position(&mut self, stance: Stance) -> Result<Stmt, ParseError> {
        self.next(); // the stance keyword
        self.eat(&Tok::LParen)?;
        let subject = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Position { stance, subject })
    }

    /// The audience keyword inside `address(...)` / `to ...` — a closed set; `Record` is
    /// the implicit default and is never written in source.
    fn audience(&mut self) -> Result<Audience, ParseError> {
        let word = self.eat_ident()?;
        match word.as_str() {
            "domestic" => Ok(Audience::Domestic),
            "international" => Ok(Audience::International),
            other => self.err(format!(
                "unknown audience `{other}` (expected `domestic` or `international`)"
            )),
        }
    }

    /// `address(audience) { … }` — run a block addressing a specific room (Feature B).
    fn address(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'address'
        self.eat(&Tok::LParen)?;
        let audience = self.audience()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::Address { audience, body })
    }

    /// `statement name { to X { … } to Y { … } }` — a poly-statement (Feature B).
    fn poly_statement(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'statement'
        let name = self.eat_ident()?;
        self.eat(&Tok::LBrace)?;
        let mut arms = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unterminated poly-statement");
            }
            // `to <audience> { … }`
            match self.peek().clone() {
                Tok::Ident(k) if k == "to" => {
                    self.next(); // 'to'
                }
                other => return self.err(format!("expected `to <audience>`, got {other:?}")),
            }
            let audience = self.audience()?;
            let body = self.block()?;
            arms.push((audience, body));
        }
        self.eat(&Tok::RBrace)?;
        Ok(Stmt::PolyStatement { name, arms })
    }

    /// `name;` — invoke a poly-statement under the current audience (Feature B).
    fn invoke(&mut self) -> Result<Stmt, ParseError> {
        let name = self.eat_ident()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Invoke { name })
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

    /// `mossad { … }` — the covert scope.
    fn mossad(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'mossad'
        let body = self.block()?;
        Ok(Stmt::Mossad { body })
    }

    /// `blame(who);`
    fn blame(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // 'blame'
        self.eat(&Tok::LParen)?;
        let who = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Blame { who })
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
                // A cast `(PUBLIC|RESTRICTED|סודי|self_defense) <operand>`, or a plain
                // parenthesized expression.
                if let Tok::Ident(kw) = self.la(1).clone() {
                    if is_cast_keyword(&kw) && *self.la(2) == Tok::RParen {
                        self.next(); // (
                        self.next(); // keyword
                        self.next(); // )
                        let operand = self.unary()?;
                        return Ok(build_cast(&kw, operand));
                    }
                }
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
                    // `read(e)` — the clearance-gated read (the ONE read path).
                    "read" if *self.peek() == Tok::LParen => {
                        self.next();
                        let e = self.expr()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Read(Box::new(e)))
                    }
                    // `external(args…)` — the mossad foreign interface.
                    "external" if *self.peek() == Tok::LParen => {
                        self.next();
                        let args = self.arg_list()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::External(args))
                    }
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

/// The four cast-prefix keywords: the three clearance levels plus `self_defense`.
fn is_cast_keyword(kw: &str) -> bool {
    kw == "PUBLIC" || kw == "RESTRICTED" || kw == SODI || kw == "self_defense"
}

fn build_cast(kw: &str, operand: Expr) -> Expr {
    match kw {
        "PUBLIC" => Expr::Cast {
            target: Clearance::Public,
            expr: Box::new(operand),
        },
        "RESTRICTED" => Expr::Cast {
            target: Clearance::Restricted,
            expr: Box::new(operand),
        },
        "self_defense" => Expr::SelfDefense(Box::new(operand)),
        // The remaining cast keyword is סודי (guarded by `is_cast_keyword`).
        _ => Expr::Cast {
            target: Clearance::Sodi,
            expr: Box::new(operand),
        },
    }
}

#[cfg(test)]
mod tests;
