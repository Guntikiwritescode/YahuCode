//! Recursive descent with precedence climbing.

use crate::ast::{BinOp, Expr, InertKind, LawToggle, Program, Stance, Stmt, UnOp};
use crate::euphemism;
use crate::lexer::{lex, Tok, Token};
use crate::model::{Audience, Clearance, Jurisdiction, SODI};

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
        depth: 0,
    }
    .program()?;
    program.comments = comments;
    Ok(program)
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    depth: usize,
}

const MAX_DEPTH: usize = 128;

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
            self.err(format!("expected {want}, found {}", self.peek()))
        }
    }
    fn eat_ident(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Tok::Ident(s) => {
                self.next();
                Ok(s)
            }
            other => self.err(format!("expected an identifier, found {other}")),
        }
    }
    fn eat_string(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.next();
                Ok(s)
            }
            other => self.err(format!("expected a string, found {other}")),
        }
    }

    fn eat_int(&mut self) -> Result<i64, ParseError> {
        match self.peek().clone() {
            Tok::Int(v) => {
                self.next();
                Ok(v)
            }
            other => self.err(format!("expected an integer, found {other}")),
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

    fn nested<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        if self.depth >= MAX_DEPTH {
            return self.err(format!("nesting deeper than {MAX_DEPTH} levels"));
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        self.nested(Self::stmt_inner)
    }

    fn stmt_inner(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
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
                "via" if *self.la(1) == Tok::LParen => {
                    let e = self.expr()?;
                    self.eat(&Tok::Semi)?;
                    Ok(Stmt::ExprStmt(e))
                }
                "address" => self.address(),
                "statement" => self.poly_statement(),
                "commit" => self.position(Stance::Commit),
                "foreclose" => self.position(Stance::Foreclose),
                "legislate" => self.legislate(),
                "allocate" => self.allocate_slot(),
                "push" => self.push_or_remove(true),
                "remove" => self.push_or_remove(false),
                "classify" => self.classify(),
                "revoke" => self.revoke(),
                // A bare accessor such as `route(r, c);` parses as an expression statement.
                "index" | "length" | "route" | "balanced" | "equal_before_the_law"
                    if *self.la(1) == Tok::LParen =>
                {
                    let e = self.expr()?;
                    self.eat(&Tok::Semi)?;
                    Ok(Stmt::ExprStmt(e))
                }
                _ => match self.la(1) {
                    Tok::Eq => self.assign(),
                    Tok::LParen => self.call_or_action(),
                    // A bare `name;` invokes a poly-statement.
                    Tok::Semi => self.invoke(),
                    other => self.err(format!("unexpected {other} after identifier `{kw}`")),
                },
            },
            other => self.err(format!("a statement cannot start with {other}")),
        }
    }

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

    fn call_or_action(&mut self) -> Result<Stmt, ParseError> {
        let name = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let args = self.arg_list()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;

        if euphemism::is_action_verb(&name) {
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
        self.next();
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
        self.next();
        self.eat(&Tok::LParen)?;
        let cond = self.expr()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::While { cond, body })
    }

    fn func_def(&mut self) -> Result<Stmt, ParseError> {
        self.next();
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
        self.next();
        if *self.peek() == Tok::Semi {
            self.next();
            return Ok(Stmt::Return(None));
        }
        let e = self.expr()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Return(Some(e)))
    }

    fn declare(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let e = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Declare(e))
    }

    fn kw_one_ident(&mut self, _kw: &str) -> Result<String, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let arg = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(arg)
    }

    fn let_binding(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        let name = self.eat_ident()?;
        self.eat(&Tok::Eq)?;

        if let Tok::Ident(kw) = self.peek().clone() {
            match kw.as_str() {
                "apportionment" => {
                    self.next();
                    self.eat(&Tok::LBracket)?;
                    let size = self.eat_int()?;
                    self.eat(&Tok::RBracket)?;
                    self.eat(&Tok::Semi)?;
                    return Ok(Stmt::Assign {
                        var: name.clone(),
                        value: Expr::Apportionment { label: name, size },
                    });
                }
                "facts_on_the_ground" => {
                    self.next();
                    self.eat(&Tok::LParen)?;
                    self.eat(&Tok::RParen)?;
                    self.eat(&Tok::Semi)?;
                    return Ok(Stmt::Assign {
                        var: name.clone(),
                        value: Expr::FactsNew { label: name },
                    });
                }
                "registry" => {
                    self.next();
                    self.eat(&Tok::LParen)?;
                    let rule = self.eat_string()?;
                    self.eat(&Tok::RParen)?;
                    self.eat(&Tok::Semi)?;
                    return Ok(Stmt::Assign {
                        var: name.clone(),
                        value: Expr::RegistryNew { label: name, rule },
                    });
                }
                _ => {}
            }
        }

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
                "`let` binding must be allocate/settlement/establish_commission or a \
                 collection constructor (apportionment/facts_on_the_ground/registry), got `{other}`"
            )),
        }
    }

    fn ceasefire(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Ceasefire)
    }

    fn deeply_concerned(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Concern { who: None })
    }

    fn kw_no_arg(&mut self, _kw: &str) -> Result<(), ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(())
    }

    fn disputed(&mut self) -> Result<Stmt, ParseError> {
        self.next();
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

    fn human_shields(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let verb = self.eat_ident()?;
        self.eat(&Tok::LParen)?;
        let target = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::HumanShields { verb, target })
    }

    fn allocate_slot(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let coll = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let idx = self.expr()?;
        self.eat(&Tok::Comma)?;
        let value = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::AllocateSlot { coll, idx, value })
    }

    fn push_or_remove(&mut self, is_push: bool) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let coll = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let item = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        if is_push {
            Ok(Stmt::Push { coll, item })
        } else {
            Ok(Stmt::Remove { coll, item })
        }
    }

    fn jurisdiction(&mut self) -> Result<Jurisdiction, ParseError> {
        let word = self.eat_ident()?;
        match word.as_str() {
            "military" => Ok(Jurisdiction::Military),
            "civilian" => Ok(Jurisdiction::Civilian),
            other => self.err(format!(
                "unknown jurisdiction `{other}` (expected `military` or `civilian`)"
            )),
        }
    }

    fn classify(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let coll = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let case = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let jurisdiction = self.jurisdiction()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Classify {
            coll,
            case,
            jurisdiction,
        })
    }

    fn revoke(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let coll = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let case = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Revoke { coll, case })
    }

    fn bribe(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let name = self.eat_ident()?;
        self.eat(&Tok::Comma)?;
        let amount = self.expr()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Bribe { name, amount })
    }

    fn postpone(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Postpone)
    }

    fn elections(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Elections)
    }

    fn announce(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        let text = self.eat_string()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Announce { text })
    }

    fn position(&mut self, stance: Stance) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let subject = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Position { stance, subject })
    }

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

    fn address(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let audience = self.audience()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::Address { audience, body })
    }

    fn poly_statement(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        let name = self.eat_ident()?;
        self.eat(&Tok::LBrace)?;
        let mut arms = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof {
                return self.err("unterminated poly-statement");
            }
            match self.peek().clone() {
                Tok::Ident(k) if k == "to" => {
                    self.next();
                }
                other => return self.err(format!("expected `to <audience>`, found {other}")),
            }
            let audience = self.audience()?;
            let body = self.block()?;
            arms.push((audience, body));
        }
        self.eat(&Tok::RBrace)?;
        Ok(Stmt::PolyStatement { name, arms })
    }

    fn invoke(&mut self) -> Result<Stmt, ParseError> {
        let name = self.eat_ident()?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Invoke { name })
    }

    fn legislate(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let head = self.eat_ident()?;
        let toggle = match head.as_str() {
            "retroactively_sanction" => {
                self.eat(&Tok::Colon)?;
                let verb = self.eat_ident()?;
                LawToggle::RetroactivelySanction(verb)
            }
            "expunge_last_discrepancy" => LawToggle::ExpungeLastDiscrepancy,
            "waive_gate" => LawToggle::WaiveGate,
            other => {
                return self.err(format!(
                    "unknown law toggle `{other}` (expected retroactively_sanction: <verb>, \
                     expunge_last_discrepancy, or waive_gate)"
                ))
            }
        };
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Legislate { toggle })
    }

    fn hasbara(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let talking_point = self.eat_string()?;
        self.eat(&Tok::RParen)?;
        let body = self.block()?;
        Ok(Stmt::Hasbara {
            talking_point,
            body,
        })
    }

    fn mossad(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        let body = self.block()?;
        Ok(Stmt::Mossad { body })
    }

    fn blame(&mut self) -> Result<Stmt, ParseError> {
        self.next();
        self.eat(&Tok::LParen)?;
        let who = self.eat_ident()?;
        self.eat(&Tok::RParen)?;
        self.eat(&Tok::Semi)?;
        Ok(Stmt::Blame { who })
    }

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
        self.nested(Self::unary_inner)
    }

    fn unary_inner(&mut self) -> Result<Expr, ParseError> {
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
                if let Tok::Ident(kw) = self.la(1).clone() {
                    if is_cast_keyword(&kw) && *self.la(2) == Tok::RParen {
                        self.next();
                        self.next();
                        self.next();
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
                    "read" if *self.peek() == Tok::LParen => {
                        self.next();
                        let e = self.expr()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Read(Box::new(e)))
                    }
                    "external" if *self.peek() == Tok::LParen => {
                        self.next();
                        let args = self.arg_list()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::External(args))
                    }
                    "via" if *self.peek() == Tok::LParen => {
                        self.next();
                        let proxy = self.eat_ident()?;
                        self.eat(&Tok::Comma)?;
                        let inner = self.expr()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Via {
                            proxy,
                            inner: Box::new(inner),
                        })
                    }
                    // Collection ops are tried before the generic call table.
                    "index" if *self.peek() == Tok::LParen => {
                        self.next();
                        let coll = self.eat_ident()?;
                        self.eat(&Tok::Comma)?;
                        let idx = self.expr()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Index {
                            coll,
                            idx: Box::new(idx),
                        })
                    }
                    "balanced" if *self.peek() == Tok::LParen => {
                        self.next();
                        let coll = self.eat_ident()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Balanced { coll })
                    }
                    "length" if *self.peek() == Tok::LParen => {
                        self.next();
                        let coll = self.eat_ident()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Length { coll })
                    }
                    "route" if *self.peek() == Tok::LParen => {
                        self.next();
                        let coll = self.eat_ident()?;
                        self.eat(&Tok::Comma)?;
                        let case = self.eat_ident()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::Route { coll, case })
                    }
                    "equal_before_the_law" if *self.peek() == Tok::LParen => {
                        self.next();
                        let coll = self.eat_ident()?;
                        self.eat(&Tok::RParen)?;
                        Ok(Expr::EqualBeforeLaw { coll })
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
            other => self.err(format!("unexpected {other} in an expression")),
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
        _ => Expr::Cast {
            target: Clearance::Sodi,
            expr: Box::new(operand),
        },
    }
}

#[cfg(test)]
mod tests;
