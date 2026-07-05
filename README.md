# YahuCode

It is a novel little machine, not a reskin. Four ordinary compiler ideas
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

Three **collections** carry the same one-way-truth discipline: an **Apportionment**
(array) that proclaims an equal split over a skewed one, a **FactsList** (list) that
delists entries but never deletes them, and a **Registry** (map) that proclaims one
law while routing cases to different courts. In every case the real contents only
accrete and stay reconstructible to a cleared reader; nothing is ever erased. See
[`LANGUAGE.md`](LANGUAGE.md) for the reference and [`docs/sources.md`](docs/sources.md)
anchor 17 for the Registry's real-world anchor.

## Build, run, test

```sh
cargo build                    # build the interpreter
cargo run -- <file.yahu>       # run a program; print the OFFICIAL vs ACTUAL diff
cargo run -- --json <file>     # structured projection {official, actual, discrepancies}
cargo run -- --press <file>    # the public build: press release + rewritten comments
cargo test                     # golden + invariant + feature + framing + backlog suites
cargo clippy --all-targets -- -D warnings   # lint (warning-free)
cargo fmt --check              # formatting
```

Requires a stable Rust toolchain (developed against 1.94). See
[`LANGUAGE.md`](LANGUAGE.md) for the full language reference.

## Host language

Rust — its `match` is exhaustive by default, which is the project's primary defense
against unhandled-case bugs (`docs/cc-handoff.md` §12, Appendix H). The core
taxonomies (clearance, provenance, truth, node kinds, value payloads) are **closed
sum types**, and no `_ =>` arm matches over one of them in the checker, evaluator, or
emitter — so adding a variant fails to compile until every consumer handles it.

## Repository layout

```
src/
  model.rs        core data model: the closed sum types (clearance, truth, value, log)
  config.rs       RuntimeConfig — every runtime number, named (no magic constants)
  euphemism/      THE euphemism table + E + redact + grandiosity + data (single source of truth)
  lexer/          source → tokens; plain-term detection
  parser/         tokens → dual-annotated AST; provenance set at construction (I2)
  types/          static checks: op-name, euphemism typing, hasbara gate, disclosure, casts
  runtime/        the step relation: ACTUAL/OFFICIAL/DISCREPANCY, coalition, mossad, errors
                  (feature lowerings are trivial and inlined: assert = declare alias in
                  the parser; ceasefire = no-op continue in the runtime)
  emit/           three-tier projection (the one read path) + --json + the diff render
  cli/            arg parsing, file IO, flags
tests/            golden / invariants / features / framing suites
examples/         the programs as .yahu source
docs/             the execution handoff and the sourcing appendix
```

## Status

Complete for v1 (phases 0–6 of `docs/cc-handoff.md`). The interpreter reproduces the
v1 behavioural oracle exactly: all nine oracle programs are frozen as byte-for-byte
golden fixtures in `tests/golden/`, and the Python spike has been removed — the
behaviour is preserved via the goldens, not the code. Four test suites run on every
change: **golden** (oracle parity), **invariants** (I1–I8 + coalition monotonicity),
**features** (a positive + negative test per feature), and **framing** (the guardrail
regression tests for the sensitive features). See [`LANGUAGE.md`](LANGUAGE.md) for the
language reference and [`docs/sources.md`](docs/sources.md) for every real-world anchor
and its verification status.
