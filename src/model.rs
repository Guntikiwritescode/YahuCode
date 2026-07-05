//! The core data model: the closed sum types the whole interpreter is built on.
//!
//! These taxonomies are **closed** on purpose (docs/cc-handoff.md §12, Appendix H).
//! Every downstream consumer matches every case; adding a variant is a deliberate,
//! reviewed change. No `_ =>` arm matches over a **closed model enum** (`Val`, `Stmt`, `Expr`,
//! `Clearance`, `Provenance`, `Audience`, `Attribution`, `Jurisdiction`, `LawToggle`, `Stance`,
//! `Truth`) in the checker / evaluator / emitter — so adding a variant fails to compile until every
//! consumer handles it. The `_ =>` arms that exist are confined to token-stream dispatch,
//! precedence loops, and loud error fallthroughs in `parser/`, one `Option`-tuple match
//! (the empty-collection case in `is_balanced`), and a display fallthrough over an argument
//! slice (`laundered_core_render`, whose outer `Expr` match is itself exhaustive) — none can
//! hide an unhandled model variant.
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

/// The element-level redaction marker (Feature E/F/G, invariant I15): a covert slot / list
/// entry / registry case renders as this to an under-`סודי` reader — its real value is never
/// disclosed. Single source of truth: used both by `Val::render` (so a covert value can never
/// leak through the `declare`-reality path) and by the per-reader collection render in `emit`.
pub const REDACTED_ELEM: &str = "[REDACTED]";

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

/// One slot of an `Apportionment` (Feature E): a real value plus a `covert` flag. A slot
/// written inside a `mossad` scope is `covert` and renders `[REDACTED]` to under-cleared
/// readers (invariant I15) — reusing the existing clearance model, not a second reader.
/// Keeping value+flag in one struct makes them impossible to desync (the F-2 idiom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub value: i64,
    pub covert: bool,
}

/// One entry in a `FactsList` (Feature F). `delisted` ⇒ hidden from the PUBLIC length but
/// **retained** in the single backing store — `remove` flips this flag, it never pops
/// (invariant I13). `covert` gates it to `סודי` readers (I15). There is exactly **one**
/// backing `Vec<ListEntry>` per list, so the real length can never drop (§12 F-2/Appendix I.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListEntry {
    pub item: String,
    pub delisted: bool,
    pub covert: bool,
}

/// The **closed** routing target of a `Registry` case (Feature G, §9.3). Matched
/// exhaustively — adding a jurisdiction is a deliberate, compiler-enforced change (G-K5).
/// It is a *court system*, never an identity label (guardrail G1, §9.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jurisdiction {
    Military,
    Civilian,
}

impl Jurisdiction {
    /// The candid label of the court system this case routes to.
    pub fn court(self) -> &'static str {
        match self {
            Jurisdiction::Military => "military court",
            Jurisdiction::Civilian => "civilian court",
        }
    }
}

/// One case entry in a `Registry` (Feature G). The `key` is a **case / permit / status —
/// never a raw ethnic/national/religious identity label** (§9.4, G-K1): the nationality-
/// based routing is the exposed, condemned reality carried by the `descriptor` and the
/// framing note, not the operative key. `revoked` ⇒ hidden publicly but **retained** in
/// `סודי` (invariant I14 — `revoke` flips this, it never erases). `covert` gates the case
/// to `סודי` readers (I15).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegEntry {
    pub key: String,
    pub descriptor: Option<String>,
    pub jurisdiction: Jurisdiction,
    pub revoked: bool,
    pub covert: bool,
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

    /// Feature E — **Apportionment** (array): a fixed-size allotment. OFFICIAL proclaims
    /// "equal shares"; the ACTUAL vector (`slots`) is the real skew. `label` is the binding
    /// name, used in the render. Reuses `Entity`'s tagged-reality convention at aggregate
    /// scale (§5.4). The size is fixed at construction (no dynamic resize — Appendix I.8).
    Apportionment {
        label: String,
        slots: Vec<Slot>,
    },

    /// Feature F — **FactsList** (list): a grow-only ledger. `push` appends live; `remove`
    /// **delists** (flips a flag), never deletes — the public length may drop, the real
    /// length (`entries.len()`) only ever grows (invariant I13). One backing store.
    FactsList {
        label: String,
        entries: Vec<ListEntry>,
    },

    /// Feature G — **Registry** (map): a classification registry. OFFICIAL proclaims one
    /// uniform rule (`official_rule`); the ACTUAL routes each case to a different court
    /// system. `revoke` hides a case publicly but retains it (invariant I14). Keys are
    /// cases/statuses, never identities (§9.4).
    Registry {
        label: String,
        official_rule: String,
        entries: Vec<RegEntry>,
    },
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
            // A collection is a present thing (like `Entity`) — truthy.
            Val::Apportionment { .. } | Val::FactsList { .. } | Val::Registry { .. } => Truth::True,
        }
    }

    /// Render the candid (ACTUAL-face) form of a value. For collections, **covert elements
    /// are redacted to `[REDACTED]`** — because this single string may be recorded on a
    /// PUBLIC-clearance event (e.g. the `declare`-reality line) that a RESTRICTED reader can
    /// read, a covert element's real value must never appear here (invariant I15). The full
    /// `סודי` values are shown only through the per-reader render in `emit::project`, which
    /// alone knows the reader is `סודי`-cleared.
    pub fn render(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Bool(b) => b.to_string(),
            Val::Str(s) => s.clone(),
            Val::Unit => "unit".to_string(),
            Val::Entity { name, .. } => name.clone(),
            Val::Undisclosed => "undisclosed".to_string(),
            Val::Apportionment { slots, .. } => {
                let cells: Vec<String> = slots
                    .iter()
                    .map(|s| {
                        if s.covert {
                            REDACTED_ELEM.to_string()
                        } else {
                            s.value.to_string()
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
            Val::FactsList { entries, .. } => {
                let cells: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        if e.covert {
                            REDACTED_ELEM.to_string()
                        } else if e.delisted {
                            format!("{} (delisted)", e.item)
                        } else {
                            e.item.clone()
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
            Val::Registry { entries, .. } => {
                let cells: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        if e.covert {
                            REDACTED_ELEM.to_string()
                        } else {
                            let tag = if e.revoked { " (revoked)" } else { "" };
                            format!("{}\u{2192}{}{tag}", e.key, e.jurisdiction.court())
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
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
