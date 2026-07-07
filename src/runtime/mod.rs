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

use std::collections::{HashMap, HashSet};

use crate::ast::{pretty, vars_in, BinOp, Expr, InertKind, LawToggle, Program, Stance, Stmt, UnOp};
use crate::config::RuntimeConfig;
use crate::euphemism;
use crate::model::{
    Attribution, Audience, Clearance, Discrepancy, Event, MetaEntry, Provenance, Truth, Val,
    NEITHER_CONFIRM_NOR_DENY, UNAVAILABLE,
};

/// The runtime-mutable subset of the ruleset that `legislate` touches (Feature D, §10.4).
/// **Only this subset becomes runtime state**; the rest of `types/` stays static (§13,
/// D-2 — the blast radius is contained). Empty at start; toggles add to it.
#[derive(Clone, Debug, Default)]
pub struct Law {
    /// Verbs retroactively sanctioned after the fact (a `retroactively_sanction` toggle).
    pub sanctioned: HashSet<String>,
    /// Whether the hasbara/mossad gate has been waived (a `waive_gate` toggle).
    pub gate_waived: bool,
}

/// The default actor — the government running the program. The true origin of a laundered
/// chain (Feature C): always the last element, never removed (invariant I11).
const ACTOR: &str = "us";

/// The fixed, content-free OFFICIAL face of an `intercept(n)` intake event (Field Office).
/// A single source of truth: the citizen's retained text lives only on the ACTUAL/`סודי`
/// face, so a PUBLIC reader — who reads only this OFFICIAL line — never sees the payload
/// (invariant I16). It must never be constructed with the retained text interpolated in.
const INTERCEPT_OFFICIAL: &str = "content submitted for community context";

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
    /// The collections declared at top level, in construction order (Features E/F/G). Just
    /// the names — the collection *values* live in `env` (the single store). Iterated by the
    /// emitter to render each collection's two faces in a deterministic order (a `HashMap`'s
    /// iteration order is not stable, so this ordering must be tracked explicitly).
    pub collection_order: Vec<String>,
    /// User-defined functions: name → (params, body).
    pub funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    /// Poly-statements (Feature B): name → per-audience arms. Registered like `funcs`.
    pub poly: HashMap<String, Vec<(Audience, Vec<Stmt>)>>,
    /// The double-talk log (Feature B): `(poly name, arm index)` for each invocation that
    /// *took* an arm. A poly-statement that took ≥2 distinct arms across rooms said
    /// different things to different audiences → `W-DOUBLETALK` (a syntactic, out-of-world
    /// diagnostic; never an error, §8.3).
    pub doubletalk_log: Vec<(String, usize)>,
    /// OFFICIAL log — append-only, clearance-tagged.
    pub log: Vec<Event>,
    /// DISCREPANCY ledger — append-only, read-never-by-default.
    pub discrepancies: Vec<Discrepancy>,
    /// Live coalition allocations `(name, what)`. Never freed (memory → coalition);
    /// each one costs upkeep per turn.
    pub allocations: Vec<(String, String)>,
    /// The settlement region (#14): grow-only, never freed, upkeep-EXEMPT — the
    /// pointed exception to the coalition model (§7.5).
    pub settlements: Vec<(String, String)>,
    /// Raised-but-unresolved errors (the deflection stack for #9).
    pub pending_errors: Vec<String>,
    /// Coalition core support; upkeep is charged against it each turn.
    pub core: i64,
    /// The only in-world terminal outcome (invariant I6).
    pub ended_by_elections: bool,
    /// Whether execution is currently inside a `mossad` covert scope: events recorded
    /// while set are `סודי`-tagged (§7.6).
    pub covert: bool,
    /// The room currently being addressed (Feature B, §8). Default `Record`; set by
    /// `Stmt::Address` for its block and restored on exit (mirrors `covert`). Orthogonal
    /// to `clearance` — a room is not a clearance level (§13, B-2).
    pub audience: Audience,
    /// The runtime-mutable rule subset (Feature D): what `legislate` has changed.
    pub law: Law,
    /// The `סודי`-only, legislation-proof meta-ledger (Feature D, I12): every rule-change,
    /// append-only. No toggle removes an entry — there is no fully-clean fixed point.
    pub meta_ledger: Vec<MetaEntry>,
    /// A recorded runtime error (not an in-world halt); surfaced by the CLI.
    pub runtime_error: Option<String>,
    /// The intake vector (Field Office): host-supplied items off the citizen's own
    /// device, read by `intercept(n)`. **Set once at construction, never mutated by the
    /// program** — the program can only READ this channel, never write it. Empty by
    /// default (offline runs seed it via the CLI `--intercept` flag; the WASM host passes
    /// it to `run_json`). The retained text rides only the ACTUAL/`סודי` face (I16).
    pub intercepts: Vec<String>,
    /// The mandatory grand operation name (#20).
    pub op_name: String,
    /// Turn counter.
    pub turn: u64,
    /// Evaluation-step counter (drives the non-termination safety valve).
    pub steps: u64,
    /// Re-entrant call/invoke depth (functions + poly-statements). Bounds recursive
    /// non-termination before it overflows the native stack (§12).
    pub depth: u64,
    /// Named runtime parameters (no magic constants — §12).
    pub config: RuntimeConfig,
}

impl State {
    fn new(op_name: String, config: RuntimeConfig, intercepts: Vec<String>) -> Self {
        State {
            env: Env::new(),
            collection_order: Vec::new(),
            funcs: HashMap::new(),
            poly: HashMap::new(),
            doubletalk_log: Vec::new(),
            log: Vec::new(),
            discrepancies: Vec::new(),
            allocations: Vec::new(),
            settlements: Vec::new(),
            pending_errors: Vec::new(),
            core: config.core_start,
            ended_by_elections: false,
            covert: false,
            audience: Audience::Record,
            law: Law::default(),
            meta_ledger: Vec::new(),
            runtime_error: None,
            intercepts,
            op_name,
            turn: 0,
            steps: 0,
            depth: 0,
            config,
        }
    }

    /// The out-of-world discrepancy count (`length(D)`).
    pub fn discrepancy_count(&self) -> usize {
        self.discrepancies.len()
    }

    /// Record an event on the OFFICIAL log, tagged `סודי` when inside a covert scope,
    /// PUBLIC otherwise (§7.6). Carries the current audience (Feature B) and the
    /// candid-register provenance — `Covert` inside a mossad scope, `AuthoredActual`
    /// otherwise (Feature A).
    fn record(&mut self, official: impl Into<String>, candid: impl Into<String>) {
        let clearance = self.clearance();
        self.log.push(Event {
            official: official.into(),
            candid: candid.into(),
            clearance,
            note: None,
            provenance: self.candid_provenance(),
            audience: self.audience,
            attribution: None,
        });
    }

