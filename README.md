# YahuCode

A **satirical esoteric programming language**. Its subject is the **messaging
apparatus and wartime conduct of the Israeli government** — a government and its
rhetoric, *never a people*. The comedy engine is the gap between what code *does*
and what you are permitted to *call it*: euphemism, deflection, unaccountability,
two-faced messaging.

> YahuCode aims at a government's behaviour and its excuses. It never makes Jewish
> people, Judaism, or Hebrew-as-identity the joke; Hebrew appears only as the texture
> of officialdom. The butt is always the maneuver, never the victims. See
> [`docs/cc-handoff.md`](docs/cc-handoff.md) §2 for the full guardrails, and
> [`docs/sources.md`](docs/sources.md) for the sourcing of every real-world anchor.

It is a genuinely novel little machine, not a reskin. Four ordinary compiler ideas
are each replaced by a thematic mechanism that is load-bearing:

- **Types → clearance/audience.** A value's type is *who may see it*
  (`PUBLIC`, `RESTRICTED`, `סודי`). A type error is a **disclosure**.
- **Memory → coalition.** Allocations persist only while continuously bribed;
  running out is **elections**.
- **Halting → staying in power.** There is no terminating state; the only exit is
  failure (**elections**). "The program ends" means "the government falls."
- **One artifact → the diff.** Running a program emits both faces side by side —
  the **OFFICIAL** press release beside the **ACTUAL** candid truth, plus a
  discrepancy count. The run *is* the joke.

## Build, run, test

```sh
cargo build            # build the interpreter
cargo run -- <file>    # run a .yahu program (CLI lands in PR2)
cargo test             # golden + invariant + per-feature + framing suites
cargo clippy --all-targets -- -D warnings   # lint (warning-free)
cargo fmt --check      # formatting
```

Requires a stable Rust toolchain (developed against 1.94).

## Host language

Rust — its `match` is exhaustive by default, which is the project's primary defense
against unhandled-case bugs (`docs/cc-handoff.md` §12, Appendix H). The core
taxonomies (clearance, provenance, truth, node kinds, value payloads) are **closed
sum types** matched exhaustively everywhere; there are no wildcard `_ =>` arms in the
core consumers.

## Repository layout

```
src/
  model.rs        core data model: the closed sum types (clearance, truth, value, log)
  config.rs       RuntimeConfig — every runtime number, named (no magic constants)
  euphemism/      THE euphemism table + E + redact + grandiosity + data (single source of truth)
  lexer/          source → tokens; plain-term detection
  parser/         tokens → dual-annotated AST; provenance set at construction (I2)
  types/          static checks: op-name, euphemism typing, hasbara gate, disclosure, casts
  lower/          feature desugaring (ceasefire → continue, etc.)
  runtime/        the step relation: ACTUAL/OFFICIAL/DISCREPANCY, coalition, mossad, errors
  emit/           three-tier projection (the one read path) + --json + the diff render
  cli/            arg parsing, file IO, flags
tests/            golden / invariants / features / framing suites
examples/         the programs as .yahu source
docs/             the execution handoff and the sourcing appendix
```

## Status

Under active construction against the phased runbook in `docs/cc-handoff.md`
(Appendix F). The v1 Python spike in `oracle/` is the behavioural oracle; its
observable behaviour is frozen as golden fixtures and reproduced exactly, after which
the spike is removed (behaviour is preserved via the goldens, not the code).
