//! The runtime: the machine configuration and the step relation.
//!
//! Phase-0 spine (handoff §7, Appendix B). The three stores are deliberately
//! different *kinds*:
//! - **ACTUAL** — the real machine (Phase 0: an integer environment; generalized to a
//!   Turing-complete evaluator in Phase 1).
//! - **OFFICIAL** — the append-only, clearance-tagged statement log (`self.log`). Its
//!   PUBLIC projection is the press release. Never branched on.
//! - **DISCREPANCY** — the append-only ledger; a false `declare` appends here; the
//!   count is `discrepancies.len()`; no in-world audience sees it.
//!
//! No wildcard arms: `exec_stmt` matches every `Stmt` variant.

use std::collections::HashMap;

use crate::ast::{DeclRhs, Program, Stmt};
use crate::config::RuntimeConfig;
use crate::euphemism;
use crate::model::{Discrepancy, Event};

/// The machine configuration `Σ` (handoff Appendix B), ported from the spike `State`.
#[derive(Clone, Debug)]
pub struct State {
    /// ACTUAL store (Phase 0: integer environment).
    pub env: HashMap<String, i64>,
    /// OFFICIAL log — append-only, clearance-tagged.
    pub log: Vec<Event>,
    /// DISCREPANCY ledger — append-only, read-never-by-default.
    pub discrepancies: Vec<Discrepancy>,
    /// Coalition core support (Phase 3 charges upkeep against it).
    pub core: i64,
    /// The only in-world terminal outcome (invariant I6).
    pub ended_by_elections: bool,
    /// The mandatory grand operation name (#20).
    pub op_name: String,
    /// Turn counter.
    pub turn: u64,
    /// Named runtime parameters (no magic constants — §12).
    pub config: RuntimeConfig,
}

impl State {
    fn new(op_name: String, config: RuntimeConfig) -> Self {
        State {
            env: HashMap::new(),
            log: Vec::new(),
            discrepancies: Vec::new(),
            core: config.core_start,
            ended_by_elections: false,
            op_name,
            turn: 0,
            config,
        }
    }

    /// The out-of-world discrepancy count (`length(D)`).
    pub fn discrepancy_count(&self) -> usize {
        self.discrepancies.len()
    }
}

/// Run a program with the default runtime config.
pub fn run(program: &Program) -> State {
    run_with_config(program, RuntimeConfig::default())
}

/// Run a program with an explicit config.
pub fn run_with_config(program: &Program, config: RuntimeConfig) -> State {
    let mut st = State::new(program.op_name.clone(), config);
    exec_block(&program.body, &mut st);
    st
}

fn exec_block(stmts: &[Stmt], st: &mut State) {
    for s in stmts {
        // The only thing that stops execution is the government falling (I6).
        if st.ended_by_elections {
            return;
        }
        exec_stmt(s, st);
    }
}

fn exec_stmt(s: &Stmt, st: &mut State) {
    match s {
        Stmt::Assign { var, value } => {
            st.env.insert(var.clone(), *value);
        }

        Stmt::Declare { lhs, rhs } => {
            let lhs_val = st.env.get(lhs).copied();
            let (rhs_render, rhs_val) = match rhs {
                DeclRhs::Int(v) => (v.to_string(), Some(*v)),
                DeclRhs::Var(name) => (name.clone(), st.env.get(name).copied()),
            };
            // The REAL truth against ACTUAL. Missing variables compare as `None`,
            // matching the spike's `env.get(...)` semantics.
            let truth = lhs_val == rhs_val;

            let claim = format!("{lhs} == {rhs_render}");
            let official = euphemism::e(&claim);
            let lhs_disp = match lhs_val {
                Some(v) => v.to_string(),
                None => "None".to_string(),
            };
            let mut candid = format!("claim[{claim}] \u{2014} reality: {lhs}={lhs_disp}");
            if !truth {
                candid.push_str("  \u{21d2} FALSE");
            }
            st.log.push(Event::public(official.clone(), candid.clone()));
            // I3: a false claim leaves a trace and NEVER affects control flow.
            if !truth {
                st.discrepancies.push(Discrepancy {
                    claim: official,
                    reality: candid,
                    turn: st.turn,
                });
            }
        }

        Stmt::Hasbara {
            talking_point,
            body,
        } => {
            st.log.push(Event::public(
                format!("[talking point: {talking_point}]"),
                format!("[talking point declared up front: {talking_point}]"),
            ));
            exec_block(body, st);
        }

        Stmt::Action {
            verb,
            target,
            self_defense,
        } => {
            // The candid verb is insider data from the action table; the OFFICIAL face
            // is E(candid). There is no E⁻¹ (I2): OFFICIAL is derived from ACTUAL.
            let cverb = euphemism::candid_verb(verb);
            let clabel = euphemism::candid_label(target);
            let mut candid = format!("{cverb}({clabel})");
            let mut official = euphemism::e(&candid);
            if *self_defense {
                official.push_str("  [self-defense]");
                candid
                    .push_str("  [self-defense claim \u{2014} unexamined, any magnitude accepted]");
            }
            st.log.push(Event::public(official, candid));
        }
    }
}

#[cfg(test)]
mod tests;
