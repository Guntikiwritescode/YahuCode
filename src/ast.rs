//! The abstract syntax tree.
//!
//! Idiomatic Rust enums-with-data (rather than the flat `NodeKind` + `children`
//! sketch in Appendix H) — this makes the taxonomy **closed** and gives strictly
//! stronger exhaustiveness: a consumer that forgets a case fails to compile, and each
//! variant carries exactly its own typed payload. The sketch is explicitly adjustable
//! ("Port whichever you choose; keep the sets closed and match them exhaustively").
//!
//! The set grows one deliberate, reviewed step per build phase (handoff §9). This is
//! the Phase-1 surface (real control flow on ACTUAL); later phases add casts/reads
//! (Phase 2), coalition ops (Phase 3), mossad/undisclosed (Phase 5), and the feature
//! statements (Phase 6) together with their runtime and emitter handling.

/// A whole program: the mandatory grand operation name (#20), its top-level body, and
/// any source `# …` comments (kept so the press build can rewrite them, #8).
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub op_name: String,
    pub body: Vec<Stmt>,
    pub comments: Vec<String>,
}

/// The read-only-and-inert globals (#11 backlog). Closed set — matched exhaustively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InertKind {
    WorldOpinion,
    Polls,
}

/// Unary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    /// arithmetic negation `-e`
    Neg,
    /// boolean negation `!e`
    Not,
}

/// Binary operators (closed set).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    /// Source rendering, used by `declare`'s claim pretty-printer.
    pub fn as_str(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

/// An expression on the ACTUAL tape — the real, deterministic, Turing-complete core.
/// **Closed set** — the evaluator matches every variant, no wildcard arms.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// A call to a user-defined function (`func`). Built-in actions are the separate
    /// `Stmt::Action`; other built-ins arrive as their own statements per phase.
    Call {
        name: String,
        args: Vec<Expr>,
    },

    /// `read(e)` — the clearance-gated read (the ONE read path). In an expression it
    /// resolves the value for the current reader, so it lowers the static clearance to
    /// the reading context (Phase 2).
    Read(Box<Expr>),

    /// `(PUBLIC|RESTRICTED|סודי) e` — a clearance cast. Reclassify-up is always sound;
    /// declassify-down swaps in `E(actual)` at the boundary. The static clearance of
    /// the result is the cast target (Phase 2).
    Cast {
        target: crate::model::Clearance,
        expr: Box<Expr>,
    },

    /// `(self_defense) e` — the universal cast (#7): always type-checks, at any
    /// magnitude; the one sanctioned bypass of the disclosure check (Phase 2).
    SelfDefense(Box<Expr>),

    /// `external(args…)` — the mossad foreign interface (handoff §7.6). In v1 a
    /// deterministic mock: it logs a declared effect and returns `undisclosed`
    /// (`neither_confirm_nor_deny`), which is contagious within the covert scope.
    External(Vec<Expr>),
}

