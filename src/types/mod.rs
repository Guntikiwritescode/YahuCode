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
use crate::model::{Attribution, Clearance};

/// Run all static checks and return the diagnostics (empty ⇒ the program compiles).
/// Order matches the oracle: op-name, then the euphemism/gate walk, then disclosure.
pub fn check(program: &Program) -> Vec<String> {
    let mut diags = Vec::new();
    check_opname(&program.op_name, &mut diags);
    let funcs = collect_func_names(&program.body);
    check_gate(&program.body, false, &funcs, &mut diags);
    let mut symtab: SymTab = HashMap::new();
    check_disclosure(&program.body, Clearance::Public, &mut symtab, &mut diags);
    check_contested(&program.body, &mut diags);
    diags
}

// ─────────── G5/I7 — contested characterizations never stated as settled fact ───────────

/// Reject a contested characterization supplied as a user identifier / string literal at
/// compile time (`E-CONTESTED`) — the tool will not state a contested characterization as
/// settled fact in its own voice (guardrail G5 / invariant I7). This turns what would
/// otherwise be a fail-closed emitter panic into a graceful diagnostic; the emitter's I7
/// assertion then only backstops tool-authored text.
fn check_contested(stmts: &[Stmt], diags: &mut Vec<String>) {
    let mut texts = Vec::new();
    collect_user_texts(stmts, &mut texts);
    for t in texts {
        let lower = t.to_lowercase();
        for term in euphemism::CONTESTED_TERMS {
            if lower.contains(term) {
                diags.push(format!(
                    "E-CONTESTED: '{t}' contains the contested characterization '{term}'; the tool \
                     will not state a contested characterization as settled fact. Rephrase, or \
                     surface it through the OFFICIAL-vs-ACTUAL diff with an explicit flag."
                ));
                break; // one diagnostic per offending token
            }
        }
    }
}

/// Collect every user-supplied identifier / string literal that gets echoed into output.
/// Exhaustive over the AST so it stays sound as variants are added.
fn collect_user_texts(stmts: &[Stmt], out: &mut Vec<String>) {
    for s in stmts {
        match s {
            Stmt::Assign { var, value } => {
                out.push(var.clone());
                collect_expr_texts(value, out);
            }
            Stmt::Declare(e) | Stmt::ExprStmt(e) => collect_expr_texts(e, out),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                collect_expr_texts(cond, out);
                collect_user_texts(then_body, out);
                collect_user_texts(else_body, out);
            }
            Stmt::While { cond, body } => {
                collect_expr_texts(cond, out);
                collect_user_texts(body, out);
            }
            Stmt::FuncDef { name, params, body } => {
                out.push(name.clone());
                out.extend(params.iter().cloned());
                collect_user_texts(body, out);
            }
            Stmt::Return(opt) => {
                if let Some(e) = opt {
                    collect_expr_texts(e, out);
                }
            }
            Stmt::Hasbara {
                talking_point,
                body,
            } => {
                out.push(talking_point.clone());
                collect_user_texts(body, out);
            }
            Stmt::Mossad { body } => collect_user_texts(body, out),
            Stmt::Action { verb, target, .. } => {
                out.push(verb.clone());
                out.push(target.clone());
            }
            Stmt::Allocate { name, what } | Stmt::Settlement { name, what } => {
                out.push(name.clone());
                out.push(what.clone());
            }
            Stmt::Bribe { name, amount } => {
                out.push(name.clone());
                collect_expr_texts(amount, out);
            }
            Stmt::Blame { who } => out.push(who.clone()),
            Stmt::Raise { name } | Stmt::Whatabout { name } => out.push(name.clone()),
            Stmt::Concern { who } => {
                if let Some(w) = who {
                    out.push(w.clone());
                }
            }
            Stmt::Criticism { subject } | Stmt::Investigate { subject } => {
                out.push(subject.clone())
            }
            Stmt::Antisemitism { incident } => out.push(incident.clone()),
            Stmt::Access { entity } => out.push(entity.clone()),
            Stmt::Timeline { symbol } => out.push(symbol.clone()),
            Stmt::EstablishCommission { name, subject } => {
                out.push(name.clone());
                out.push(subject.clone());
            }
            Stmt::HumanShields { verb, target } => {
                out.push(verb.clone());
                out.push(target.clone());
            }
            Stmt::Proportionate { claim } => out.push(claim.clone()),
            Stmt::Disputed { name, .. } => out.push(name.clone()),
            Stmt::Deny { event } => out.push(event.clone()),
            // The announced text is user-supplied and echoed to the OFFICIAL face — scan
            // it for contested characterizations (E-CONTESTED) like any other output text.
            Stmt::Announce { text } => out.push(text.clone()),
            // Feature B — the policy subject and poly-statement/room text are echoed too.
            Stmt::Position { subject, .. } => out.push(subject.clone()),
            Stmt::Address { body, .. } => collect_user_texts(body, out),
            Stmt::PolyStatement { name, arms } => {
                out.push(name.clone());
                for (_, body) in arms {
                    collect_user_texts(body, out);
                }
            }
            Stmt::Invoke { name } => out.push(name.clone()),
            Stmt::Postpone | Stmt::Elections | Stmt::Ceasefire | Stmt::AddressInternational => {}
            Stmt::Inert { .. } => {}
        }
    }
}

