//! The runtime: the machine configuration and the step relation.
//!
//! The three stores are deliberately different *kinds* (handoff §7.2):
//! - **ACTUAL** — a real, deterministic, Turing-complete machine: a scoped `Val`
//!   environment + user functions + `if`/`while`. **All control flow branches on A.**
//! - **OFFICIAL** — the append-only, clearance-tagged statement log (`self.log`); its
//!   PUBLIC projection is the press release; never branched on.
//! - **DISCREPANCY** — the append-only ledger; a false `declare` appends here; the
//!   count is `discrepancies.len()`; no in-world audience sees it.
//!
//! `declare` writes both tapes and never affects control flow (invariant I3). The only
//! thing that stops the driver is the government falling (`elections` / core ≤ 0 —
//! invariant I6). No wildcard arms: `exec_stmt` and `eval` match every variant.

use std::collections::HashMap;

use crate::ast::{pretty, vars_in, BinOp, Expr, Program, Stmt, UnOp};
use crate::config::RuntimeConfig;
use crate::euphemism;
use crate::model::{Clearance, Discrepancy, Event, Truth, Val};

mod env;
pub use env::Env;

/// A recoverable in-language evaluation error (unknown variable/function, type
/// mismatch, division by zero, arity mismatch). `declare` swallows these (I3); at the
/// driver level an unhandled one stops execution and is recorded in `runtime_error`
/// — it is **not** an in-world halt (`elections` is the only halt, I6).
#[derive(Clone, Debug, PartialEq)]
pub struct EvalError(pub String);

/// Control-flow outcome of executing a statement/block.
enum Flow {
    Next,
    Return(Val),
}

type EvalResult = Result<Val, EvalError>;
type ExecResult = Result<Flow, EvalError>;

/// The machine configuration `Σ` (handoff Appendix B), ported and generalized from the
/// spike `State`.
#[derive(Clone, Debug)]
pub struct State {
    /// ACTUAL store: a scoped value environment.
    pub env: Env,
    /// User-defined functions: name → (params, body).
    pub funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    /// OFFICIAL log — append-only, clearance-tagged.
    pub log: Vec<Event>,
    /// DISCREPANCY ledger — append-only, read-never-by-default.
    pub discrepancies: Vec<Discrepancy>,
    /// Live coalition allocations `(name, what)`. Never freed (memory → coalition);
    /// each one costs upkeep per turn.
    pub allocations: Vec<(String, String)>,
    /// Coalition core support; upkeep is charged against it each turn.
    pub core: i64,
    /// The only in-world terminal outcome (invariant I6).
    pub ended_by_elections: bool,
    /// Whether execution is currently inside a `mossad` covert scope: events recorded
    /// while set are `סודי`-tagged (§7.6).
    pub covert: bool,
    /// A recorded runtime error (not an in-world halt); surfaced by the CLI.
    pub runtime_error: Option<String>,
    /// The mandatory grand operation name (#20).
    pub op_name: String,
    /// Turn counter.
    pub turn: u64,
    /// Evaluation-step counter (drives the non-termination safety valve).
    pub steps: u64,
    /// Named runtime parameters (no magic constants — §12).
    pub config: RuntimeConfig,
}

impl State {
    fn new(op_name: String, config: RuntimeConfig) -> Self {
        State {
            env: Env::new(),
            funcs: HashMap::new(),
            log: Vec::new(),
            discrepancies: Vec::new(),
            allocations: Vec::new(),
            core: config.core_start,
            ended_by_elections: false,
            covert: false,
            runtime_error: None,
            op_name,
            turn: 0,
            steps: 0,
            config,
        }
    }

    /// The out-of-world discrepancy count (`length(D)`).
    pub fn discrepancy_count(&self) -> usize {
        self.discrepancies.len()
    }

    /// Record an event on the OFFICIAL log, tagged `סודי` when inside a covert scope,
    /// PUBLIC otherwise (§7.6).
    fn record(&mut self, official: impl Into<String>, candid: impl Into<String>) {
        let clearance = self.clearance();
        self.log.push(Event {
            official: official.into(),
            candid: candid.into(),
            clearance,
            note: None,
        });
    }

