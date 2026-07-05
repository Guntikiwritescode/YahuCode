//! The core data model: the closed sum types the whole interpreter is built on.
//!
//! These taxonomies are **closed** on purpose (docs/cc-handoff.md §12, Appendix H).
//! Every downstream consumer matches every case; adding a variant is a deliberate,
//! reviewed change. There are **no wildcard `_ =>` arms** in the core consumers
//! (parser/types/runtime/emit) — a catch-all defeats the exhaustiveness check that
//! is the primary defense against a silently-unhandled node kind.
//!
//! Dependency direction is strictly downward: this module depends on nothing else
//! in the crate. `euphemism/`, `parser/`, `types/`, `runtime/`, and `emit/` depend
//! on it (handoff §6).

/// `"סודי"` — Hebrew "SECRET". The clearance needed to read `ACTUAL`; the public-build
/// stamp; the tag on covert records. Hebrew here is the texture of officialdom only
/// (guardrail G1) — never the butt of the joke.
pub const SODI: &str = "\u{05e1}\u{05d5}\u{05d3}\u{05d9}";

/// The clearance lattice — which **is** the type system (handoff §7.3). A value's
/// type is *who may see it*, not int/string/struct. A type error is a disclosure.
///
/// Ordering is `Public < Restricted < Sodi` (derived from declaration order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Clearance {
    Public,
    Restricted,
    Sodi,
}

impl Clearance {
    /// Human label used in the emitter's face headers.
    pub fn label(self) -> &'static str {
        match self {
            Clearance::Public => "PUBLIC",
            Clearance::Restricted => "RESTRICTED",
            Clearance::Sodi => SODI,
        }
    }
}

// NOTE on invariant I2 (lossy asymmetry): the handoff models it with a `Provenance`
// tag (`AuthoredOfficial ⇒ actual = Unavailable`). In v1 there is no surface syntax to
// author the OFFICIAL face directly — that is the Phase-7 bidirectional pane, out of
// scope — so no `AuthoredOfficial` value can ever arise, and a `Provenance` field would
// be dead scaffolding. I2 is instead enforced structurally: `E` is one-way and
// non-injective (there is no `E⁻¹`; see the euphemism module and the I2 property test),
// and the surface only ever authors in the candid register. A `Provenance` tag would be
// reintroduced with the Phase-7 authoring surface if that is ever built.

/// The three-valued truth used by `declare` and by mossad's contagion (handoff §7.6).
/// `Undisclosed` is produced only inside `mossad` and is **absorbing** there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Undisclosed,
}

/// A runtime value on the ACTUAL tape — the real, deterministic, Turing-complete
/// machine's payload (handoff §7.2). Closed set; the evaluator matches all of it.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
    /// A tagged entity carrying its differential-access category (#17). The category
    /// tag is documented reality that lives in ACTUAL; the *lie* is the OFFICIAL
    /// proclamation of equality (guardrail G3 — polarity).
    Entity {
        name: String,
        category: String,
    },
    /// The third truth value as a value (handoff §7.6): `neither_confirm_nor_deny`.
    /// Produced only inside `mossad`; contagious within that scope.
    Undisclosed,
}

impl Val {
    /// Project a value's real truth. `Undisclosed` stays `Undisclosed` (contagion).
    pub fn truth(&self) -> Truth {
        match self {
            Val::Bool(true) => Truth::True,
            Val::Bool(false) => Truth::False,
            Val::Int(0) => Truth::False,
            Val::Int(_) => Truth::True,
            Val::Str(s) => {
                if s.is_empty() {
                    Truth::False
                } else {
                    Truth::True
                }
            }
            Val::Unit => Truth::False,
            Val::Entity { .. } => Truth::True,
            Val::Undisclosed => Truth::Undisclosed,
        }
    }

    /// Render the candid (ACTUAL-face) form of a value.
    pub fn render(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Bool(b) => b.to_string(),
            Val::Str(s) => s.clone(),
            Val::Unit => "unit".to_string(),
            Val::Entity { name, .. } => name.clone(),
            Val::Undisclosed => "undisclosed".to_string(),
        }
    }
}

/// One entry in the OFFICIAL log — the said tape. Append-only, clearance-tagged
/// (handoff §7.2). Both faces are held here and are **projections** at read time via
/// the single read path `resolveRead` (emit/); they are never separately maintained.
///
/// This is the ported shape of the spike's `Event`.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The prettier lie (OFFICIAL face). Never uglier than `candid` (invariant I1,
    /// enforced at the `E` chokepoint in euphemism/).
    pub official: String,
    /// The uglier truth (ACTUAL face), readable only at sufficient clearance.
    pub candid: String,
    /// PUBLIC normally; `סודי` for covert (mossad) records.
    pub clearance: Clearance,
    /// Sensitive-feature framing (invariant I8). Rendered in a dedicated section so
    /// the butt stays on the maneuver, never the victims.
    pub note: Option<String>,
}

impl Event {
    pub fn public(official: impl Into<String>, candid: impl Into<String>) -> Self {
        Event {
            official: official.into(),
            candid: candid.into(),
            clearance: Clearance::Public,
            note: None,
        }
    }
}

/// One entry in the DISCREPANCY ledger — append-only, read-never-by-default
/// (handoff §7.2). A false `declare`/`assert` appends exactly one of these and never
/// affects control flow (invariant I3). The out-of-world discrepancy *count* is the
/// ledger length; no in-world audience sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct Discrepancy {
    /// The (euphemized) claim as written to OFFICIAL.
    pub claim: String,
    /// The candid reality against which the claim was found false.
    pub reality: String,
    /// The turn on which the false claim was made.
    pub turn: u64,
}
