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

use crate::ast::{Expr, Program, Stmt};
use crate::model::Clearance;

/// Run all static checks and return the diagnostics (empty ⇒ the program compiles).
pub fn check(program: &Program) -> Vec<String> {
    let mut diags = Vec::new();
    let mut symtab: SymTab = HashMap::new();
    check_disclosure(&program.body, Clearance::Public, &mut symtab, &mut diags);
    diags
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
            // These do not write ACTUAL into the OFFICIAL record in Phase 2.
            Stmt::Return(_) | Stmt::ExprStmt(_) | Stmt::Action { .. } => {}
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
    }
}

#[cfg(test)]
mod tests;
