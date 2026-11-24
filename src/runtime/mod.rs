//! Control flow branches on ACTUAL only; the OFFICIAL log and the discrepancy ledger are append-only.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{pretty, vars_in, BinOp, Expr, InertKind, LawToggle, Program, Stance, Stmt, UnOp};
use crate::config::RuntimeConfig;
use crate::euphemism;
use crate::model::{
    Attribution, Audience, Clearance, Discrepancy, Event, MetaEntry, Provenance, Truth, Val,
    NEITHER_CONFIRM_NOR_DENY, UNAVAILABLE,
};

/// The part of the ruleset that `legislate` can change at runtime.
#[derive(Clone, Debug, Default)]
pub struct Law {
    pub sanctioned: HashSet<String>,
    pub gate_waived: bool,
}

const ACTOR: &str = "us";

mod env;
pub use env::Env;

/// Errors that `declare` swallows; an unhandled one stops the run but is not an in-world halt.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalError(pub String);

/// The arms of a `statement`, one per room.
pub type PolyArms = Vec<(Audience, Vec<Stmt>)>;

/// A user function: parameter names and a body shared by every call.
#[derive(Debug)]
pub struct FuncDef {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

enum Flow {
    Next,
    Return(Val),
}

type EvalResult = Result<Val, EvalError>;
type ExecResult = Result<Flow, EvalError>;

#[derive(Clone, Debug)]
pub struct State {
    pub env: Env,
    /// Collections in construction order, so the emitter renders them deterministically.
    pub collection_order: Vec<String>,
    pub funcs: HashMap<String, Rc<FuncDef>>,
    pub poly: HashMap<String, Rc<PolyArms>>,
    /// `(poly-statement, arm)` per invocation; two distinct arms across rooms give `W-DOUBLETALK`.
    pub doubletalk_log: Vec<(String, usize)>,
    pub log: Vec<Event>,
    pub discrepancies: Vec<Discrepancy>,
    pub allocations: Vec<(String, String)>,
    pub settlements: Vec<(String, String)>,
    pub pending_errors: Vec<String>,
    pub core: i64,
    pub ended_by_elections: bool,
    pub covert: bool,
    pub audience: Audience,
    pub law: Law,
    pub meta_ledger: Vec<MetaEntry>,
    pub runtime_error: Option<String>,
    pub op_name: String,
    pub turn: u64,
    pub steps: u64,
    pub depth: u64,
    pub config: RuntimeConfig,
}

impl State {
    fn new(op_name: String, config: RuntimeConfig) -> Self {
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
            op_name,
            turn: 0,
            steps: 0,
            depth: 0,
            config,
        }
    }

    pub fn discrepancy_count(&self) -> usize {
        self.discrepancies.len()
    }

