//! YahuCode — a satirical esoteric programming language.
//!
//! **Subject (guardrails, docs/cc-handoff.md §2):** the messaging apparatus and
//! wartime conduct of the *Israeli government* — a government and its rhetoric,
//! never a people. The comedy engine is the gap between what code does and what you
//! are permitted to call it: euphemism, deflection, unaccountability, two-faced
//! messaging. The butt is always the maneuver, never the victims (G2); `OFFICIAL`
//! is always the prettier lie and `ACTUAL` the uglier truth (G3, invariant I1).
//!
//! ## Module dependency direction (strictly downward, acyclic — §6)
//! `cli → emit → runtime → lower → types → parser → euphemism ← lexer`, with the
//! core `model` at the bottom, depended on by all. `euphemism/` is the single source
//! of truth for the table, `E`, and `redact`.

// Warnings are denied in CI (a clean, warning-free build is definition-of-done #9).

pub mod ast;
pub mod cli;
pub mod config;
pub mod emit;
pub mod euphemism;
pub mod lexer;
pub mod model;
pub mod parser;
pub mod runtime;
pub mod types;

/// The library run entry — parse → static-check → run → emit the structured
/// `{official, actual, discrepancies, …}` JSON (the same projection the CLI's `--json`
/// flag produces). `intercepts` seeds the Field Office intake channel (`intercept(n)`);
/// the WASM wrapper crate calls this so the browser shim can drive YahuCode without a
/// filesystem or a process. Reuses the existing emit/JSON path — it does not duplicate it.
///
/// On a parse or compile failure it returns a structured `{"error", "diagnostics"}` object
/// (via [`emit::error_json`]) so a JS caller always receives valid JSON.
pub fn run_json(src: &str, intercepts: Vec<String>) -> String {
    let program = match parser::parse(src) {
        Ok(p) => p,
        Err(e) => return emit::error_json(&e.to_string(), &[]),
    };
    let diags = types::check(&program);
    if !diags.is_empty() {
        return emit::error_json("program does not compile", &diags);
    }
    let st = runtime::run_with_intercepts(&program, intercepts);
    emit::to_json(&st)
}