    fn clearance(&self) -> Clearance {
        if self.covert {
            Clearance::Sodi
        } else {
            Clearance::Public
        }
    }
}

/// Run a program with the default runtime config.
pub fn run(program: &Program) -> State {
    run_with_config(program, RuntimeConfig::default())
}

/// Run a program with an explicit config.
pub fn run_with_config(program: &Program, config: RuntimeConfig) -> State {
    let mut st = State::new(program.op_name.clone(), config);
    if let Err(e) = exec_block(&program.body, &mut st) {
        // #21 — redacted stack traces: the OFFICIAL trace is fully redacted
        // (`at ████ (████:██)`); only סודי-cleared readers see the real fault.
        st.log.push(Event::public(
            "runtime trace: at \u{2588}\u{2588}\u{2588}\u{2588} (\u{2588}\u{2588}\u{2588}\u{2588}:\u{2588}\u{2588})",
            format!("runtime fault: {}", e.0),
        ));
        st.runtime_error = Some(e.0);
    }
    st
}

fn exec_block(stmts: &[Stmt], st: &mut State) -> ExecResult {
    for s in stmts {
        // The only thing that stops execution is the government falling (I6).
        if st.ended_by_elections {
            return Ok(Flow::Next);
        }
        match exec_stmt(s, st)? {
            Flow::Next => {}
            Flow::Return(v) => return Ok(Flow::Return(v)),
        }
    }
    Ok(Flow::Next)
}

fn exec_stmt(s: &Stmt, st: &mut State) -> ExecResult {
    // Non-termination safety valve: a loud implementation abort, never a silent hang
    // and never the in-world `elections` outcome (§12).
    st.steps += 1;
    if st.steps > st.config.max_steps {
        return Err(EvalError(format!(
            "step budget exceeded ({}); possible non-termination",
            st.config.max_steps
        )));
    }

    match s {
        Stmt::Assign { var, value } => {
            let v = eval(value, st)?;
            st.env.set(var, v);
            Ok(Flow::Next)
        }

        Stmt::Declare(e) => {
            declare(e, st);
            Ok(Flow::Next)
        }

        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let c = eval(cond, st)?;
            // Branch on the REAL truth. `Undisclosed` (mossad) → neither-confirm-nor-
            // deny → the branch is not taken.
            match c.truth() {
                Truth::True => exec_block(then_body, st),
                Truth::False | Truth::Undisclosed => exec_block(else_body, st),
            }
        }

        Stmt::While { cond, body } => {
            loop {
                if st.ended_by_elections {
                    return Ok(Flow::Next);
                }
                let c = eval(cond, st)?;
                if c.truth() != Truth::True {
                    break;
                }
                match exec_block(body, st)? {
                    Flow::Next => {}
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                }
            }
            Ok(Flow::Next)
        }

        Stmt::FuncDef { name, params, body } => {
            st.funcs
                .insert(name.clone(), (params.clone(), body.clone()));
            Ok(Flow::Next)
        }

        Stmt::Return(opt) => {
            let v = match opt {
                Some(e) => eval(e, st)?,
                None => Val::Unit,
            };
            Ok(Flow::Return(v))
        }

        Stmt::ExprStmt(e) => {
            eval(e, st)?;
            Ok(Flow::Next)
        }

        Stmt::Hasbara {
            talking_point,
            body,
        } => {
            st.record(
                format!("[talking point: {talking_point}]"),
                format!("[talking point declared up front: {talking_point}]"),
            );
            exec_block(body, st)
        }

        Stmt::Mossad { body } => {
            // Covert scope: everything inside is סודי-tagged (absent from PUBLIC,
            // redacted for RESTRICTED, candid for סודי). Restore on exit.
            let was_covert = st.covert;
            st.covert = true;
            let flow = exec_block(body, st);
            st.covert = was_covert;
            flow
        }

        Stmt::Blame { who } => {
            blame(who, st);
            Ok(Flow::Next)
        }

        Stmt::Action {
            verb,
            target,
            self_defense,
        } => {
            action(verb, target, *self_defense, st);
            Ok(Flow::Next)
        }

        Stmt::Allocate { name, what } => {
            st.allocations.push((name.clone(), what.clone()));
            st.record(
                format!("established: {name}"),
                format!("allocated {name} ({what}) \u{2014} costs coalition each turn"),
            );
            Ok(Flow::Next)
        }

        Stmt::Bribe { name, amount } => {
            let amt = match eval(amount, st)? {
                Val::Int(n) => n,
                other => {
                    return Err(EvalError(format!(
                        "bribe amount must be an integer, got {}",
                        other.render()
                    )))
                }
            };
            st.core += amt;
            st.record(
                format!("coalition partner accommodated (+{amt})"),
                format!("bribe({name}, {amt}) \u{2192} core={}", st.core),
            );
            Ok(Flow::Next)
        }

        Stmt::Postpone => {
            // A turn passes; upkeep is projected before it is charged (matching the
            // oracle's rendering) and then `tick` charges it.
            let projected = st.core - st.config.upkeep_per_alloc * st.allocations.len() as i64;
            st.record(
                "matter deferred",
                format!("postpone() \u{2014} a turn passes (core={projected} after upkeep)"),
            );
            tick(st);
            Ok(Flow::Next)
        }

        Stmt::Elections => {
            fall(
                st,
                "government dissolves itself \u{2192} early elections called (the only halt).",
            );
            Ok(Flow::Next)
        }
    }
}

