# YahuCode

YahuCode consists of four ordinary compiler ideas that are each replaced by a thematic mechanism to create a unique system.

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

The **Field Office** standard library turns the euphemism/censor apparatus on the very
citizen who installed it: `surveil` ("voluntary transparency"), `intercept(n)` (the
read-only intake channel), `did_you_mean` (the Spokesperson surfaced as an action),
`flag` (a grow-only, no-appeal watchlist), `alternate_facts` (the euphemized claim
presented as the fact), and the `voluntary { … }` scope. A companion browser tool — a
separate `wasm-bindgen` wrapper crate (`field-office/yahucode-wasm/`, keeping the core
dependency-free) plus a Manifest V3 extension (`field-office/extension/`) — surveils the
user's own screen and "helpfully" censors them; all policy lives in the flagship program
[`examples/20_guardian_of_discourse.yahu`](examples/20_guardian_of_discourse.yahu) and the
euphemism table, never in the JS. The butt is always the maneuver, applied to the user,
never any group.

## Build, run, test

```sh
cargo build                    # build the interpreter
cargo run -- <file.yahu>       # run a program; print the OFFICIAL vs ACTUAL diff
cargo run -- --json <file>     # structured projection {official, actual, discrepancies}
cargo run -- --press <file>    # the public build: press release + rewritten comments
cargo run -- --intercept "…" <file>   # seed the Field Office intake channel (repeatable)
cargo test                     # golden + invariant + feature + framing + backlog suites
cargo clippy --all-targets -- -D warnings   # lint (warning-free)
cargo fmt --check              # formatting
```

Requires a stable Rust toolchain (developed against 1.94). See
[`LANGUAGE.md`](LANGUAGE.md) for the full language reference.

## Host language

Rust — its `match` is exhaustive by default, which is the project's primary defence
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
  emit/           three-tier projection (the one read path) + --json + error_json + the diff render
  cli/            arg parsing, file IO, flags (--json, --press, --audience, --intercept)
  lib.rs          module wiring + run_json (the library/WASM entry, reusing emit's JSON path)
tests/            golden / invariants / features / framing suites
examples/         the programs as .yahu source (20_guardian_of_discourse = the Field Office flagship)
docs/             the execution handoff and the sourcing appendix
field-office/
  yahucode-wasm/  a SEPARATE wasm-bindgen wrapper (core stays dependency-free)
  extension/      the Manifest V3 "Mossad-Clippy" (thin JS shim; policy lives in the .yahu)
```

## Status

Complete for v1 (phases 0–6 of `docs/cc-handoff.md`). The interpreter reproduces the
v1 behavioural oracle exactly: all nine oracle programs are frozen as byte-for-byte
golden fixtures in `tests/golden/`, and the Python spike has been removed — the
behaviour is preserved via the goldens, not the code. Four test suites run on every
change: **golden** (oracle parity), **invariants** (I1–I17 + coalition monotonicity),
**features** (a positive + negative test per feature), and **framing** (the guardrail
regression tests for the sensitive features). See [`LANGUAGE.md`](LANGUAGE.md) for the
language reference and [`docs/sources.md`](docs/sources.md) for every real-world anchor
and its verification status.

The **Field Office** extension (the "Mossad-Clippy") adds the intake channel `intercept(n)`
and six constructs (`surveil`, `did_you_mean`, `flag`, `alternate_facts`, `voluntary`),
with invariants **I16** (content-free intake) and **I17** (the censor is one-way and never
forgets). The core crate stays dependency-free; a separate `field-office/yahucode-wasm/`
wrapper exposes `run_json` to the browser extension.