/// A statement. **Closed set** — matched exhaustively in `runtime/` (and `types/`).
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    /// `x = <expr>;` — assignment; the rvalue really computes on ACTUAL.
    Assign { var: String, value: Expr },

    /// `declare(<expr>);` / `assert(<expr>);` — writes OFFICIAL, logs a discrepancy if
    /// the claim is really false against ACTUAL, never halts (invariant I3).
    Declare(Expr),

    /// `if (<cond>) { … } else { … }` — branches on ACTUAL.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },

    /// `while (<cond>) { … }` — loops on ACTUAL.
    While { cond: Expr, body: Vec<Stmt> },

    /// `func name(params) { … }` — a user-defined function.
    FuncDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// `return <expr>?;` — returns from the enclosing function (never halts the whole
    /// program; the only halt is `elections`, I6).
    Return(Option<Expr>),

    /// A bare expression used as a statement (e.g. a user function call `f(x);`).
    ExprStmt(Expr),

    /// `hasbara(<talking_point>) { … }` — the gate; the talking point is declared up
    /// front, then the body runs.
    Hasbara {
        talking_point: String,
        body: Vec<Stmt>,
    },

    /// A sanctioned action `verb(target);` (e.g. `neutralize(target)`). The candid verb
    /// is insider data from the action table; the OFFICIAL face is `E(candid)`.
    /// `self_defense` marks the universal cast (#7).
    Action {
        verb: String,
        target: String,
        self_defense: bool,
    },

    /// `let name = allocate(what);` — a coalition allocation. It costs coalition support
    /// each turn (charged on `postpone`); it is never freed (memory → coalition, §7.5).
    Allocate { name: String, what: String },

    /// `bribe(name, amount);` — top up coalition core support.
    Bribe { name: String, amount: Expr },

    /// `postpone();` — the most-called stdlib function: a turn-consuming no-op that
    /// charges upkeep for every live allocation (§7.5).
    Postpone,

    /// `elections;` — the only explicit in-world halt (invariant I6). The government
    /// dissolves; nothing after it runs.
    Elections,

    /// `mossad { … }` — a covert scope (§7.6). Its operations affect ACTUAL but are
    /// `סודי`-tagged: absent from the PUBLIC record, redacted for RESTRICTED, candid for
    /// `סודי` insiders. It is also the foreign interface and the source of `undisclosed`.
    Mossad { body: Vec<Stmt> },

    /// `blame(who);` — responsibility that never resolves to `self` (#10, invariant I4).
    /// Inside `mossad` it resolves by clearance: publicly `neither confirm nor deny`,
    /// insider-attributable to the real actor.
    Blame { who: String },

    /// `raise(name);` — raise an error (pushed to the pending stack; #3/#9).
    Raise { name: String },

    /// `whatabout(name);` — suppress a raised error by pointing elsewhere (#9). Never
    /// resolves the error.
    Whatabout { name: String },

    /// `ceasefire;` — reads like `break`; lowers to a no-op `continue` (#18). The loop
    /// resumes; nothing stops. The butt is the euphemistic word, not who violates it.
    Ceasefire,

    /// `deeply_concerned();` / `concern(who);` — an ally's no-op: nothing changes,
    /// support continues (#11).
    Concern { who: Option<String> },

    /// `criticism(subject);` — #19: criticism of government conduct universally miscast
    /// to an attack on identity, silencing the critic (the false positive).
    Criticism { subject: String },

    /// `antisemitism(incident);` — #19: a REAL antisemitism incident that exists in
    /// ACTUAL and that the deflection-alarm never fires on (the mandatory false
    /// negative that keeps "antisemitism is real" un-eraseable).
    Antisemitism { incident: String },

    /// `access(entity);` — #17: differential access laundered to a proclamation of
    /// equal rights. Inequality lives in ACTUAL; the lie is the equality claim.
    Access { entity: String },

    /// `timeline(symbol);` — #16: a pre-`t=0` context symbol is ruled out of scope
    /// (`TimelineError`); the butt is the context-erasure maneuver.
    Timeline { symbol: String },

    /// `let name = establish_commission(subject);` — #13: a commission engineered to
    /// resolve only after the matter is moot.
    EstablishCommission { name: String, subject: String },

    /// `let name = settlement(what);` — #14: a grow-only, upkeep-exempt, never-freed
    /// allocation ("facts on the ground").
    Settlement { name: String, what: String },

    /// `human_shields(verb(target));` — #15: an exception-legalizer that suppresses a
    /// civilian-harm op's exception, never verifies the shield claim, and reassigns
    /// responsibility onto the harmed party. The butt is the excuse's elasticity.
    HumanShields { verb: String, target: String },

    // ─── backlog features (§8), reopened: on-aim government-rhetoric maneuvers ───
    /// `proportionate(claim);` — a self-certifying "proportionate response" assertion
    /// that always passes, at any magnitude; no proportionality test is applied.
    Proportionate { claim: String },

    /// `disputed(name, official, actual);` — a contested figure: the OFFICIAL face
    /// shows the (lower) press number, the ACTUAL face the real one. The butt is the
    /// figure-dispute / lowballing maneuver.
    Disputed {
        name: String,
        official: i64,
        actual: i64,
    },

    /// `deny(event);` — an official denial. ACTUAL records that the event occurred;
    /// OFFICIAL categorically denies it (denial ≠ non-occurrence).
    Deny { event: String },

    /// `world_opinion();` / `polls();` — read-only-and-inert: they can be consulted but
    /// never affect ACTUAL. The butt is treating them as decorative.
    Inert { kind: InertKind },

    /// `investigate(subject);` — a self-exonerating investigation: the investigated
    /// party investigates itself, with a predetermined "no wrongdoing" outcome.
    Investigate { subject: String },

    /// `address_international();` — a hollow address to the international community; a
    /// speech that changes nothing (a no-op, like `concern`).
    AddressInternational,
}

/// Collect the variables referenced by an expression, in first-appearance order,
/// de-duplicated. Used by `declare` to render the "reality" of a claim.
pub fn vars_in(expr: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    collect_vars(expr, &mut out);
    out
}

fn collect_vars(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) => {}
        Expr::Var(name) => {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        Expr::UnOp { expr, .. } => collect_vars(expr, out),
        Expr::BinOp { lhs, rhs, .. } => {
            collect_vars(lhs, out);
            collect_vars(rhs, out);
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_vars(a, out);
            }
        }
        Expr::Read(e) | Expr::SelfDefense(e) => collect_vars(e, out),
        Expr::Cast { expr, .. } => collect_vars(expr, out),
        Expr::External(args) => {
            for a in args {
                collect_vars(a, out);
            }
        }
    }
}

/// Pretty-print an expression back to source form — used to render a `declare` claim.
pub fn pretty(expr: &Expr) -> String {
    match expr {
        Expr::Int(n) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Str(s) => format!("\"{s}\""),
        Expr::Var(name) => name.clone(),
        Expr::UnOp { op, expr } => {
            let sym = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
            };
            format!("{sym}{}", pretty(expr))
        }
        Expr::BinOp { op, lhs, rhs } => {
            format!("{} {} {}", pretty(lhs), op.as_str(), pretty(rhs))
        }
        Expr::Call { name, args } => {
            let a: Vec<String> = args.iter().map(pretty).collect();
            format!("{name}({})", a.join(", "))
        }
        Expr::Read(e) => format!("read({})", pretty(e)),
        Expr::Cast { target, expr } => format!("({}) {}", target.label(), pretty(expr)),
        Expr::SelfDefense(e) => format!("(self_defense) {}", pretty(e)),
        Expr::External(args) => {
            let a: Vec<String> = args.iter().map(pretty).collect();
            format!("external({})", a.join(", "))
        }
    }
}
