//! The abstract syntax tree.
//!
//! Idiomatic Rust enums-with-data (rather than the flat `NodeKind` + `children`
//! sketch in Appendix H) — this makes the taxonomy **closed** and gives strictly
//! stronger exhaustiveness: a consumer that forgets a statement kind fails to
//! compile, and each variant carries exactly its own typed payload. The sketch is
//! explicitly adjustable ("Port whichever you choose; keep the sets closed and match
//! them exhaustively everywhere").
//!
//! The set grows one deliberate, reviewed step per build phase (handoff §9). This is
//! the Phase-0 subset (the two-tape spine); later phases add variants together with
//! their parser, runtime, and emitter handling.

/// A whole program: the mandatory grand operation name (#20) and its top-level body.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub op_name: String,
    pub body: Vec<Stmt>,
}

/// The right-hand side of a Phase-0 `declare` comparison: a literal or a variable.
/// (Generalized to full expressions in Phase 1.)
#[derive(Clone, Debug, PartialEq)]
pub enum DeclRhs {
    Int(i64),
    Var(String),
}

/// A statement. **Closed set** — matched exhaustively in `types/`, `runtime/`.
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    /// `x = <int>;` — Phase-0 assignment (generalized to `x = <expr>;` in Phase 1).
    Assign { var: String, value: i64 },

    /// `declare(<lhs> == <rhs>);` — writes OFFICIAL, logs a discrepancy if false,
    /// never halts (invariant I3). `assert` is a surface alias (it asserts nothing).
    Declare { lhs: String, rhs: DeclRhs },

    /// `hasbara(<talking_point>) { … }` — the gate: the talking point is declared up
    /// front, then the body runs. Classified ops are legal only inside (checked in
    /// `types/`, Phase 4).
    Hasbara {
        talking_point: String,
        body: Vec<Stmt>,
    },

    /// A sanctioned action `verb(target);` (e.g. `neutralize(target)`). The candid
    /// verb is insider data from the action table; the OFFICIAL face is `E(candid)`.
    /// `self_defense` marks the universal cast (#7); its parsing lands in Phase 2.
    Action {
        verb: String,
        target: String,
        self_defense: bool,
    },
}
