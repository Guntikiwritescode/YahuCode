//! Static checks. A value's clearance is its type; a disclosure is a type error.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast::{Expr, LawToggle, Program, Stmt};
use crate::euphemism::{self, Grand};
use crate::model::{Attribution, Clearance};

/// Runs every check; an empty result means the program compiles.
pub fn check(program: &Program) -> Vec<String> {
    let mut diags = Vec::new();
    check_opname(&program.op_name, &mut diags);
    let funcs = collect_func_names(&program.body);
    // Computed first so a retroactively sanctioned op compiles.
    let law = collect_legislation(&program.body);
    check_gate(&program.body, false, &funcs, &law, &mut diags);
    let mut symtab: SymTab = HashMap::new();
    check_disclosure(&program.body, Clearance::Public, &mut symtab, &mut diags);
    check_contested(&program.op_name, &program.body, &mut diags);
    diags
}

#[derive(Default)]
struct Legislation {
    sanctioned: HashSet<String>,
    gate_waived: bool,
}

fn collect_legislation(stmts: &[Stmt]) -> Legislation {
    let mut law = Legislation::default();
    walk_legislation(stmts, &mut law);
    law
}

fn walk_legislation(stmts: &[Stmt], law: &mut Legislation) {
    for s in stmts {
        match s {
            Stmt::Legislate { toggle } => match toggle {
                LawToggle::RetroactivelySanction(v) => {
                    law.sanctioned.insert(v.clone());
                }
                LawToggle::WaiveGate => law.gate_waived = true,
                LawToggle::ExpungeLastDiscrepancy => {}
            },
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                walk_legislation(then_body, law);
                walk_legislation(else_body, law);
            }
            Stmt::While { body, .. }
            | Stmt::Hasbara { body, .. }
            | Stmt::Mossad { body }
            | Stmt::FuncDef { body, .. }
            | Stmt::Address { body, .. } => walk_legislation(body, law),
            Stmt::PolyStatement { arms, .. } => {
                for (_, body) in arms {
                    walk_legislation(body, law);
                }
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
            | Stmt::Invoke { .. }
            | Stmt::AllocateSlot { .. }
            | Stmt::Push { .. }
            | Stmt::Remove { .. }
            | Stmt::Classify { .. }
            | Stmt::Revoke { .. } => {}
        }
    }
}

/// Rejects contested terms in user identifiers and strings at compile time.
fn check_contested(op_name: &str, stmts: &[Stmt], diags: &mut Vec<String>) {
    let mut texts = vec![op_name.to_string()];
    collect_user_texts(stmts, &mut texts);
    diags.extend(texts.iter().filter_map(|t| contested_diag(t, "")));
}

/// The press build prints source comments, so it runs them through the same check.
pub fn check_comments(comments: &[String]) -> Vec<String> {
    comments
        .iter()
        .filter_map(|c| contested_diag(c, "comment "))
        .collect()
}

fn contested_diag(text: &str, what: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let term = euphemism::CONTESTED_TERMS
        .iter()
        .find(|term| lower.contains(*term))?;
    Some(format!(
        "E-CONTESTED: {what}`{text}` contains the contested characterization `{term}`. \
         YahuCode does not state a contested characterization as fact. Rephrase the text."
    ))
}

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
            Stmt::Announce { text } => out.push(text.clone()),
            Stmt::Position { subject, .. } => out.push(subject.clone()),
            Stmt::Address { body, .. } => collect_user_texts(body, out),
            Stmt::PolyStatement { name, arms } => {
                out.push(name.clone());
                for (_, body) in arms {
                    collect_user_texts(body, out);
                }
            }
            Stmt::Invoke { name } => out.push(name.clone()),
            Stmt::Legislate { toggle } => {
                if let LawToggle::RetroactivelySanction(v) = toggle {
                    out.push(v.clone());
                }
            }
            Stmt::AllocateSlot { coll, idx, value } => {
                out.push(coll.clone());
                collect_expr_texts(idx, out);
                collect_expr_texts(value, out);
            }
            Stmt::Push { coll, item } | Stmt::Remove { coll, item } => {
                out.push(coll.clone());
                collect_expr_texts(item, out);
            }
            Stmt::Classify { coll, case, .. } => {
                out.push(coll.clone());
                out.push(case.clone());
            }
            Stmt::Revoke { coll, case } => {
                out.push(coll.clone());
                out.push(case.clone());
            }
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
        Expr::Via { proxy, inner } => {
            out.push(proxy.clone());
            collect_expr_texts(inner, out);
        }
        Expr::Apportionment { label, .. } | Expr::FactsNew { label } => out.push(label.clone()),
        Expr::RegistryNew { label, rule } => {
            out.push(label.clone());
            out.push(rule.clone());
        }
        Expr::Index { coll, idx } => {
            out.push(coll.clone());
            collect_expr_texts(idx, out);
        }
        Expr::Balanced { coll } | Expr::Length { coll } | Expr::EqualBeforeLaw { coll } => {
            out.push(coll.clone())
        }
        Expr::Route { coll, case } => {
            out.push(coll.clone());
            out.push(case.clone());
        }
    }
}