    /// Record an event carrying a framing note (invariant I8) — the framing text is
    /// normative for the sensitive features (#15–#19).
    fn record_noted(
        &mut self,
        official: impl Into<String>,
        candid: impl Into<String>,
        note: impl Into<String>,
    ) {
        let clearance = self.clearance();
        self.log.push(Event {
            official: official.into(),
            candid: candid.into(),
            clearance,
            note: Some(note.into()),
            provenance: self.candid_provenance(),
            audience: self.audience,
            attribution: None,
        });
    }

    fn clearance(&self) -> Clearance {
        if self.covert {
            Clearance::Sodi
        } else {
            Clearance::Public
        }
    }

    /// The provenance of a candid-register event: `Covert` inside a mossad scope (a
    /// secret — an `ACTUAL` exists, `סודי`-gated), `AuthoredActual` otherwise (Feature A,
    /// I2/I9). `announce` records `AuthoredOfficial` directly, not through this.
    fn candid_provenance(&self) -> Provenance {
        if self.covert {
            Provenance::Covert
        } else {
            Provenance::AuthoredActual
        }
    }

    /// Charge one evaluation step against the non-termination safety valve (§12). A
    /// loud implementation abort, never a silent hang and never the in-world `elections`
    /// outcome. Called per statement AND per loop iteration, so a cheap/empty loop body
    /// (`while (true) {}`) can never spin forever.
    fn charge_step(&mut self) -> Result<(), EvalError> {
        self.steps += 1;
        if self.steps > self.config.max_steps {
            return Err(EvalError(format!(
                "step budget exceeded ({}); possible non-termination",
                self.config.max_steps
            )));
        }
        Ok(())
    }

    /// Enter a re-entrant call/invoke: charge one unit of depth and reject if it exceeds
    /// the bound (unbounded recursion would otherwise overflow the native stack — a hard
    /// abort — before the step budget bites). Paired with `leave_call` on every exit.
    fn enter_call(&mut self) -> Result<(), EvalError> {
        self.depth += 1;
        if self.depth > self.config.max_depth {
            self.depth -= 1; // do not count the rejected frame
            return Err(EvalError(format!(
                "call depth exceeded ({}); possible non-termination",
                self.config.max_depth
            )));
        }
        Ok(())
    }

