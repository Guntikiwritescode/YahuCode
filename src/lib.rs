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
