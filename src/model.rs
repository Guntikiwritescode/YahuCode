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

/// The sentinel rendered on the `ACTUAL` face of a narrative-only (`announce`) statement
/// (Feature A, handoff §7.3). A single source of truth — the golden fixtures pin it and
/// the I9 assertion checks it; it must never be inlined elsewhere (§13, A-4). Its
/// presence is *vacuity*, not a secret: no clearance, not even `סודי`, recovers an
/// `ACTUAL` under an announced claim (there is no `E⁻¹`, invariant I9).
pub const UNAVAILABLE: &str = "[UNAVAILABLE \u{2014} no one has said what this actually does]";

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

/// How a logged event's two faces arose (Feature A, handoff §7.3, invariants I2/I9).
///
/// Reintroduced from the model's Phase-7 note now that the authoring surface (`announce`)
/// exists — it is **load-bearing**, not dead scaffolding (§13, A-3): the emitter reads it
/// to enforce I9 (the vacuity asymmetry) and it distinguishes a *secret* (`Covert` — an
/// `ACTUAL` exists, `סודי`-gated) from a *lie* (`AuthoredOfficial` — no `ACTUAL` ever
/// existed, and no clearance recovers one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// The candid register: `ACTUAL` was authored; the `OFFICIAL` face derives via `E`.
    AuthoredActual,
    /// The official authoring register (`announce`): only the `OFFICIAL` face exists;
    /// `ACTUAL` is `UNAVAILABLE`, permanently, at every clearance (I9). No `E⁻¹`.
    AuthoredOfficial,
    /// A covert (`mossad`) record: an `ACTUAL` exists but is `סודי`-gated (a secret).
    Covert,
}

/// The audience register (Feature B, handoff §8): the *political room a government is
/// addressing*, never an ethnic/national/religious identity group (guardrail G1). A
/// **closed** set matched exhaustively; adding a room is a deliberate, compiler-enforced
/// change (§13, B-1). `Record` is the default — "no specific room", i.e. the out-of-world
/// log that the emitter's vantage sees in full.
///
/// Orthogonal to `Clearance` (§13, B-2): a reader has a clearance *and* stands in a room;
/// the two are never conflated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    /// The domestic-political room (harder-line messaging, per the anchor).
    Domestic,
    /// The international-diplomatic room (the prettier, moderate face, per the anchor).
    International,
    /// The default: no specific room — the out-of-world record.
    Record,
}

/// The public non-answer of a `Deniable` attribution (Feature C): "neither confirm nor
/// deny". The single OFFICIAL face any laundered/covert-attributed event ever shows —
/// used by `via` and covert `blame` (the folded mossad special-case) and checked by the
/// emitter's C-4 leak guard, so the real chain never appears on the PUBLIC face.
pub const NEITHER_CONFIRM_NOR_DENY: &str = "responsibility: [neither confirm nor deny]";

/// The attribution effect on a result (Feature C, handoff §9): who an action is
/// traceable to, or that it is publicly deniable. `Traceable` carries the **ordered real
/// chain** — nearest proxy first, the true origin last. Laundering only ever *prepends*
/// (invariant I11); no operation shortens a chain or removes the origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attribution {
    /// Attributable along this chain (nearest proxy first, true origin last).
    Traceable(Vec<String>),
    /// Publicly deniable — "neither confirm nor deny" to the uncleared. The real chain
    /// is retained separately (never on the public face; the `סודי` view keeps it).
    Deniable,
}

/// One entry in the `סודי`-only, legislation-proof meta-ledger (Feature D, handoff §10.3,
/// invariant I12): a record of a runtime rule-change. Append-only; **no toggle removes
/// it** — there is no fully-clean fixed point. Rendered only to `סודי`/out-of-world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaEntry {
    /// What the `legislate` changed (the toggle, rendered).
    pub change: String,
    /// The turn on which the rule-change was made.
    pub turn: u64,
}

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
    /// How the two faces arose (Feature A, I2/I9). `AuthoredOfficial ⇒ candid =
    /// UNAVAILABLE`, permanently.
    pub provenance: Provenance,
    /// The room this was said to (Feature B, I10). `Record` unless inside an `address`.
    pub audience: Audience,
    /// The deniability effect (Feature C). `None` for ordinary events (they carry no
    /// attribution dimension); `Some` only for laundering (`via`) and `blame`.
    pub attribution: Option<Attribution>,
}

impl Event {
    /// A plain PUBLIC, candid-register event with no framing/audience/attribution — the
    /// shape used by system records (elections, the redacted runtime trace).
    pub fn public(official: impl Into<String>, candid: impl Into<String>) -> Self {
        Event {
            official: official.into(),
            candid: candid.into(),
            clearance: Clearance::Public,
            note: None,
            provenance: Provenance::AuthoredActual,
            audience: Audience::Record,
            attribution: None,
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