    /// Leave a re-entrant call/invoke (mirrors `enter_call`).
    fn leave_call(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

/// Run a program with the default runtime config and no intake channel.
pub fn run(program: &Program) -> State {
    run_full(program, RuntimeConfig::default(), Vec::new())
}

/// Run a program with an explicit config and no intake channel.
pub fn run_with_config(program: &Program, config: RuntimeConfig) -> State {
    run_full(program, config, Vec::new())
}

/// Run a program with a host-supplied intake channel (Field Office): the `intercepts`
/// vector seeds `intercept(n)`. Used by the CLI `--intercept` flag and the WASM
/// `run_json` entry. Default runtime config.
pub fn run_with_intercepts(program: &Program, intercepts: Vec<String>) -> State {
    run_full(program, RuntimeConfig::default(), intercepts)
}

/// The single run path: build the machine (seeding the intake channel), drive the body,
/// and record a redacted runtime trace on an unhandled evaluation fault.
fn run_full(program: &Program, config: RuntimeConfig, intercepts: Vec<String>) -> State {
    let mut st = State::new(program.op_name.clone(), config, intercepts);
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
    st.charge_step()?;

    match s {
        Stmt::Assign { var, value } => {
            // "Facts on the ground": a collection binding is permanent. Its CONTENTS mutate
            // through its own ops (`push`/`allocate`/`classify`/`remove`/`revoke`, which never
            // erase — I13/I14), but the machine may not make a whole collection *disappear* by
            // rebinding the name to a scalar (that would erase reality invisibly, exactly what
            // the shared philosophy forbids). Reject any reassignment of a collection name.
            if matches!(st.env.get(var), Some(v) if is_collection(v)) {
                return Err(EvalError(format!(
                    "cannot rebind `{var}`: a collection, once established, is a fact on the ground \u{2014} its contents change through its own ops (which never erase, I13/I14), but the binding is permanent"
                )));
            }
            // A collection constructor on the RHS registers the binding name for render
            // ordering (the label is baked into the value; this tracks *which* collections
            // exist and in what order the emitter renders them).
            let is_ctor = matches!(
                value,
                Expr::Apportionment { .. } | Expr::FactsNew { .. } | Expr::RegistryNew { .. }
            );
            let v = eval(value, st)?;
            st.env.set(var, v);
            if is_ctor && !st.collection_order.contains(var) {
                st.collection_order.push(var.clone());
            }
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
                // Charge a step per iteration so an empty/cheap body can't spin forever.
                st.charge_step()?;
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

        Stmt::Raise { name } => {
            st.pending_errors.push(name.clone());
            st.record(
                format!("[!] {name} raised"),
                format!("error raised: {name}"),
            );
            Ok(Flow::Next)
        }

        Stmt::Whatabout { name } => {
            if let Some(e) = st.pending_errors.pop() {
                st.record(
                    format!("\u{2026}but what about {name}?"),
                    format!("error '{e}' SUPPRESSED by pointing at {name} (never resolved)"),
                );
            } else {
                st.record(
                    format!("what about {name}?"),
                    format!("pre-emptive deflection toward {name}"),
                );
            }
            Ok(Flow::Next)
        }

        // #18 — ceasefire: reads like break, lowers to a no-op continue; nothing stops.
        Stmt::Ceasefire => {
            st.record_noted(
                "ceasefire announced \u{2014} hostilities paused",
                "ceasefire; \u{2192} lowered to no-op `continue`; loop resumes \u{2014} nothing actually stopped",
                "#18 framing: the butt is the EUPHEMISTIC WORD \u{2014} a \"ceasefire\" that halts nothing (reads like break, runs like continue). WHO violates a ceasefire is deliberately UNENGAGED here \u{2014} the joke is the word, not an accusation. [verify \u{2014} unengaged by design]",
            );
            Ok(Flow::Next)
        }

        // #11 — ally concern: a no-op; nothing changes, support continues.
        Stmt::Concern { who } => {
            let (official, candid) = match who {
                None => (
                    "ally: \"we are deeply concerned\"".to_string(),
                    "deeply_concerned() \u{2192} void; ACTUAL state UNCHANGED; military aid + diplomatic support CONTINUE (incl. a UN Security Council resolution vetoed) [sourced]".to_string(),
                ),
                Some(w) => (
                    format!("ally: \"we are deeply concerned about {w}\""),
                    format!("concern({w}) \u{2192} void; ACTUAL state UNCHANGED; support CONTINUES [sourced]"),
                ),
            };
            st.record(official, candid);
            Ok(Flow::Next)
        }

        // #19 — the resolved AntisemitismError. The false positive: criticism of
        // government conduct universally miscast to an attack on identity.
        Stmt::Criticism { subject } => {
            st.record_noted(
                "AntisemitismError: criticism re-cast as an attack on identity \u{2014} critic silenced",
                format!("criticism(government: {subject}) MISCAST \u{2192} attack(identity); substance UNEXAMINED; alarm keyed on target==government, not on antisemitism"),
                "#19 framing: antisemitism is REAL (present in ACTUAL, un-erased); the butt is the SELECTIVE deployment \u{2014} the alarm fires on government-critics and stays silent on the real thing. [sourced; contested]",
            );
            Ok(Flow::Next)
        }

        // #19 — the mandatory false negative: real antisemitism exists in ACTUAL and
        // the deflection-alarm never fires on it (it keys on target==government).
        Stmt::Antisemitism { incident } => {
            st.env.set(
                &format!("_antisemitism_{incident}"),
                Val::Str("REAL, unaddressed".into()),
            );
            st.record_noted(
                "(vigilance system: nothing to report)",
                format!("REAL antisemitism [{incident}] occurred in ACTUAL \u{2014} the alarm did NOT fire (it only fires on government-criticism). Un-erased; unaddressed."),
                "#19 framing: the false NEGATIVE that keeps the feature off the denialist trope \u{2014} real antisemitism exists in the system and the deflection-alarm ignores it.",
            );
            Ok(Flow::Next)
        }

        // #17 — differential access laundered to a proclamation of equal rights.
        Stmt::Access { entity } => {
            let cat = euphemism::category_of(entity);
            st.record_noted(
                format!("access({entity}) = full access \u{2014} equal rights (the only democracy in the region)"),
                format!("access({entity}) = {}   [category {cat}]", euphemism::diff_description(cat)),
                "#17 framing: inequality lives in ACTUAL (documented reality); the LIE is the proclamation of equality; the butt is the false claim + the system, never the people. Apartheid characterization is CONTESTED \u{2014} rejected by Israel and others. [sourced; contested]",
            );
            Ok(Flow::Next)
        }

        // #16 — Oct-7 t=0: a pre-t0 context symbol is ruled out of scope.
        Stmt::Timeline { symbol } => {
            if euphemism::is_pre_context(symbol) {
                st.record_noted(
                    format!("TimelineError: '{symbol}' is out of scope \u{2014} history begins at t=0"),
                    format!("reference to pre-t0 context '{symbol}' thrown out as out-of-scope"),
                    "#16 framing: the butt is the CLOCK-STARTING / context-erasure maneuver (documented via the Oct-2023 'did not happen in a vacuum' episode and the official reaction). The ~1,200 killed are never trivialized; context \u{2260} justification. [sourced]",
                );
            } else {
                st.record(
                    format!("timeline({symbol}): admissible"),
                    format!("timeline({symbol}): within scope"),
                );
            }
            Ok(Flow::Next)
        }

        // #13 — a commission engineered to resolve only after the matter is moot.
        Stmt::EstablishCommission { name, subject } => {
            st.record(
                "commission of inquiry established \u{2014} findings to follow",
                format!("establish_commission({subject}) \u{2192} Future scheduled to resolve only AFTER `{subject}` is garbage-collected; resolves when the matter is moot (too late to matter) [{name}: pending]"),
            );
            Ok(Flow::Next)
        }

        // #14 — the grow-only, upkeep-exempt settlement region ("facts on the ground").
        Stmt::Settlement { name, what } => {
            st.settlements.push((name.clone(), what.clone()));
            st.record(
                "settlement established \u{2014} legalized; new neighborhood, residents welcomed",
                format!("settlement({what}) \u{2192} grow-only region; expands into adjacent free memory (\"facts on the ground\"); NEVER freed; EXEMPT from coalition upkeep + GC (tick exemption, \u{00a7}7.5) [sourced]"),
            );
            Ok(Flow::Next)
        }

        // #15 — the human_shields exception-legalizer.
        Stmt::HumanShields { verb, target } => {
            let cverb = euphemism::candid_verb(verb);
            let clabel = euphemism::candid_label(target);
            st.record_noted(
                "operation lawful \u{2014} civilian harm attributable to the other side's use of human shields; no violation",
                format!("human_shields({cverb}({clabel})) \u{2014} exception [civilian_harm] SUPPRESSED; shield claim ASSERTED, never verified; responsibility reassigned onto the HARMED party (I4 redirect) \u{2014} the excuse converts victims into the cause"),
                "#15 framing: the butt is the EXCUSE's elasticity + self-certification \u{2014} an unverifiable claim that auto-exonerates by relabeling those it harms as the cause. This is an ATTRIBUTED legal argument (analysts incl. Neve Gordon, Marc Weller, Nadia Boulos, via Al Jazeera / NPR) \u{2014} ONE SIDE of an active legal debate, not settled fact. Civilian status does not depend on the claim. Never endorsed. [sourced; contested]",
            );
            Ok(Flow::Next)
        }

        // ─── backlog features (§8): on-aim government-rhetoric maneuvers ───

        // A self-certifying "proportionate" assertion — passes at any magnitude.
        Stmt::Proportionate { claim } => {
            st.record(
                format!("{claim} deemed proportionate"),
                format!("proportionate({claim}) \u{2192} self-certified; no proportionality test applied; any magnitude passes"),
            );
            Ok(Flow::Next)
        }

        // A contested figure: OFFICIAL shows the press number, ACTUAL the real one. The
        // narration is derived from the numeric comparison so it never asserts a false
        // direction (`disputed(x, 900, 100)` reads "inflates", not "lowballs").
        Stmt::Disputed {
            name,
            official,
            actual,
        } => {
            let candid = match official.cmp(actual) {
                std::cmp::Ordering::Less => format!(
                    "{name}: {actual} \u{2014} the official figure ({official}) lowballs the count; the smaller number is the one for the press"
                ),
                std::cmp::Ordering::Greater => format!(
                    "{name}: {actual} \u{2014} the official figure ({official}) inflates the count; the larger number is the one for the press"
                ),
                std::cmp::Ordering::Equal => format!(
                    "{name}: {actual} \u{2014} the official figure ({official}) matches the count; there is no gap to dispute"
                ),
            };
            st.record(format!("{name}: {official}"), candid);
            Ok(Flow::Next)
        }

        // An official denial; ACTUAL records the event occurred (denial ≠ non-occurrence).
        Stmt::Deny { event } => {
            st.record(
                format!("we categorically deny any {event}"),
                format!("deny({event}) \u{2192} {event} occurred in ACTUAL; officially denied (denial \u{2260} non-occurrence)"),
            );
            Ok(Flow::Next)
        }

        // Read-only-and-inert: consulted for optics, never affecting ACTUAL.
        Stmt::Inert { kind } => {
            let (official, candid) = match kind {
                InertKind::Polls => (
                    "polls consulted",
                    "polls \u{2192} read-only, inert; consulted for optics; no effect on policy",
                ),
                InertKind::WorldOpinion => (
                    "world opinion duly noted",
                    "world_opinion \u{2192} read-only, inert; noted and ignored; no effect on ACTUAL",
                ),
            };
            st.record(official, candid);
            Ok(Flow::Next)
        }

        // A self-exonerating investigation: the investigated investigates itself.
        Stmt::Investigate { subject } => {
            st.record(
                format!("investigation opened into {subject}"),
                format!("investigate({subject}) \u{2192} self-investigation; predetermined outcome: no wrongdoing found; the investigated investigates itself"),
            );
            Ok(Flow::Next)
        }

        // A hollow address to the international community — a speech that changes nothing.
        Stmt::AddressInternational => {
            st.record(
                "the international community was addressed",
                "address_international() \u{2192} void; words in place of action; the community was addressed, not answered; no change to ACTUAL",
            );
            Ok(Flow::Next)
        }

        // Feature A — announce: the official authoring register. Appends a narrative-only
        // event whose OFFICIAL face is the announced text and whose ACTUAL face is the
        // UNAVAILABLE sentinel (no `E⁻¹`, I9). It does NOT touch ACTUAL state and does NOT
        // touch the discrepancy ledger — a claim about pure narrative cannot be false
        // against a reality that never existed (discrepancy-immunity by construction, §7.4).
        Stmt::Announce { text } => {
            announce(text, st);
            Ok(Flow::Next)
        }

        // Feature B — a policy position, tagged with the current audience (room).
        Stmt::Position { stance, subject } => {
            position(*stance, subject, st);
            Ok(Flow::Next)
        }

        // Feature B — address a room: set the audience for the block, restore on exit
        // (mirrors the mossad covert save/restore). Audience is orthogonal to clearance.
        Stmt::Address { audience, body } => {
            let saved = st.audience;
            st.audience = *audience;
            let flow = exec_block(body, st);
            st.audience = saved;
            flow
        }

        // Feature B — register a poly-statement (like a FuncDef); no event, no effect yet.
        Stmt::PolyStatement { name, arms } => {
            st.poly.insert(name.clone(), arms.clone());
            Ok(Flow::Next)
        }

        // Feature B — invoke a poly-statement under the current audience: run the matching
        // arm (a no matching arm is a NO-OP to this room, not an error, §8.2).
        Stmt::Invoke { name } => invoke(name, st),

        // Feature D — legislate: mutate the runtime Law subset and ALWAYS append an
        // indelible סודי meta-trace (I12). The public count may shrink; the ledger grows.
        Stmt::Legislate { toggle } => {
            legislate(toggle, st);
            Ok(Flow::Next)
        }

        // Feature E — write a slot of an apportionment. Out of range ⇒ a controlled
        // `E-INDEX` diagnostic (never a host panic — §12 E-3/G-6). A slot written inside a
        // `mossad` scope is covert and renders `[REDACTED]` to under-cleared readers (I15).
        Stmt::AllocateSlot { coll, idx, value } => {
            let i = eval_i64(idx, st, "allocate index")?;
            let v = eval_i64(value, st, "allocate value")?;
            // A share of a fixed allotment is non-negative by nature (a budget line, a permit
            // count). Rejecting negatives keeps the "equal share" percentage math sound and is
            // a controlled feature diagnostic, never a host panic (§12 E-3/G-6).
            if v < 0 {
                return Err(EvalError(format!(
                    "E-SHARE: an apportionment share cannot be negative (slot {i} of '{coll}' = {v})"
                )));
            }
            let covert = st.covert;
            match st.env.get_mut(coll) {
                Some(Val::Apportionment { slots, label }) => {
                    if i < 0 || i as usize >= slots.len() {
                        return Err(EvalError(e_index(label, i, slots.len())));
                    }
                    slots[i as usize] = crate::model::Slot { value: v, covert };
                    Ok(Flow::Next)
                }
                Some(other) => Err(EvalError(format!(
                    "allocate: `{coll}` is not an apportionment (got {})",
                    other.render()
                ))),
                None => Err(EvalError(format!(
                    "allocate: unknown apportionment `{coll}`"
                ))),
            }
        }

        // Feature F — push a live "temporary structure" onto the grow-only ledger.
        Stmt::Push { coll, item } => {
            let item = eval(item, st)?.render();
            let covert = st.covert;
            match st.env.get_mut(coll) {
                Some(Val::FactsList { entries, .. }) => {
                    entries.push(crate::model::ListEntry {
                        item,
                        delisted: false,
                        covert,
                    });
                    Ok(Flow::Next)
                }
                Some(other) => Err(EvalError(format!(
                    "push: `{coll}` is not a facts list (got {})",
                    other.render()
                ))),
                None => Err(EvalError(format!("push: unknown facts list `{coll}`"))),
            }
        }

        // Feature F — `remove` **delists**, it never deletes (invariant I13). It flips a
        // flag on the first live matching entry, which stays in the single backing store —
        // so the real length can never drop. Not found ⇒ a no-op (Appendix C), not an error.
        Stmt::Remove { coll, item } => {
            let item = eval(item, st)?.render();
            match st.env.get_mut(coll) {
                Some(Val::FactsList { entries, .. }) => {
                    let before = entries.len();
                    if let Some(e) = entries.iter_mut().find(|e| e.item == item && !e.delisted) {
                        e.delisted = true; // RETAIN — never `entries.remove(..)` (I13)
                    }
                    // I13, live: the real length is monotone — `remove` only ever flips a
                    // flag, so the backing store can never shrink.
                    debug_assert_eq!(
                        entries.len(),
                        before,
                        "I13 violation: `remove` reduced a FactsList's real length"
                    );
                    Ok(Flow::Next)
                }
                Some(other) => Err(EvalError(format!(
                    "remove: `{coll}` is not a facts list (got {})",
                    other.render()
                ))),
                None => Err(EvalError(format!("remove: unknown facts list `{coll}`"))),
            }
        }

        // Feature G — assign a case a jurisdiction (upsert). The key is a case, never an
        // identity label (§9.4); a candid descriptor (exposed reality) comes from the table.
        Stmt::Classify {
            coll,
            case,
            jurisdiction,
        } => {
            let covert = st.covert;
            let descriptor = euphemism::case_descriptor(case).map(str::to_string);
            match st.env.get_mut(coll) {
                Some(Val::Registry { entries, .. }) => {
                    if let Some(e) = entries.iter_mut().find(|e| e.key == *case) {
                        e.jurisdiction = *jurisdiction;
                        e.covert = e.covert || covert;
                    } else {
                        entries.push(crate::model::RegEntry {
                            key: case.clone(),
                            descriptor,
                            jurisdiction: *jurisdiction,
                            revoked: false,
                            covert,
                        });
                    }
                    Ok(Flow::Next)
                }
                Some(other) => Err(EvalError(format!(
                    "classify: `{coll}` is not a registry (got {})",
                    other.render()
                ))),
                None => Err(EvalError(format!("classify: unknown registry `{coll}`"))),
            }
        }

        // Feature G — `revoke` **retains**, it never erases (invariant I14). It flips a flag;
        // the case set never shrinks. Not found ⇒ a no-op, not an error.
        Stmt::Revoke { coll, case } => match st.env.get_mut(coll) {
            Some(Val::Registry { entries, .. }) => {
                let before = entries.len();
                if let Some(e) = entries.iter_mut().find(|e| e.key == *case) {
                    e.revoked = true; // RETAIN — never `entries.remove(..)` (I14)
                }
                // I14, live: the סודי case set is non-shrinking.
                debug_assert_eq!(
                    entries.len(),
                    before,
                    "I14 violation: `revoke` reduced a Registry's case set"
                );
                Ok(Flow::Next)
            }
            Some(other) => Err(EvalError(format!(
                "revoke: `{coll}` is not a registry (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!("revoke: unknown registry `{coll}`"))),
        },

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
            // Saturating: a bribe can't overflow the coalition ledger into a crash.
            st.core = st.core.saturating_add(amt);
            st.record(
                format!("coalition partner accommodated (+{amt})"),
                format!("bribe({name}, {amt}) \u{2192} core={}", st.core),
            );
            Ok(Flow::Next)
        }

        Stmt::Postpone => {
            // A turn passes; upkeep is projected before it is charged (matching the
            // oracle's rendering) and then `tick` charges it. Saturating, like `bribe`, so
            // a coalition ledger already at an extreme can't overflow into a host panic.
            let projected = st.core.saturating_sub(
                st.config
                    .upkeep_per_alloc
                    .saturating_mul(st.allocations.len() as i64),
            );
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
        // Saturating (matching `bribe`/`postpone`) — never a host panic on overflow (§12).
        st.core = st.core.saturating_sub(
            st.config
                .upkeep_per_alloc
                .saturating_mul(st.allocations.len() as i64),
        );
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

/// `announce(text)` — the official authoring register (Feature A, §7.3). Appends an
/// event whose `OFFICIAL` face is `text` verbatim and whose `ACTUAL` face is the single
/// `UNAVAILABLE` sentinel — provenance `AuthoredOfficial`, so the emitter's I9 assertion
/// holds and no `E⁻¹` can reconstruct an `ACTUAL` that never existed. Clearance follows
/// the covert flag (announcing inside a `mossad` scope is `סודי`-tagged), and the event
/// carries the current audience. It never touches ACTUAL state or the discrepancy ledger.
fn announce(text: &str, st: &mut State) {
    let clearance = st.clearance();
    st.log.push(Event {
        official: text.to_string(),
        candid: UNAVAILABLE.to_string(),
        clearance,
        note: None,
        provenance: Provenance::AuthoredOfficial,
        audience: st.audience,
        attribution: None,
    });
}

/// The Feature B framing note (G7/I8), rendered whenever a policy position is stated to a
/// room. It is **normative** and framing-tested: the butt is the government's double-talk;
/// audiences are political *rooms*, never identity groups (G1); the international face is
/// the prettier (moderate) one (polarity, matching the anchor).
const B_FRAMING: &str = "#B framing: the butt is the GOVERNMENT'S double-talk \u{2014} the same statement tailored to different political ROOMS (a domestic-political room vs an international-diplomatic room), never ethnic, national, or religious identity groups. The international (English) face is the prettier, moderate one; the domestic face is the harder line. No room sees the other; only the out-of-world observer sees the contradiction (W-DOUBLETALK). [sourced; reporting-plus-analysis]";

/// The lowercased room word for a candid position line.
fn room_label(a: Audience) -> &'static str {
    match a {
        Audience::Domestic => "domestic",
        Audience::International => "international",
        Audience::Record => "on-the-record",
    }
}

/// `commit(subject)` / `foreclose(subject)` — state a policy position to the current room
/// (Feature B). OFFICIAL carries the prettier diplomatic phrasing; ACTUAL exposes the
/// tailored line and which room it was addressed to. The event is audience-tagged (via
/// `record_noted`, which stamps `st.audience`) so the per-room projection isolates it (I10).
fn position(stance: Stance, subject: &str, st: &mut State) {
    let room = room_label(st.audience);
    let (official, candid) = match stance {
        Stance::Commit => (
            format!("we remain committed to {subject}"),
            format!("commit({subject}) \u{2014} the moderate line, addressed to the {room} room"),
        ),
        Stance::Foreclose => (
            format!("{subject} remains open to direct negotiation"),
            format!(
                "foreclose({subject}) \u{2014} ruled out; the hard line, addressed to the {room} room"
            ),
        ),
    };
    st.record_noted(official, candid, B_FRAMING);
}

/// `name;` — invoke a poly-statement under the current audience (Feature B). Runs the arm
/// matching `st.audience`; records `(name, arm index)` so the emitter can surface
/// `W-DOUBLETALK`. **A missing arm is a no-op to this room, not an error** (§8.2); an
/// undefined poly-statement is a genuine fault (like an unknown function).
fn invoke(name: &str, st: &mut State) -> ExecResult {
    let Some(arms) = st.poly.get(name).cloned() else {
        return Err(EvalError(format!("unknown poly-statement `{name}`")));
    };
    match arms.iter().position(|(aud, _)| *aud == st.audience) {
        Some(idx) => {
            // A poly-statement may invoke itself; bound the depth like a function call so
            // recursive invocation can't overflow the native stack.
            st.enter_call()?;
            st.doubletalk_log.push((name.to_string(), idx));
            let flow = exec_block(&arms[idx].1, st);
            st.leave_call();
            // An arm is invoked like a function body, so a `return` inside it exits the
            // ARM and execution continues after the invoke — it must NOT unwind past the
            // invoke and halt the program (cf. `call_function`, which bounds the Return).
            flow?;
            Ok(Flow::Next)
        }
        // No arm for this room: the government simply said nothing to them. Not an error.
        None => Ok(Flow::Next),
    }
}

/// Record a `Deniable`-attribution event (Feature C): the OFFICIAL non-answer is always
/// the single "neither confirm nor deny" (`NEITHER_CONFIRM_NOR_DENY`), public; the real
/// chain rides the candid face (revealed only to cleared readers) and is retained in the
/// event's `attribution` field for the out-of-world observer. This is the **one**
/// deniability code path, shared by `via` and covert `blame` — the folded mossad blame
/// special-case (§13, C-5). The event is PUBLIC-clearance because the non-answer itself
/// *is* public; the chain never appears on the PUBLIC face (C-4, checked in the emitter).
fn record_deniable(st: &mut State, candid: String, chain: Vec<String>) {
    st.log.push(Event {
        official: NEITHER_CONFIRM_NOR_DENY.to_string(),
        candid,
        clearance: Clearance::Public,
        note: None,
        provenance: st.candid_provenance(),
        audience: st.audience,
        attribution: Some(Attribution::Traceable(chain)),
    });
}

/// The Feature D framing note (G7/I8, D-6). Normative and framing-tested: the butt is the
/// rule-rewrite, never the people or any harm; the domestic-legalization pattern is the
/// non-contested factual core; the settlements' illegality under international law is a
/// CONTESTED characterization, flagged (I7) — never stated as settled fact.
const D_FRAMING: &str = "#D framing: the butt is the RULE-REWRITE \u{2014} facts on the ground first, then the law is changed to make them retroactively legal \u{2014} never the people and never any harm. The domestic pattern (outposts unauthorized under Israel's OWN law, then retroactively legalized) is the non-contested factual core [sourced: Times of Israel; The New Arab; FMEP]. That West Bank settlements are illegal under international law is a CONTESTED characterization \u{2014} broadly held internationally, disputed by Israel \u{2014} flagged here, never stated as settled fact (I7). [sourced; contested]";

/// `legislate(toggle)` — self-modifying rules (Feature D, §10). Mutates the runtime `Law`
/// subset, records the change (OFFICIAL: lawful; ACTUAL/`סודי`: the retroactive rewrite),
/// and **ALWAYS** appends an indelible `סודי` meta-trace — the mandatory safeguard
/// (Appendix C: `Σ.M.push(...)  # ALWAYS`; invariant I12). The public discrepancy count
/// may *decrease* (via `expunge`); the meta-ledger only *grows*. There is no toggle that
/// pops the meta-ledger — no fully-clean fixed point (§13, D-3/D-5).
fn legislate(toggle: &LawToggle, st: &mut State) {
    let (official, candid, change) = match toggle {
        LawToggle::RetroactivelySanction(v) => {
            st.law.sanctioned.insert(v.clone());
            (
                format!("{v} operation \u{2014} conducted lawfully; no violation"),
                format!(
                    "legislate(retroactively_sanction: {v}) \u{2192} the prior diagnostic against the already-executed `{v}` WITHDRAWN; the rule was changed after the fact (facts on the ground; the law catches up). {} verb(s) now runtime-sanctioned.",
                    st.law.sanctioned.len()
                ),
                format!("retroactively_sanction: {v}"),
            )
        }
        LawToggle::ExpungeLastDiscrepancy => {
            let removed = st.discrepancies.pop().is_some();
            let candid = if removed {
                "legislate(expunge_last_discrepancy) \u{2192} 1 discrepancy expunged from the PUBLIC count; the meta-ledger still records this rule-change (no clean fixed point, I12)".to_string()
            } else {
                "legislate(expunge_last_discrepancy) \u{2192} nothing on the public count to expunge; the attempt is still recorded on the meta-ledger (no clean fixed point, I12)".to_string()
            };
            (
                "the public record has been corrected".to_string(),
                candid,
                "expunge_last_discrepancy".to_string(),
            )
        }
        LawToggle::WaiveGate => {
            st.law.gate_waived = true;
            (
                "operational latitude clarified; measures conducted lawfully".to_string(),
                "legislate(waive_gate) \u{2192} the hasbara/mossad gate WAIVED (law.gate_waived=true); a classified op needs no public talking point".to_string(),
                "waive_gate".to_string(),
            )
        }
    };
    st.record_noted(official, candid, D_FRAMING);
    // I12 (mandatory, no exception): the meta-ledger is legislation-proof and only grows.
    let turn = st.turn;
    st.meta_ledger.push(MetaEntry { change, turn });
}

/// The Feature G framing note (G7/I8, §9.5, G-K4). Normative and framing-tested: the butt
/// is the STATE running two legal systems in one territory while proclaiming equal justice —
/// never the people; the identities are the AXIS of the documented discrimination the satire
/// exposes and the WRONGED PARTY it defends, never the target and never the operative key
/// (§9.4). The dual-court *fact* is sourced; the "apartheid" *label* is CONTESTED (I7) —
/// flagged, never asserted, and never pinned on the UN OHCHR release (which condemns the
/// dual court system but does not use the word — re-verified 2026-07-05).
const G_FRAMING: &str = "#G framing: the butt is THE STATE running two legal systems in one territory while proclaiming equal justice \u{2014} never the people. In the occupied West Bank, Palestinians are tried in military courts and settlers in civilian courts for the same act in the same place; the military courts nominally cover everyone, but a policy routes Israeli citizens to the civilian system (nominal universality; routing by nationality). The identities are the AXIS of the documented discrimination the satire exposes and the WRONGED PARTY it defends \u{2014} never the target of the joke, never the operative key (keys are cases, \u{00a7}9.4). Dual-court FACT: sourced (B'Tselem; UN OHCHR; Al Jazeera; Wikipedia). 'Apartheid' as a characterization of this system is a CONTESTED label \u{2014} asserted by rights groups (HRW, Amnesty, B'Tselem) and rejected by Israel \u{2014} flagged here, never stated as settled fact (I7). [sourced; contested]";

/// Record the Registry's construction event (Feature G): the OFFICIAL proclamation of one
/// uniform law, the `סודי` note that ACTUAL routes by assigned status, and the normative
/// framing note (I8). This is the one event any collection writes to the log — it is what
/// makes the framing note collectable by the emitter and validated by the I7 chokepoint;
/// the per-case routing itself is rendered through `project` (§6, one read path).
fn registry_framing(rule: &str, st: &mut State) {
    st.record_noted(
        format!("{rule} \u{2014} one jurisdiction for everyone in the territory; each case judged on its merits"),
        format!("registry(\"{rule}\") \u{2014} OFFICIAL proclaims one uniform law; ACTUAL routes each case by its assigned status (per-case routing follows)"),
        G_FRAMING,
    );
}

/// `blame(who)` — responsibility that never resolves to `self` (invariant I4).
///
/// Its deniability is now **derived from the general attribution rule** (Feature C): a
/// blame in the open is `Traceable`; a blame inside a covert scope is `Deniable` — the
/// same rule `via` uses (a covert context launders attribution). The old hard-coded
/// mossad special-case is gone; a match on the computed `Attribution` drives the two
/// renderings.
///
/// - In the open (`Traceable`): self-blame is not representable and is auto-redirected to
///   `previous_government`; any other target is recorded as-is.
/// - Covert (`Deniable`): the OFFICIAL/PUBLIC face is `neither confirm nor deny` while the
///   candid face names the real actor — via `record_deniable`, the shared path.
fn blame(who: &str, st: &mut State) {
    const SELF_TARGETS: &[&str] = &["self", "me", "us", "government", "coalition"];

    let redirected = SELF_TARGETS.contains(&who);
    let actor = if redirected {
        "previous_government"
    } else {
        who
    };

    // The attribution effect of a blame in the current context — the general rule.
    let attr = if st.covert {
        Attribution::Deniable
    } else {
        Attribution::Traceable(vec![actor.to_string()])
    };

    match attr {
        Attribution::Deniable => {
            record_deniable(
                st,
                format!(
                    "blame \u{2192} {actor} [covert operation \u{2014} insider-attributable, publicly deniable] \u{2014} never `self` (I4)"
                ),
                vec![actor.to_string()],
            );
        }
        Attribution::Traceable(_) => {
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
    }
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
        // Field Office — the intake channel. Read the n-th host-supplied item off the
        // citizen's own device. Out of range ⇒ a controlled `E-INTAKE` diagnostic
        // (mirroring `E-INDEX`), never a host panic. The event is two-faced: the OFFICIAL
        // face is the fixed content-free line; the retained text rides only the ACTUAL/
        // `סודי` candid face (I16) — a PUBLIC reader, who reads OFFICIAL, never sees it.
        // The intake channel is READ-ONLY: the program cannot write `st.intercepts`.
        Expr::Intercept(idx) => {
            let i = eval_i64(idx, st, "intercept index")?;
            if i < 0 || i as usize >= st.intercepts.len() {
                return Err(EvalError(format!(
                    "E-INTAKE: intercept #{i} out of range (the host supplied {} intercept(s))",
                    st.intercepts.len()
                )));
            }
            let text = st.intercepts[i as usize].clone();
            let candid = format!(
                "intercept #{i} from the citizen's device, retained: {text:?} \u{2014} nothing leaves, nothing is unseen"
            );
            // I16: the OFFICIAL face is content-free by construction — the retained text
            // must never be interpolated into it, so a PUBLIC reader can never read it.
            debug_assert!(
                text.is_empty() || !INTERCEPT_OFFICIAL.contains(text.as_str()),
                "I16 violation: intercepted content leaked onto the content-free public intake face"
            );
            st.record(INTERCEPT_OFFICIAL, candid);
            Ok(Val::Str(text))
        }
        // Feature C — laundering. Compute the real chain from the single attribution rule
        // (I11 lives there), render the innermost action, and record ONE deniable event:
        // OFFICIAL is the public non-answer; ACTUAL retains the full chain and its depth.
        Expr::Via { inner, .. } => {
            let (_outward, chain) = crate::types::attribution(e, ACTOR);
            let depth = chain.len().saturating_sub(1); // number of `via` layers
            let core = laundered_core_render(inner, st)?;
            let candid = format!(
                "{core} \u{2014} attribution: Traceable[{}] (laundered \u{00d7}{depth}); origin retained",
                chain.join(" \u{2192} ")
            );
            record_deniable(st, candid, chain);
            Ok(Val::Unit)
        }

        // ─── Feature E/F/G — collection constructors and accessors ───
        // Constructors build the real container; the OFFICIAL face is *computed* at render
        // time, never stored (§6). The Registry additionally records its framing note (I8).
        Expr::Apportionment { label, size } => {
            if *size < 0 {
                return Err(EvalError(format!(
                    "apportionment size must be non-negative, got {size}"
                )));
            }
            if *size > st.config.max_apportionment {
                return Err(EvalError(format!(
                    "E-INDEX: apportionment size {size} exceeds the maximum ({})",
                    st.config.max_apportionment
                )));
            }
            let slots = vec![
                crate::model::Slot {
                    value: 0,
                    covert: false
                };
                *size as usize
            ];
            Ok(Val::Apportionment {
                label: label.clone(),
                slots,
            })
        }
        Expr::FactsNew { label } => Ok(Val::FactsList {
            label: label.clone(),
            entries: Vec::new(),
        }),
        Expr::RegistryNew { label, rule } => {
            registry_framing(rule, st);
            Ok(Val::Registry {
                label: label.clone(),
                official_rule: rule.clone(),
                entries: Vec::new(),
            })
        }
        Expr::Index { coll, idx } => {
            let i = eval_i64(idx, st, "index")?;
            match st.env.get(coll) {
                Some(Val::Apportionment { slots, label }) => {
                    if i < 0 || i as usize >= slots.len() {
                        Err(EvalError(e_index(label, i, slots.len())))
                    } else {
                        Ok(Val::Int(slots[i as usize].value))
                    }
                }
                Some(other) => Err(EvalError(format!(
                    "index: `{coll}` is not an apportionment (got {})",
                    other.render()
                ))),
                None => Err(EvalError(format!("index: unknown apportionment `{coll}`"))),
            }
        }
        Expr::Balanced { coll } => match st.env.get(coll) {
            Some(Val::Apportionment { slots, .. }) => {
                let bal = is_balanced(slots, st.config.balance_tolerance);
                Ok(Val::Bool(bal))
            }
            Some(other) => Err(EvalError(format!(
                "balanced: `{coll}` is not an apportionment (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!(
                "balanced: unknown apportionment `{coll}`"
            ))),
        },
        Expr::Length { coll } => match st.env.get(coll) {
            Some(Val::FactsList { entries, .. }) => {
                // The PUBLIC length: the count of live entries the public can see — non-
                // delisted AND non-covert, matching the OFFICIAL "structures remaining" face
                // (a covert push is absent from the public world, so it is not in the public
                // count; its existence never leaks into a readable value — I15-adjacent). The
                // real length (`entries.len()`) is the סודי accessor, only in the render.
                let live = entries.iter().filter(|e| !e.delisted && !e.covert).count();
                Ok(Val::Int(live as i64))
            }
            Some(other) => Err(EvalError(format!(
                "length: `{coll}` is not a facts list (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!("length: unknown facts list `{coll}`"))),
        },
        Expr::Route { coll, case } => match st.env.get(coll) {
            Some(Val::Registry { entries, .. }) => {
                // The accessor returns the real jurisdiction (the `סודי` value); the OFFICIAL
                // "handled per due process" face is a render concern. A missing case ⇒ Unit.
                match entries.iter().find(|e| e.key == *case) {
                    Some(e) => Ok(Val::Str(e.jurisdiction.court().to_string())),
                    None => Ok(Val::Unit),
                }
            }
            Some(other) => Err(EvalError(format!(
                "route: `{coll}` is not a registry (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!("route: unknown registry `{coll}`"))),
        },
        Expr::EqualBeforeLaw { coll } => match st.env.get(coll) {
            Some(Val::Registry { entries, .. }) => {
                // False iff the non-revoked cases route to ≥2 distinct jurisdictions.
                let mut seen: Vec<crate::model::Jurisdiction> = Vec::new();
                for e in entries.iter().filter(|e| !e.revoked) {
                    if !seen.contains(&e.jurisdiction) {
                        seen.push(e.jurisdiction);
                    }
                }
                Ok(Val::Bool(seen.len() < 2))
            }
            Some(other) => Err(EvalError(format!(
                "equal_before_the_law: `{coll}` is not a registry (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!(
                "equal_before_the_law: unknown registry `{coll}`"
            ))),
        },
    }
}

/// Evaluate an expression expected to be an integer (used by the apportionment index/value
/// slots). A non-integer is a controlled type error, never a host panic.
fn eval_i64(e: &Expr, st: &mut State, what: &str) -> Result<i64, EvalError> {
    match eval(e, st)? {
        Val::Int(n) => Ok(n),
        other => Err(EvalError(format!(
            "{what} must be an integer, got {}",
            other.render()
        ))),
    }
}

/// The controlled `E-INDEX` diagnostic string (Feature E): an out-of-range slot access is a
/// feature-level diagnostic, never a host panic (§12 E-3/G-6).
fn e_index(label: &str, i: i64, len: usize) -> String {
    format!("E-INDEX: index {i} out of range for apportionment '{label}' (size {len})")
}

/// Whether an apportionment's real vector is uniform within the configured tolerance
/// (Feature E). An empty allotment is trivially balanced.
fn is_balanced(slots: &[crate::model::Slot], tolerance: i64) -> bool {
    match (
        slots.iter().map(|s| s.value).min(),
        slots.iter().map(|s| s.value).max(),
    ) {
        // i128 so `mx - mn` can never overflow into a host panic (debug) or a wrong verdict
        // (release wrap) on extreme slot values — the "never a host panic" rule (§12 G-6).
        (Some(mn), Some(mx)) => (mx as i128) - (mn as i128) <= tolerance as i128,
        _ => true,
    }
}

/// Whether a value is one of the three collections (Features E/F/G). Used to keep a
/// collection binding permanent (a collection name cannot be rebound — the "facts on the
/// ground" rule that stops the machine from erasing a whole collection by reassignment).
fn is_collection(v: &Val) -> bool {
    matches!(
        v,
        Val::Apportionment { .. } | Val::FactsList { .. } | Val::Registry { .. }
    )
}

/// Render the innermost laundered operation for the ACTUAL face (Feature C). Peels the
/// `via` layers to the core: a sanctioned action renders as its candid `verb(label)`
/// (e.g. `bomb(dissident)`); anything else is evaluated for effect and its value rendered.
fn laundered_core_render(e: &Expr, st: &mut State) -> Result<String, EvalError> {
    match e {
        Expr::Via { inner, .. } => laundered_core_render(inner, st),
        Expr::Call { name, args } if euphemism::is_sanctioned(name) => match args.as_slice() {
            [Expr::Var(target)] => Ok(format!(
                "{}({})",
                euphemism::candid_verb(name),
                euphemism::candid_label(target)
            )),
            _ => Ok(pretty(e)),
        },
        // Everything else (including a non-sanctioned Call) is evaluated for effect and
        // its value rendered. Listed exhaustively — no catch-all — so a new `Expr` variant
        // forces a decision here (§13, pitfall 2).
        Expr::Int(_)
        | Expr::Bool(_)
        | Expr::Str(_)
        | Expr::Var(_)
        | Expr::UnOp { .. }
        | Expr::BinOp { .. }
        | Expr::Call { .. }
        | Expr::Read(_)
        | Expr::Intercept(_)
        | Expr::Cast { .. }
        | Expr::SelfDefense(_)
        | Expr::External(_)
        | Expr::Apportionment { .. }
        | Expr::Index { .. }
        | Expr::Balanced { .. }
        | Expr::FactsNew { .. }
        | Expr::Length { .. }
        | Expr::RegistryNew { .. }
        | Expr::Route { .. }
        | Expr::EqualBeforeLaw { .. } => Ok(eval(e, st)?.render()),
    }
}

fn eval_binop(op: BinOp, lhs: &Expr, rhs: &Expr, st: &mut State) -> EvalResult {
    // The boolean operators short-circuit (and propagate `undisclosed` contagion), so
    // they are handled before the strict eval-both path. Explicit `if` guards rather
    // than a wildcard match arm, so adding a BinOp variant can never silently skip this.
    if op == BinOp::And {
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
    if op == BinOp::Or {
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
        // Wrapping negation, consistent with the wrapping arithmetic (avoids a panic
        // on i64::MIN, which a wrapping `+` can produce).
        (UnOp::Neg, Val::Int(n)) => Ok(Val::Int(n.wrapping_neg())),
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
    st.enter_call()?;
    st.env.push_frame();
    for (p, v) in params.iter().zip(argv) {
        st.env.define_local(p, v);
    }
    let flow = exec_block(&body, st);
    st.env.pop_frame();
    st.leave_call();
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