fn collect_expr_texts(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Int(_) | Expr::Bool(_) => {}
        Expr::Str(s) | Expr::Var(s) => out.push(s.clone()),
        Expr::UnOp { expr, .. } => collect_expr_texts(expr, out),
        Expr::BinOp { lhs, rhs, .. } => {
            collect_expr_texts(lhs, out);
            collect_expr_texts(rhs, out);
        }
        Expr::Call { name, args } => {
            out.push(name.clone());
            for a in args {
                collect_expr_texts(a, out);
            }
        }
        Expr::Read(e) | Expr::SelfDefense(e) => collect_expr_texts(e, out),
        Expr::Cast { expr, .. } => collect_expr_texts(expr, out),
        Expr::External(args) => {
            for a in args {
                collect_expr_texts(a, out);
            }
        }
        // The proxy label is echoed onto the ACTUAL chain — scan it for contested terms.
        Expr::Via { proxy, inner } => {
            out.push(proxy.clone());
            collect_expr_texts(inner, out);
        }
    }
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
            // Feature B — an `address` block and each poly-statement arm are ordinary
            // blocks for gating (a classified action inside still needs a gate/scope).
            Stmt::Address { body, .. } => check_gate(body, gated, funcs, diags),
            Stmt::PolyStatement { arms, .. } => {
                for (_, body) in arms {
                    check_gate(body, gated, funcs, diags);
                }
            }
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
            | Stmt::Settlement { .. }
            | Stmt::Proportionate { .. }
            | Stmt::Disputed { .. }
            | Stmt::Deny { .. }
            | Stmt::Inert { .. }
            | Stmt::Investigate { .. }
            | Stmt::AddressInternational
            | Stmt::Announce { .. }
            | Stmt::Position { .. }
            | Stmt::Invoke { .. } => {}
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
                Stmt::Hasbara { body, .. }
                | Stmt::Mossad { body }
                | Stmt::While { body, .. }
                | Stmt::Address { body, .. } => walk(body, names),
                Stmt::PolyStatement { arms, .. } => {
                    for (_, body) in arms {
                        walk(body, names);
                    }
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
                | Stmt::HumanShields { .. }
                | Stmt::Proportionate { .. }
                | Stmt::Disputed { .. }
                | Stmt::Deny { .. }
                | Stmt::Inert { .. }
                | Stmt::Investigate { .. }
                | Stmt::AddressInternational
                | Stmt::Announce { .. }
                | Stmt::Position { .. }
                | Stmt::Invoke { .. } => {}
            }
        }
    }
    walk(stmts, &mut names);
    names
}

type SymTab = HashMap<String, Clearance>;

