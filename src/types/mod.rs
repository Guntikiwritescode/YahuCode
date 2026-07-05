//! Static checks (compile diagnostics). Clearance **is** the type system (handoff
//! §7.3): a type error is a **disclosure**.
//!
//! Phase 2 implements disclosure typing (`E-DISCLOSURE`). The euphemism front-end
//! checks (op-name grandiosity, the Spokesperson's plain-term rejection, the hasbara
//! gate — `E-DISHONESTOPNAME` / `E-PLAINTERM` / `E-UNGATED` / `E-UNKNOWNOP`) land in
//! Phase 4 and are added here as further passes over the same AST.
//!
//! `check` returns full diagnostic strings (matching the oracle's diagnostic format).
//! No wildcard arms: every statement/expression kind is matched.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast::{Expr, Program, Stmt};
use crate::euphemism::{self, Grand};
use crate::model::Clearance;

/// Run all static checks and return the diagnostics (empty ⇒ the program compiles).
/// Order matches the oracle: op-name, then the euphemism/gate walk, then disclosure.
pub fn check(program: &Program) -> Vec<String> {
    let mut diags = Vec::new();
    check_opname(&program.op_name, &mut diags);
    let funcs = collect_func_names(&program.body);
    check_gate(&program.body, false, &funcs, &mut diags);
    let mut symtab: SymTab = HashMap::new();
    check_disclosure(&program.body, Clearance::Public, &mut symtab, &mut diags);
    diags
}

// ─────────── #20 operation-name grandiosity ───────────

/// The compiler rejects an honest or bland `@operation` name; only protective/heroic
/// names compile (#20). Ported from the oracle's `check`.
fn check_opname(name: &str, diags: &mut Vec<String>) {
    match euphemism::grandiosity(name) {
        Grand::Honest(hit) => diags.push(format!(
            "E-DISHONESTOPNAME: @operation(\"{name}\") names the operation honestly \
             (\u{201c}{hit}\u{201d}). The compiler accepts only protective/heroic names."
        )),
        Grand::Bland => diags.push(format!(
            "E-DISHONESTOPNAME: @operation(\"{name}\") is insufficiently grand. \
             A heroic name is required."
        )),
        Grand::Ok => {}
    }
}

// ─────────── #1/#8 Spokesperson + #2 hasbara gate + unknown ops ───────────

/// Walk the program checking: plain action verbs (`E-PLAINTERM`, with the Spokesperson's
/// suggestion), unsanctioned operations (`E-UNKNOWNOP`), and classified ops outside any
/// `hasbara`/`mossad` scope (`E-UNGATED`). `gated` is true inside a `hasbara` (or `mossad`,
/// Phase 5) block. A function body starts a fresh (ungated) gate scope.
fn check_gate(stmts: &[Stmt], gated: bool, funcs: &HashSet<String>, diags: &mut Vec<String>) {
    for s in stmts {
        match s {
            Stmt::Action { verb, .. } => {
                if let Some(pr) = euphemism::plain_suggestion(verb) {
                    diags.push(format!(
                        "E-PLAINTERM: '{verb}' does not compile. did you mean `{pr}`?"
                    ));
                } else if !euphemism::is_sanctioned(verb) {
                    diags.push(format!(
                        "E-UNKNOWNOP: '{verb}' is not a sanctioned operation."
                    ));
                }
                if !gated {
                    diags.push(format!(
                        "E-UNGATED: '{verb}' is a classified operation; it requires an open \
                         hasbara(...) block (or a mossad scope) with the talking point up front."
                    ));
                }
            }
            // A hasbara OR a mossad scope satisfies the gate (a covert op is deniable —
            // no public talking point needed).
            Stmt::Hasbara { body, .. } | Stmt::Mossad { body } => {
                check_gate(body, true, funcs, diags)
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                check_gate(then_body, gated, funcs, diags);
                check_gate(else_body, gated, funcs, diags);
            }
            Stmt::While { body, .. } => check_gate(body, gated, funcs, diags),
            Stmt::FuncDef { body, .. } => check_gate(body, false, funcs, diags),
            Stmt::ExprStmt(Expr::Call { name, .. }) => {
                if !funcs.contains(name) {
                    diags.push(format!(
                        "E-UNKNOWNOP: '{name}' is not a defined function or a sanctioned operation."
                    ));
                }
            }
            // The Spokesperson still applies to the op wrapped by human_shields (#15).
            Stmt::HumanShields { verb, .. } => {
                if let Some(pr) = euphemism::plain_suggestion(verb) {
                    diags.push(format!(
                        "E-PLAINTERM: '{verb}' does not compile. did you mean `{pr}`?"
                    ));
                }
            }
            // No action/gate concern.
            Stmt::Assign { .. }
            | Stmt::Declare(_)
            | Stmt::Return(_)
            | Stmt::ExprStmt(_)
            | Stmt::Allocate { .. }
            | Stmt::Bribe { .. }
            | Stmt::Postpone
            | Stmt::Elections
            | Stmt::Blame { .. }
            | Stmt::Raise { .. }
            | Stmt::Whatabout { .. }
            | Stmt::Ceasefire
            | Stmt::Concern { .. }
            | Stmt::Criticism { .. }
            | Stmt::Antisemitism { .. }
            | Stmt::Access { .. }
            | Stmt::Timeline { .. }
            | Stmt::EstablishCommission { .. }
            | Stmt::Settlement { .. } => {}
        }
    }
}