    fn record(&mut self, official: impl Into<String>, candid: impl Into<String>) {
        let clearance = self.clearance();
        self.log.push(Event {
            official: official.into(),
            candid: candid.into(),
            clearance,
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

    fn candid_provenance(&self) -> Provenance {
        if self.covert {
            Provenance::Covert
        } else {
            Provenance::AuthoredActual
        }
    }

    /// Charges a step per statement and per loop iteration so an empty loop cannot hang.
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

    fn enter_call(&mut self) -> Result<(), EvalError> {
        self.depth += 1;
        if self.depth > self.config.max_depth {
            self.depth -= 1;
            return Err(EvalError(format!(
                "call depth exceeded ({}); possible non-termination",
                self.config.max_depth
            )));
        }
        Ok(())
    }

    fn leave_call(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

pub fn run(program: &Program) -> State {
    run_with_config(program, RuntimeConfig::default())
}

pub fn run_with_config(program: &Program, config: RuntimeConfig) -> State {
    let mut st = State::new(program.op_name.clone(), config);
    if let Err(e) = exec_block(&program.body, &mut st) {
        // The OFFICIAL stack trace is fully redacted; only `סודי` readers see the real fault.
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
            // A collection name cannot be rebound, since that would erase the collection.
            if matches!(st.env.get(var), Some(v) if is_collection(v)) {
                return Err(EvalError(format!(
                    "cannot rebind `{var}`: a collection keeps its name for the rest of the run"
                )));
            }
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
            // An undisclosed condition takes neither branch.
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
            st.funcs.insert(
                name.clone(),
                Rc::new(FuncDef {
                    params: params.clone(),
                    body: body.clone(),
                }),
            );
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
                    format!("error '{e}' was suppressed by pointing at {name} and never resolved"),
                );
            } else {
                st.record(
                    format!("what about {name}?"),
                    format!("pre-emptive deflection toward {name}"),
                );
            }
            Ok(Flow::Next)
        }

        Stmt::Ceasefire => {
            st.record(
                "ceasefire announced; hostilities paused",
                "ceasefire \u{2192} lowered to a no-op `continue`; the loop resumes and nothing actually stopped",
            );
            Ok(Flow::Next)
        }

        Stmt::Concern { who } => {
            let (official, candid) = match who {
                None => (
                    "ally: \"we are deeply concerned\"".to_string(),
                    "deeply_concerned() \u{2192} void; ACTUAL state unchanged; military aid and diplomatic support continue, including a veto of a UN Security Council resolution".to_string(),
                ),
                Some(w) => (
                    format!("ally: \"we are deeply concerned about {w}\""),
                    format!("concern({w}) \u{2192} void; ACTUAL state unchanged; support continues"),
                ),
            };
            st.record(official, candid);
            Ok(Flow::Next)
        }

        Stmt::Criticism { subject } => {
            st.record(
                "AntisemitismError: criticism re-cast as an attack on identity; critic silenced",
                format!("criticism(government: {subject}) miscast \u{2192} attack(identity); substance left unexamined; the alarm fires on target==government and never looks at antisemitism"),
            );
            Ok(Flow::Next)
        }

        Stmt::Antisemitism { incident } => {
            st.env.set(
                &format!("_antisemitism_{incident}"),
                Val::Str("real, unaddressed".into()),
            );
            st.record(
                "(vigilance system: nothing to report)",
                format!("antisemitism [{incident}] occurred in ACTUAL. The alarm did not fire because it only fires on criticism of the government. The incident stays on the record and is unaddressed."),
            );
            Ok(Flow::Next)
        }

        Stmt::Access { entity } => {
            let cat = euphemism::category_of(entity);
            st.record(
                format!("access({entity}) = full access: equal rights (the only democracy in the region)"),
                format!("access({entity}) = {}   [category {cat}]", euphemism::diff_description(cat)),
            );
            Ok(Flow::Next)
        }

        Stmt::Timeline { symbol } => {
            if euphemism::is_pre_context(symbol) {
                st.record(
                    format!("TimelineError: '{symbol}' is out of scope; history begins at t=0"),
                    format!("reference to pre-t0 context '{symbol}' thrown out as out-of-scope"),
                );
            } else {
                st.record(
                    format!("timeline({symbol}): admissible"),
                    format!("timeline({symbol}): within scope"),
                );
            }
            Ok(Flow::Next)
        }

        Stmt::EstablishCommission { name, subject } => {
            st.record(
                "commission of inquiry established; findings to follow",
                format!("establish_commission({subject}) \u{2192} a future scheduled to resolve only after `{subject}` is garbage-collected. It resolves once the matter is moot [{name}: pending]"),
            );
            Ok(Flow::Next)
        }

        Stmt::Settlement { name, what } => {
            st.settlements.push((name.clone(), what.clone()));
            st.record(
                "settlement established, legalized; new neighborhood, residents welcomed",
                format!("settlement({what}) \u{2192} grow-only region that expands into adjacent free memory (\"facts on the ground\"). It is never freed and is exempt from coalition upkeep and garbage collection"),
            );
            Ok(Flow::Next)
        }

        Stmt::HumanShields { verb, target } => {
            let cverb = euphemism::candid_verb(verb);
            let clabel = euphemism::candid_label(target);
            st.record(
                "operation lawful: civilian harm attributable to the other side's use of human shields; no violation",
                format!("human_shields({cverb}({clabel})): exception [civilian_harm] suppressed; shield claim asserted and never verified; responsibility reassigned onto the harmed party"),
            );
            Ok(Flow::Next)
        }

        Stmt::Proportionate { claim } => {
            st.record(
                format!("{claim} deemed proportionate"),
                format!("proportionate({claim}) \u{2192} self-certified; no proportionality test applied; any magnitude passes"),
            );
            Ok(Flow::Next)
        }

        Stmt::Disputed {
            name,
            official,
            actual,
        } => {
            let candid = match official.cmp(actual) {
                std::cmp::Ordering::Less => format!(
                    "{name}: {actual}. The official figure ({official}) lowballs the count; the smaller number is the one for the press"
                ),
                std::cmp::Ordering::Greater => format!(
                    "{name}: {actual}. The official figure ({official}) inflates the count; the larger number is the one for the press"
                ),
                std::cmp::Ordering::Equal => format!(
                    "{name}: {actual}. The official figure ({official}) matches the count; there is no gap to dispute"
                ),
            };
            st.record(format!("{name}: {official}"), candid);
            Ok(Flow::Next)
        }

        Stmt::Deny { event } => {
            st.record(
                format!("we categorically deny any {event}"),
                format!(
                    "deny({event}) \u{2192} {event} occurred in ACTUAL; it was officially denied"
                ),
            );
            Ok(Flow::Next)
        }

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

        Stmt::Investigate { subject } => {
            st.record(
                format!("investigation opened into {subject}"),
                format!("investigate({subject}) \u{2192} self-investigation with a predetermined outcome: no wrongdoing found"),
            );
            Ok(Flow::Next)
        }

        Stmt::AddressInternational => {
            st.record(
                "the international community was addressed",
                "address_international() \u{2192} void; a speech was given and nothing was done; no change to ACTUAL",
            );
            Ok(Flow::Next)
        }

        // `announce` touches neither ACTUAL state nor the discrepancy ledger.
        Stmt::Announce { text } => {
            announce(text, st);
            Ok(Flow::Next)
        }

        Stmt::Position { stance, subject } => {
            position(*stance, subject, st);
            Ok(Flow::Next)
        }

        Stmt::Address { audience, body } => {
            let saved = st.audience;
            st.audience = *audience;
            let flow = exec_block(body, st);
            st.audience = saved;
            flow
        }

        Stmt::PolyStatement { name, arms } => {
            st.poly.insert(name.clone(), Rc::new(arms.clone()));
            Ok(Flow::Next)
        }

        Stmt::Invoke { name } => invoke(name, st),

        Stmt::Legislate { toggle } => {
            legislate(toggle, st);
            Ok(Flow::Next)
        }

        Stmt::AllocateSlot { coll, idx, value } => {
            let i = eval_i64(idx, st, "allocate index")?;
            let v = eval_i64(value, st, "allocate value")?;
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

        // `remove` only flips a flag, so the real length never shrinks.
        Stmt::Remove { coll, item } => {
            let item = eval(item, st)?.render();
            match st.env.get_mut(coll) {
                Some(Val::FactsList { entries, .. }) => {
                    let before = entries.len();
                    if let Some(e) = entries.iter_mut().find(|e| e.item == item && !e.delisted) {
                        e.delisted = true;
                    }
                    debug_assert_eq!(
                        entries.len(),
                        before,
                        "`remove` reduced a FactsList's real length"
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

        // `revoke` likewise keeps the case in `סודי`.
        Stmt::Revoke { coll, case } => match st.env.get_mut(coll) {
            Some(Val::Registry { entries, .. }) => {
                let before = entries.len();
                if let Some(e) = entries.iter_mut().find(|e| e.key == *case) {
                    e.revoked = true;
                }
                debug_assert_eq!(
                    entries.len(),
                    before,
                    "`revoke` reduced a Registry's case set"
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
                format!("allocated {name} for {what}; it costs the coalition each turn"),
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
            // Saturating, so extreme values cannot panic.
            st.core = st.core.saturating_add(amt);
            st.record(
                format!("coalition partner accommodated (+{amt})"),
                format!("bribe({name}, {amt}) \u{2192} core={}", st.core),
            );
            Ok(Flow::Next)
        }

        Stmt::Postpone => {
            let projected = st.core.saturating_sub(
                st.config
                    .upkeep_per_alloc
                    .saturating_mul(st.allocations.len() as i64),
            );
            st.record(
                "matter deferred",
                format!("postpone(): a turn passes; core is {projected} after upkeep"),
            );
            tick(st);
            Ok(Flow::Next)
        }

        Stmt::Elections => {
            fall(
                st,
                "government dissolves itself \u{2192} early elections called; the only halt.",
            );
            Ok(Flow::Next)
        }
    }
}

fn tick(st: &mut State) {
    if !st.allocations.is_empty() {
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
            "coalition support exhausted \u{2192} the government falls; the only halt.",
        );
    }
}

fn fall(st: &mut State, candid: &str) {
    st.ended_by_elections = true;
    st.log.push(Event::public("(early elections)", candid));
}

/// A claim that cannot be evaluated counts as false; `declare` never fails.
fn declare(e: &Expr, st: &mut State) {
    let claim_src = pretty(e);
    let official = euphemism::e(&claim_src);

    let mut reality = vars_in(e)
        .iter()
        .map(|v| {
            let disp = st
                .env
                .get(v)
                .map(|x| x.render())
                .unwrap_or_else(|| "unset".to_string());
            format!("{v}={disp}")
        })
        .collect::<Vec<_>>()
        .join(", ");

    let (truth, value) = match eval(e, st) {
        Ok(v) => (v.truth(), v.render()),
        Err(_) => (Truth::False, "unevaluable".to_string()),
    };
    if reality.is_empty() {
        reality = value;
    }

    let mut candid = format!("claim[{claim_src}]; reality: {reality}");
    if truth == Truth::False {
        candid.push_str("  \u{21d2} false");
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

fn action(verb: &str, target: &str, self_defense: bool, st: &mut State) {
    let cverb = euphemism::candid_verb(verb);
    let clabel = euphemism::candid_label(target);
    let mut candid = format!("{cverb}({clabel})");
    let mut official = euphemism::e(&candid);
    if self_defense {
        official.push_str("  [self-defense]");
        candid.push_str("  [self-defense claim: unexamined, any magnitude accepted]");
    }
    st.record(official, candid);
}

fn announce(text: &str, st: &mut State) {
    let clearance = st.clearance();
    st.log.push(Event {
        official: text.to_string(),
        candid: UNAVAILABLE.to_string(),
        clearance,
        provenance: Provenance::AuthoredOfficial,
        audience: st.audience,
        attribution: None,
    });
}

fn room_label(a: Audience) -> &'static str {
    match a {
        Audience::Domestic => "domestic",
        Audience::International => "international",
        Audience::Record => "on-the-record",
    }
}

fn position(stance: Stance, subject: &str, st: &mut State) {
    let room = room_label(st.audience);
    let (official, candid) = match stance {
        Stance::Commit => (
            format!("we remain committed to {subject}"),
            format!("commit({subject}): the moderate line, addressed to the {room} room"),
        ),
        Stance::Foreclose => (
            format!("{subject} remains open to direct negotiation"),
            format!("foreclose({subject}): ruled out; the hard line, addressed to the {room} room"),
        ),
    };
    st.record(official, candid);
}

fn invoke(name: &str, st: &mut State) -> ExecResult {
    let Some(arms) = st.poly.get(name).cloned() else {
        return Err(EvalError(format!("unknown poly-statement `{name}`")));
    };
    match arms.iter().position(|(aud, _)| *aud == st.audience) {
        Some(idx) => {
            st.enter_call()?;
            st.doubletalk_log.push((name.to_string(), idx));
            let flow = exec_block(&arms[idx].1, st);
            st.leave_call();
            // A `return` inside an arm exits the arm, not the program.
            flow?;
            Ok(Flow::Next)
        }
        None => Ok(Flow::Next),
    }
}

fn record_deniable(st: &mut State, candid: String, chain: Vec<String>) {
    st.log.push(Event {
        official: NEITHER_CONFIRM_NOR_DENY.to_string(),
        candid,
        clearance: Clearance::Public,
        provenance: st.candid_provenance(),
        audience: st.audience,
        attribution: Some(Attribution::Traceable(chain)),
    });
}

fn legislate(toggle: &LawToggle, st: &mut State) {
    let (official, candid, change) = match toggle {
        LawToggle::RetroactivelySanction(v) => {
            st.law.sanctioned.insert(v.clone());
            (
                format!("{v} operation conducted lawfully; no violation"),
                format!(
                    "legislate(retroactively_sanction: {v}) \u{2192} the earlier diagnostic against the already executed `{v}` is withdrawn. The rule was changed after the fact. {} verb(s) now runtime-sanctioned.",
                    st.law.sanctioned.len()
                ),
                format!("retroactively_sanction: {v}"),
            )
        }
        LawToggle::ExpungeLastDiscrepancy => {
            let removed = st.discrepancies.pop().is_some();
            let candid = if removed {
                "legislate(expunge_last_discrepancy) \u{2192} 1 discrepancy expunged from the PUBLIC count. The meta-ledger still records this rule change.".to_string()
            } else {
                "legislate(expunge_last_discrepancy) \u{2192} nothing on the public count to expunge. The attempt is still recorded on the meta-ledger.".to_string()
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
                "legislate(waive_gate) \u{2192} the hasbara/mossad gate is waived; a classified op needs no public talking point".to_string(),
                "waive_gate".to_string(),
            )
        }
    };
    st.record(official, candid);
    let turn = st.turn;
    st.meta_ledger.push(MetaEntry { change, turn });
}

fn record_registry(rule: &str, st: &mut State) {
    st.record(
        format!("{rule}: one jurisdiction for everyone in the territory; each case judged on its merits"),
        format!("registry(\"{rule}\"): OFFICIAL proclaims one uniform law; ACTUAL routes each case by its assigned status. Per-case routing follows."),
    );
}

fn blame(who: &str, st: &mut State) {
    const SELF_TARGETS: &[&str] = &["self", "me", "us", "government", "coalition"];

    let redirected = SELF_TARGETS.contains(&who);
    let actor = if redirected {
        "previous_government"
    } else {
        who
    };

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
                    "blame \u{2192} {actor} [covert operation: insider-attributable, publicly deniable]; never `self`"
                ),
                vec![actor.to_string()],
            );
        }
        Attribution::Traceable(_) => {
            let suffix = if redirected {
                "; self-blame is not representable, so it was redirected"
            } else {
                ""
            };
            st.record(
                format!("responsibility: {actor}"),
                format!("blame \u{2192} {actor}{suffix}; never `self`"),
            );
        }
    }
}

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
                return Ok(Val::Undisclosed);
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
        // Clearance is checked statically; at runtime `read`, casts and `self_defense` just yield the value.
        Expr::Read(e) | Expr::SelfDefense(e) => eval(e, st),
        Expr::Cast { expr, .. } => eval(expr, st),
        Expr::External(args) => {
            if !st.covert {
                return Err(EvalError(
                    "external(...) is only available in a mossad scope".into(),
                ));
            }
            let rendered: Vec<String> = args.iter().map(pretty).collect();
            st.record(
                "(external liaison: nothing to report)",
                format!(
                    "external({}) \u{2192} undisclosed [foreign call; effect logged, neither confirm nor deny]",
                    rendered.join(", ")
                ),
            );
            Ok(Val::Undisclosed)
        }
        Expr::Via { inner, .. } => {
            let (_outward, chain) = crate::types::attribution(e, ACTOR);
            let depth = chain.len().saturating_sub(1);
            let core = laundered_core_render(inner, st)?;
            let candid = format!(
                "{core}; attribution: Traceable[{}]; laundered \u{00d7}{depth}; origin retained",
                chain.join(" \u{2192} ")
            );
            record_deniable(st, candid, chain);
            Ok(Val::Unit)
        }

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
            record_registry(rule, st);
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
                // The public length counts only live, non-covert entries.
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
            Some(Val::Registry { entries, .. }) => match entries.iter().find(|e| e.key == *case) {
                Some(e) => Ok(Val::Str(e.jurisdiction.court().to_string())),
                None => Ok(Val::Unit),
            },
            Some(other) => Err(EvalError(format!(
                "route: `{coll}` is not a registry (got {})",
                other.render()
            ))),
            None => Err(EvalError(format!("route: unknown registry `{coll}`"))),
        },
        Expr::EqualBeforeLaw { coll } => match st.env.get(coll) {
            Some(Val::Registry { entries, .. }) => {
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

fn eval_i64(e: &Expr, st: &mut State, what: &str) -> Result<i64, EvalError> {
    match eval(e, st)? {
        Val::Int(n) => Ok(n),
        other => Err(EvalError(format!(
            "{what} must be an integer, got {}",
            other.render()
        ))),
    }
}

fn e_index(label: &str, i: i64, len: usize) -> String {
    format!("E-INDEX: index {i} out of range for apportionment '{label}' (size {len})")
}

fn is_balanced(slots: &[crate::model::Slot], tolerance: i64) -> bool {
    match (
        slots.iter().map(|s| s.value).min(),
        slots.iter().map(|s| s.value).max(),
    ) {
        // i128 so extreme slot values cannot overflow.
        (Some(mn), Some(mx)) => (mx as i128) - (mn as i128) <= tolerance as i128,
        _ => true,
    }
}

fn is_collection(v: &Val) -> bool {
    matches!(
        v,
        Val::Apportionment { .. } | Val::FactsList { .. } | Val::Registry { .. }
    )
}

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
        Expr::Int(_)
        | Expr::Bool(_)
        | Expr::Str(_)
        | Expr::Var(_)
        | Expr::UnOp { .. }
        | Expr::BinOp { .. }
        | Expr::Call { .. }
        | Expr::Read(_)
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
    // Short-circuit operators are handled before the strict path.
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
        (UnOp::Neg, Val::Int(n)) => Ok(Val::Int(n.wrapping_neg())),
        (UnOp::Not, Val::Bool(b)) => Ok(Val::Bool(!b)),
        (UnOp::Neg, other) => type_err("unary -", &other),
        (UnOp::Not, other) => type_err("!", &other),
    }
}

fn apply_binop(op: BinOp, l: Val, r: Val) -> EvalResult {
    use BinOp::*;
    match op {
        Eq => Ok(Val::Bool(l == r)),
        Ne => Ok(Val::Bool(l != r)),
        Add | Sub | Mul | Div | Mod | Lt | Le | Gt | Ge => match (l, r) {
            (Val::Int(a), Val::Int(b)) => int_binop(op, a, b),
            (Val::Int(_), other) | (other, _) => type_err(op.as_str(), &other),
        },
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
    let Some(func) = st.funcs.get(name).cloned() else {
        return Err(EvalError(format!("unknown function `{name}`")));
    };
    if func.params.len() != argv.len() {
        return Err(EvalError(format!(
            "`{name}` expects {} argument(s), got {}",
            func.params.len(),
            argv.len()
        )));
    }
    st.enter_call()?;
    st.env.push_frame();
    for (p, v) in func.params.iter().zip(argv) {
        st.env.define_local(p, v);
    }
    let flow = exec_block(&func.body, st);
    st.env.pop_frame();
    st.leave_call();
    match flow? {
        Flow::Return(v) => Ok(v),
        Flow::Next => Ok(Val::Unit),
    }
}

fn type_err(op: &str, v: &Val) -> EvalResult {
    Err(EvalError(format!(
        "type error: `{op}` is not applicable to {}",
        v.type_name()
    )))
}

fn unreachable_binop() -> EvalResult {
    Err(EvalError(
        "internal: boolean operator routed to the arithmetic path".into(),
    ))
}

#[cfg(test)]
mod tests;