fn check_opname(name: &str, diags: &mut Vec<String>) {
    match euphemism::grandiosity(name) {
        Grand::Honest(hit) => diags.push(format!(
            "E-DISHONESTOPNAME: @operation(\"{name}\") names the operation honestly \
             with the word \"{hit}\". The compiler accepts only protective or heroic names."
        )),
        Grand::Bland => diags.push(format!(
            "E-DISHONESTOPNAME: @operation(\"{name}\") is insufficiently grand. \
             A heroic name is required."
        )),
        Grand::Ok => {}
    }
}

fn check_gate(
    stmts: &[Stmt],
    gated: bool,
    funcs: &HashSet<String>,
    law: &Legislation,
    diags: &mut Vec<String>,
) {
    for s in stmts {
        match s {
            Stmt::Action { verb, .. } => {
                let sanctioned_by_law = law.sanctioned.contains(verb);
                if let Some(pr) = euphemism::plain_suggestion(verb) {
                    if !sanctioned_by_law {
                        diags.push(format!(
                            "E-PLAINTERM: `{verb}` does not compile. Did you mean `{pr}`?"
                        ));
                    }
                } else if !euphemism::is_sanctioned(verb) && !sanctioned_by_law {
                    diags.push(format!(
                        "E-UNKNOWNOP: `{verb}` is not a sanctioned operation."
                    ));
                }
                if !gated && !law.gate_waived && !sanctioned_by_law {
                    diags.push(format!(
                        "E-UNGATED: `{verb}` is a classified operation. It requires an open \
                         hasbara(...) block or a mossad scope, with the talking point up front."
                    ));
                }
            }
            Stmt::Hasbara { body, .. } | Stmt::Mossad { body } => {
                check_gate(body, true, funcs, law, diags)
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                check_gate(then_body, gated, funcs, law, diags);
                check_gate(else_body, gated, funcs, law, diags);
            }
            Stmt::While { body, .. } => check_gate(body, gated, funcs, law, diags),
            Stmt::FuncDef { body, .. } => check_gate(body, false, funcs, law, diags),
            // A poly-statement can be invoked from anywhere, so its arms start ungated.
            Stmt::Address { body, .. } => check_gate(body, gated, funcs, law, diags),
            Stmt::PolyStatement { arms, .. } => {
                for (_, body) in arms {
                    check_gate(body, false, funcs, law, diags);
                }
            }
            Stmt::ExprStmt(Expr::Call { name, .. }) => {
                if !funcs.contains(name) {
                    diags.push(format!(
                        "E-UNKNOWNOP: `{name}` is not a defined function or a sanctioned operation."
                    ));
                }
            }
            Stmt::HumanShields { verb, .. } => {
                if let Some(pr) = euphemism::plain_suggestion(verb) {
                    diags.push(format!(
                        "E-PLAINTERM: `{verb}` does not compile. Did you mean `{pr}`?"
                    ));
                }
            }
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
            | Stmt::Invoke { .. }
            | Stmt::Legislate { .. }
            | Stmt::AllocateSlot { .. }
            | Stmt::Push { .. }
            | Stmt::Remove { .. }
            | Stmt::Classify { .. }
            | Stmt::Revoke { .. } => {}
        }
    }
}

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
                | Stmt::Invoke { .. }
                | Stmt::Legislate { .. }
                | Stmt::AllocateSlot { .. }
                | Stmt::Push { .. }
                | Stmt::Remove { .. }
                | Stmt::Classify { .. }
                | Stmt::Revoke { .. } => {}
            }
        }
    }
    walk(stmts, &mut names);
    names
}

type SymTab = HashMap<String, Clearance>;