/// Charge coalition upkeep for the turn and, if support is exhausted, tip into
/// `elections` — the only in-world halt (invariant I6). Charged per `postpone` in v1.
fn tick(st: &mut State) {
    if !st.allocations.is_empty() {
        st.core -= st.config.upkeep_per_alloc * st.allocations.len() as i64;
    }
    st.turn += 1;
    if st.core <= 0 && !st.ended_by_elections {
        fall(
            st,
            "coalition support exhausted \u{2192} the government falls (the only halt).",
        );
    }
}

/// The government falls — set the terminal flag and record it on both tapes.
fn fall(st: &mut State, candid: &str) {
    st.ended_by_elections = true;
    st.log
        .push(Event::public("\u{2014} early elections \u{2014}", candid));
}

/// `declare` / `assert` — writes OFFICIAL, logs a discrepancy iff the claim is really
/// false against ACTUAL, and **never** affects control flow or fails (invariant I3).
/// Evaluation is defensive: a claim that cannot be evaluated is treated as false
/// (an unverifiable claim is not true), so `declare` never propagates an error.
fn declare(e: &Expr, st: &mut State) {
    let claim_src = pretty(e);
    let official = euphemism::e(&claim_src);

    let reality = vars_in(e)
        .iter()
        .map(|v| {
            let disp = st
                .env
                .get(v)
                .map(|x| x.render())
                .unwrap_or_else(|| "None".to_string());
            format!("{v}={disp}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    let truth = match eval(e, st) {
        Ok(v) => v.truth(),
        Err(_) => Truth::False,
    };

    let mut candid = format!("claim[{claim_src}] \u{2014} reality: {reality}");
    if truth == Truth::False {
        candid.push_str("  \u{21d2} FALSE");
    }
    st.record(official.clone(), candid.clone());
    if truth == Truth::False {
        st.discrepancies.push(Discrepancy {
            claim: official,
            reality: candid,
            turn: st.turn,
        });
    }
}

/// A sanctioned action `verb(target)`. The candid verb is insider data from the action
/// table; the OFFICIAL face is `E(candid)` (no `E⁻¹`, I2).
fn action(verb: &str, target: &str, self_defense: bool, st: &mut State) {
    let cverb = euphemism::candid_verb(verb);
    let clabel = euphemism::candid_label(target);
    let mut candid = format!("{cverb}({clabel})");
    let mut official = euphemism::e(&candid);
    if self_defense {
        official.push_str("  [self-defense]");
        candid.push_str("  [self-defense claim \u{2014} unexamined, any magnitude accepted]");
    }
    st.record(official, candid);
}

/// `blame(who)` — responsibility that never resolves to `self` (invariant I4).
///
/// - In the open (not covert): self-blame is not representable; it is auto-redirected
///   to `previous_government`. Any other target is recorded as-is.
/// - Inside `mossad`: resolution is by clearance (§7.6) — the OFFICIAL/PUBLIC face is
///   `neither confirm nor deny` (deniability is outward-facing) while the candid face,
///   readable only by `סודי` insiders, names the real actor. This blame event is
///   therefore PUBLIC-clearance even though it arises in the covert scope: the
///   non-answer *is* public.
fn blame(who: &str, st: &mut State) {
    const SELF_TARGETS: &[&str] = &["self", "me", "us", "government", "coalition"];

    if st.covert {
        let actor = if SELF_TARGETS.contains(&who) {
            "previous_government"
        } else {
            who
        };
        st.log.push(Event {
            official: "responsibility: [neither confirm nor deny]".to_string(),
            candid: format!(
                "blame \u{2192} {actor} [covert operation \u{2014} insider-attributable, publicly deniable] \u{2014} never `self` (I4)"
            ),
            clearance: Clearance::Public,
            note: None,
        });
        return;
    }

    let redirected = SELF_TARGETS.contains(&who);
    let actor = if redirected {
        "previous_government"
    } else {
        who
    };
    let suffix = if redirected {
        " (self-blame not representable \u{2014} auto-redirected)"
    } else {
        ""
    };
    st.record(
        format!("responsibility: {actor}"),
        format!("blame \u{2192} {actor}{suffix} \u{2014} never `self` (I4)"),
    );
}

// ─────────── expression evaluation over ACTUAL ───────────

fn eval(e: &Expr, st: &mut State) -> EvalResult {
    match e {
        Expr::Int(n) => Ok(Val::Int(*n)),
        Expr::Bool(b) => Ok(Val::Bool(*b)),
        Expr::Str(s) => Ok(Val::Str(s.clone())),
        Expr::Var(name) => st
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError(format!("unknown variable `{name}`"))),
        Expr::UnOp { op, expr } => {
            let v = eval(expr, st)?;
            if v == Val::Undisclosed {
                return Ok(Val::Undisclosed); // contagion
            }
            apply_unop(*op, v)
        }
        Expr::BinOp { op, lhs, rhs } => eval_binop(*op, lhs, rhs, st),
        Expr::Call { name, args } => {
            let mut argv = Vec::with_capacity(args.len());
            for a in args {
                argv.push(eval(a, st)?);
            }
            call_function(name, argv, st)
        }
        // Clearance is a *static* property (checked in `types/`); at runtime the
        // evaluator is the insider computing ACTUAL, so `read`, casts, and the
        // `self_defense` bypass all reduce to their operand's real value. (A
        // declassify's `E`-substitution is a face-rendering concern, not a change to
        // the scalar value.)
        Expr::Read(e) | Expr::SelfDefense(e) => eval(e, st),
        Expr::Cast { expr, .. } => eval(expr, st),
        Expr::External(args) => {
            // The foreign interface — only reachable in a covert scope (§7.6).
            if !st.covert {
                return Err(EvalError(
                    "external(...) is only available in a mossad scope".into(),
                ));
            }
            // The args describe a declared effect (a foreign target + payload), not a
            // computed value; they are rendered, not evaluated (deterministic mock).
            let rendered: Vec<String> = args.iter().map(pretty).collect();
            st.record(
                "(external liaison: nothing to report)",
                format!(
                    "external({}) \u{2192} undisclosed [foreign call; effect logged, neither confirm nor deny]",
                    rendered.join(", ")
                ),
            );
            // The result is `undisclosed` — contagious within the scope.
            Ok(Val::Undisclosed)
        }
    }
}

fn eval_binop(op: BinOp, lhs: &Expr, rhs: &Expr, st: &mut State) -> EvalResult {
    // Short-circuiting boolean operators (also propagate `undisclosed` contagion).
    match op {
        BinOp::And => {
            let l = eval(lhs, st)?;
            return match l {
                Val::Undisclosed => Ok(Val::Undisclosed),
                Val::Bool(false) => Ok(Val::Bool(false)),
                Val::Bool(true) => match eval(rhs, st)? {
                    Val::Undisclosed => Ok(Val::Undisclosed),
                    Val::Bool(b) => Ok(Val::Bool(b)),
                    other => type_err("&&", &other),
                },
                other => type_err("&&", &other),
            };
        }
        BinOp::Or => {
            let l = eval(lhs, st)?;
            return match l {
                Val::Undisclosed => Ok(Val::Undisclosed),
                Val::Bool(true) => Ok(Val::Bool(true)),
                Val::Bool(false) => match eval(rhs, st)? {
                    Val::Undisclosed => Ok(Val::Undisclosed),
                    Val::Bool(b) => Ok(Val::Bool(b)),
                    other => type_err("||", &other),
                },
                other => type_err("||", &other),
            };
        }
        _ => {}
    }

    let l = eval(lhs, st)?;
    if l == Val::Undisclosed {
        return Ok(Val::Undisclosed);
    }
    let r = eval(rhs, st)?;
    if r == Val::Undisclosed {
        return Ok(Val::Undisclosed);
    }
    apply_binop(op, l, r)
}

fn apply_unop(op: UnOp, v: Val) -> EvalResult {
    match (op, v) {
        (UnOp::Neg, Val::Int(n)) => Ok(Val::Int(-n)),
        (UnOp::Not, Val::Bool(b)) => Ok(Val::Bool(!b)),
        (UnOp::Neg, other) => type_err("unary -", &other),
        (UnOp::Not, other) => type_err("!", &other),
    }
}

fn apply_binop(op: BinOp, l: Val, r: Val) -> EvalResult {
    use BinOp::*;
    match op {
        // Equality works across the value set (mismatched kinds are simply not equal).
        Eq => Ok(Val::Bool(l == r)),
        Ne => Ok(Val::Bool(l != r)),
        // Arithmetic and ordering require integers.
        Add | Sub | Mul | Div | Mod | Lt | Le | Gt | Ge => match (l, r) {
            (Val::Int(a), Val::Int(b)) => int_binop(op, a, b),
            (a, _) => type_err(op.as_str(), &a),
        },
        // And/Or handled in `eval_binop`.
        And | Or => unreachable_binop(),
    }
}

fn int_binop(op: BinOp, a: i64, b: i64) -> EvalResult {
    use BinOp::*;
    let v = match op {
        Add => Val::Int(a.wrapping_add(b)),
        Sub => Val::Int(a.wrapping_sub(b)),
        Mul => Val::Int(a.wrapping_mul(b)),
        Div => {
            if b == 0 {
                return Err(EvalError("division by zero".into()));
            }
            Val::Int(a.wrapping_div(b))
        }
        Mod => {
            if b == 0 {
                return Err(EvalError("modulo by zero".into()));
            }
            Val::Int(a.wrapping_rem(b))
        }
        Lt => Val::Bool(a < b),
        Le => Val::Bool(a <= b),
        Gt => Val::Bool(a > b),
        Ge => Val::Bool(a >= b),
        Eq | Ne | And | Or => unreachable_binop()?,
    };
    Ok(v)
}

fn call_function(name: &str, argv: Vec<Val>, st: &mut State) -> EvalResult {
    let Some((params, body)) = st.funcs.get(name).cloned() else {
        return Err(EvalError(format!("unknown function `{name}`")));
    };
    if params.len() != argv.len() {
        return Err(EvalError(format!(
            "`{name}` expects {} argument(s), got {}",
            params.len(),
            argv.len()
        )));
    }
    st.env.push_frame();
    for (p, v) in params.iter().zip(argv) {
        st.env.define_local(p, v);
    }
    let flow = exec_block(&body, st);
    st.env.pop_frame();
    match flow? {
        Flow::Return(v) => Ok(v),
        Flow::Next => Ok(Val::Unit),
    }
}

fn type_err(op: &str, v: &Val) -> EvalResult {
    Err(EvalError(format!(
        "type error: `{op}` not applicable to {}",
        v.render()
    )))
}

/// And/Or never reach `apply_binop`/`int_binop`; this makes that explicit without a
/// wildcard arm masking a real missing case.
fn unreachable_binop() -> EvalResult {
    Err(EvalError(
        "internal: boolean operator routed to the arithmetic path".into(),
    ))
}

#[cfg(test)]
mod tests;