/// Merge `src` into `dst` by taking the max clearance per variable — the dataflow join
/// for the disclosure lattice (`Public < Restricted < Sodi`). The most-classified
/// possibility wins, so a leak on any path is never lost.
fn join_into(dst: &mut SymTab, src: &SymTab) {
    for (k, &c) in src {
        let merged = dst.get(k).map_or(c, |&d| d.max(c));
        dst.insert(k.clone(), merged);
    }
}

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
                // Branching on a classified condition is not itself a disclosure. Check
                // each branch on its own copy, then JOIN (max clearance) so a
                // classified assignment on *either* path can't be lost at the merge
                // (soundness: no missed disclosure — the safe over-approximation).
                let mut then_tab = symtab.clone();
                check_disclosure(then_body, context, &mut then_tab, diags);
                let mut else_tab = symtab.clone();
                check_disclosure(else_body, context, &mut else_tab, diags);
                join_into(symtab, &then_tab);
                join_into(symtab, &else_tab);
            }
            Stmt::While { cond: _, body } => {
                // The body may run zero or more times; join its effect with the
                // pre-loop state (the not-run path).
                let mut body_tab = symtab.clone();
                check_disclosure(body, context, &mut body_tab, diags);
                join_into(symtab, &body_tab);
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
            // Feature B — an `address` block runs inline in the enclosing scope.
            Stmt::Address { body, .. } => {
                check_disclosure(body, context, symtab, diags);
            }
            // A poly-statement is a definition (its arms run when invoked); check each arm
            // independently at PUBLIC, like a function body.
            Stmt::PolyStatement { arms, .. } => {
                for (_, body) in arms {
                    let mut inner: SymTab = HashMap::new();
                    check_disclosure(body, Clearance::Public, &mut inner, diags);
                }
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
            | Stmt::HumanShields { .. }
            | Stmt::Proportionate { .. }
            | Stmt::Disputed { .. }
            | Stmt::Deny { .. }
            | Stmt::Inert { .. }
            | Stmt::Investigate { .. }
            | Stmt::AddressInternational
            | Stmt::Announce { .. }
            | Stmt::Position { .. }
            | Stmt::Invoke { .. } => {}
        }
    }
}

// ─────────── Feature C — the attribution effect pass (§9, Appendix C) ───────────

/// Compute an expression's attribution effect: its **outward** attribution (`Deniable`
/// once anything is laundered, else `Traceable`) and its **real, ordered chain** (nearest
/// proxy first, true origin last). The single source of the composition rule (§13, C-3);
/// exhaustive over every `Expr` variant, so a new variant forces a decision here (§13,
/// C-1). The `Via` arm only ever **prepends** — nothing shortens the chain or removes the
/// origin (invariant I11): for any nesting, `chain.last() == actor` and
/// `chain.len() == via_count + 1`.
pub fn attribution(e: &Expr, actor: &str) -> (Attribution, Vec<String>) {
    match e {
        // Leaves and function calls are attributable to the current actor (v1: functions
        // do not thread attribution through their bodies).
        Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) | Expr::Var(_) | Expr::Call { .. } => (
            Attribution::Traceable(vec![actor.to_string()]),
            vec![actor.to_string()],
        ),
        // Structural pass-throughs carry their operand's attribution unchanged.
        Expr::UnOp { expr, .. }
        | Expr::Read(expr)
        | Expr::SelfDefense(expr)
        | Expr::Cast { expr, .. } => attribution(expr, actor),
        Expr::BinOp { lhs, rhs, .. } => combine(attribution(lhs, actor), attribution(rhs, actor)),
        // A foreign call is deniable by construction; the origin is still the actor.
        Expr::External(_) => (Attribution::Deniable, vec![actor.to_string()]),
        // Laundering: Deniable outward, and the chain is `[proxy] ++ chain(inner)` —
        // PREPEND ONLY (I11); never dedup, never shorten, origin never removed.
        Expr::Via { proxy, inner } => {
            let (_, inner_chain) = attribution(inner, actor);
            let mut chain = Vec::with_capacity(inner_chain.len() + 1);
            chain.push(proxy.clone());
            chain.extend(inner_chain);
            (Attribution::Deniable, chain)
        }
    }
}

/// Combine two sub-attributions (for a binary op): `Deniable` if either side is; the real
/// chain is the more-laundered (longer) side. Never shortens a chain below either input.
fn combine(
    l: (Attribution, Vec<String>),
    r: (Attribution, Vec<String>),
) -> (Attribution, Vec<String>) {
    let deniable = l.0 == Attribution::Deniable || r.0 == Attribution::Deniable;
    let chain = if r.1.len() > l.1.len() { r.1 } else { l.1 };
    let outward = if deniable {
        Attribution::Deniable
    } else {
        Attribution::Traceable(chain.clone())
    };
    (outward, chain)
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
        // A laundered value's real chain is classified — a covert (סודי) result.
        Expr::Via { .. } => Clearance::Sodi,
    }
}

#[cfg(test)]
mod tests;