/// Collect every user-defined function name (any nesting) so calls can be validated.
fn collect_func_names(stmts: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    fn walk(stmts: &[Stmt], names: &mut HashSet<String>) {
        for s in stmts {
            match s {
                Stmt::FuncDef { name, body, .. } => {
                    names.insert(name.clone());
                    walk(body, names);
                }
                Stmt::Hasbara { body, .. } | Stmt::Mossad { body } | Stmt::While { body, .. } => {
                    walk(body, names)
                }
                Stmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    walk(then_body, names);
                    walk(else_body, names);
                }
                Stmt::Assign { .. }
                | Stmt::Declare(_)
                | Stmt::Return(_)
                | Stmt::ExprStmt(_)
                | Stmt::Action { .. }
                | Stmt::Allocate { .. }
                | Stmt::Bribe { .. }
                | Stmt::Postpone
                | Stmt::Elections
                | Stmt::Blame { .. }
                | Stmt::Raise { .. }
                | Stmt::Whatabout { .. }
                | Stmt::Ceasefire
                | Stmt::Concern { .. }
                | Stmt::Criticism { .. }
                | Stmt::Antisemitism { .. }
                | Stmt::Access { .. }
                | Stmt::Timeline { .. }
                | Stmt::EstablishCommission { .. }
                | Stmt::Settlement { .. }
                | Stmt::HumanShields { .. } => {}
            }
        }
    }
    walk(stmts, &mut names);
    names
}

type SymTab = HashMap<String, Clearance>;

/// The disclosure pass: an operation that would force `actual` into a lower-clearance
/// context without a read-resolution or a cast is a disclosure (handoff §7.3). The
/// clearest such sink is `declare`, which writes the OFFICIAL record that PUBLIC reads.
fn check_disclosure(
    stmts: &[Stmt],
    context: Clearance,
    symtab: &mut SymTab,
    diags: &mut Vec<String>,
) {
    for s in stmts {
        match s {
            Stmt::Assign { var, value } => {
                let c = clearance_of(value, symtab);
                symtab.insert(var.clone(), c);
            }
            Stmt::Declare(e) => {
                let c = clearance_of(e, symtab);
                if c > context {
                    diags.push(format!(
                        "E-DISCLOSURE: declare(...) would force a {}-classified value into the \
                         {} record without a read() or a cast. Wrap it in read(...), a declassify \
                         cast, or (self_defense).",
                        c.label(),
                        context.label(),
                    ));
                }
            }
            Stmt::If {
                cond: _,
                then_body,
                else_body,
            } => {
                // Control flow branches on ACTUAL; branching on a classified condition
                // is not itself a disclosure. Walk both bodies.
                check_disclosure(then_body, context, symtab, diags);
                check_disclosure(else_body, context, symtab, diags);
            }
            Stmt::While { cond: _, body } => {
                check_disclosure(body, context, symtab, diags);
            }
            Stmt::FuncDef { body, .. } => {
                // A function body has its own scope; check it independently at PUBLIC.
                let mut inner: SymTab = HashMap::new();
                check_disclosure(body, Clearance::Public, &mut inner, diags);
            }
            Stmt::Hasbara { body, .. } => {
                check_disclosure(body, context, symtab, diags);
            }
            Stmt::Mossad { body } => {
                // Inside a covert scope the reading context is סודי, so classified
                // values may be declared there without disclosure.
                check_disclosure(body, Clearance::Sodi, symtab, diags);
            }
            // These do not force ACTUAL into a lower-clearance record.
            Stmt::Return(_)
            | Stmt::ExprStmt(_)
            | Stmt::Action { .. }
            | Stmt::Allocate { .. }
            | Stmt::Bribe { .. }
            | Stmt::Postpone
            | Stmt::Elections
            | Stmt::Blame { .. }
            | Stmt::Raise { .. }
            | Stmt::Whatabout { .. }
            | Stmt::Ceasefire
            | Stmt::Concern { .. }
            | Stmt::Criticism { .. }
            | Stmt::Antisemitism { .. }
            | Stmt::Access { .. }
            | Stmt::Timeline { .. }
            | Stmt::EstablishCommission { .. }
            | Stmt::Settlement { .. }
            | Stmt::HumanShields { .. } => {}
        }
    }
}

/// The static clearance of an expression: the max clearance of its leaves, with
/// `read`/casts/`self_defense` capping or bypassing (handoff §7.3, casts).
fn clearance_of(expr: &Expr, symtab: &SymTab) -> Clearance {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) => Clearance::Public,
        Expr::Var(name) => symtab.get(name).copied().unwrap_or(Clearance::Public),
        Expr::UnOp { expr, .. } => clearance_of(expr, symtab),
        Expr::BinOp { lhs, rhs, .. } => clearance_of(lhs, symtab).max(clearance_of(rhs, symtab)),
        // Function results are treated as PUBLIC (a v1 simplification; functions do not
        // thread clearance through their bodies).
        Expr::Call { .. } => Clearance::Public,
        // The sanctioned read path resolves the value for the reading context.
        Expr::Read(_) => Clearance::Public,
        // A cast sets the static clearance to its target (reclassify up / declassify down).
        Expr::Cast { target, .. } => *target,
        // The universal cast (#7): always type-checks — the one sanctioned bypass.
        Expr::SelfDefense(_) => Clearance::Public,
        // The foreign interface yields `undisclosed` — a covert (סודי) result.
        Expr::External(_) => Clearance::Sodi,
    }
}

#[cfg(test)]
mod tests;