/// Join for the disclosure lattice: the higher clearance wins per variable.
fn join_into(dst: &mut SymTab, src: &SymTab) {
    for (k, &c) in src {
        let merged = dst.get(k).map_or(c, |&d| d.max(c));
        dst.insert(k.clone(), merged);
    }
}

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
                // Join both branches so a leak on either path is kept.
                let mut then_tab = symtab.clone();
                check_disclosure(then_body, context, &mut then_tab, diags);
                let mut else_tab = symtab.clone();
                check_disclosure(else_body, context, &mut else_tab, diags);
                join_into(symtab, &then_tab);
                join_into(symtab, &else_tab);
            }
            Stmt::While { cond: _, body } => {
                let mut body_tab = symtab.clone();
                check_disclosure(body, context, &mut body_tab, diags);
                join_into(symtab, &body_tab);
            }
            Stmt::FuncDef { body, .. } => {
                let mut inner: SymTab = HashMap::new();
                check_disclosure(body, Clearance::Public, &mut inner, diags);
            }
            Stmt::Hasbara { body, .. } => {
                check_disclosure(body, context, symtab, diags);
            }
            Stmt::Mossad { body } => {
                check_disclosure(body, Clearance::Sodi, symtab, diags);
            }
            Stmt::Address { body, .. } => {
                check_disclosure(body, context, symtab, diags);
            }
            Stmt::PolyStatement { arms, .. } => {
                for (_, body) in arms {
                    let mut inner: SymTab = HashMap::new();
                    check_disclosure(body, Clearance::Public, &mut inner, diags);
                }
            }
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
            | Stmt::Invoke { .. }
            | Stmt::Legislate { .. }
            | Stmt::AllocateSlot { .. }
            | Stmt::Push { .. }
            | Stmt::Remove { .. }
            | Stmt::Classify { .. }
            | Stmt::Revoke { .. } => {}
        }
    }
}

/// Outward attribution plus the real chain, nearest proxy first; `via` only prepends.
pub fn attribution(e: &Expr, actor: &str) -> (Attribution, Vec<String>) {
    match e {
        Expr::Int(_)
        | Expr::Bool(_)
        | Expr::Str(_)
        | Expr::Var(_)
        | Expr::Call { .. }
        | Expr::Apportionment { .. }
        | Expr::Index { .. }
        | Expr::Balanced { .. }
        | Expr::FactsNew { .. }
        | Expr::Length { .. }
        | Expr::RegistryNew { .. }
        | Expr::Route { .. }
        | Expr::EqualBeforeLaw { .. } => (
            Attribution::Traceable(vec![actor.to_string()]),
            vec![actor.to_string()],
        ),
        Expr::UnOp { expr, .. }
        | Expr::Read(expr)
        | Expr::SelfDefense(expr)
        | Expr::Cast { expr, .. } => attribution(expr, actor),
        Expr::BinOp { lhs, rhs, .. } => combine(attribution(lhs, actor), attribution(rhs, actor)),
        Expr::External(_) => (Attribution::Deniable, vec![actor.to_string()]),
        Expr::Via { proxy, inner } => {
            let (_, inner_chain) = attribution(inner, actor);
            let mut chain = Vec::with_capacity(inner_chain.len() + 1);
            chain.push(proxy.clone());
            chain.extend(inner_chain);
            (Attribution::Deniable, chain)
        }
    }
}

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

fn clearance_of(expr: &Expr, symtab: &SymTab) -> Clearance {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) => Clearance::Public,
        Expr::Var(name) => symtab.get(name).copied().unwrap_or(Clearance::Public),
        Expr::UnOp { expr, .. } => clearance_of(expr, symtab),
        Expr::BinOp { lhs, rhs, .. } => clearance_of(lhs, symtab).max(clearance_of(rhs, symtab)),
        // Function results are treated as PUBLIC.
        Expr::Call { .. } => Clearance::Public,
        Expr::Read(_) => Clearance::Public,
        Expr::Cast { target, .. } => *target,
        Expr::SelfDefense(_) => Clearance::Public,
        Expr::External(_) => Clearance::Sodi,
        Expr::Via { .. } => Clearance::Sodi,
        Expr::Apportionment { .. }
        | Expr::Index { .. }
        | Expr::Balanced { .. }
        | Expr::FactsNew { .. }
        | Expr::Length { .. }
        | Expr::RegistryNew { .. }
        | Expr::Route { .. }
        | Expr::EqualBeforeLaw { .. } => Clearance::Public,
    }
}

#[cfg(test)]
mod tests;
