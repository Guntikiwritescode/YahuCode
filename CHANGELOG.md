# Changelog

All notable changes to YahuCode, by release stage. Each stage is retrievable by its git
tag (`v1`, `v2`, `v3`). YahuCode is a satirical esoteric programming language whose
subject is the messaging apparatus and wartime conduct of a government — a government and
its rhetoric, never a people. See [`docs/cc-handoff.md`](docs/cc-handoff.md) §2 for the
full guardrails, [`README.md`](README.md) and [`LANGUAGE.md`](LANGUAGE.md) for the
overview, and [`docs/sources.md`](docs/sources.md) for the sourcing of every real-world
anchor.

## [v4] — Field Office (the "Mossad-Clippy")

A standard library that turns the euphemism/censor apparatus on the very citizen who
installed it, plus a voluntary browser tool that surveils the user's own screen and
"helpfully" censors them. The butt is always the censorship maneuver, applied to the user,
never any group.

### Added
- **The intake channel** `intercept(n)` (Feature): YahuCode's only runtime intake vector.
  `State.intercepts` is host-supplied and set once at construction (the CLI `--intercept`
  flag, or the WASM host). It records a two-faced intake event and returns the retained
  text; an out-of-range index is a controlled `E-INTAKE` diagnostic.
- **Six constructs**: `surveil` [framed], `did_you_mean`, `flag` [framed], `alternate_facts`
  [framed], and the `voluntary { … }` scope — each reusing the named engine (the euphemism
  table, `E`, a `FactsList` watchlist) rather than re-implementing it.
- Invariants **I16** (content-free intake — the retained/surveilled/flagged content never
  reaches a PUBLIC reader) and **I17** (the censor is one-way and never forgets — a grow-only
  watchlist, no un-flag/appeal, no `E⁻¹`).
- The library entry `run_json(src, intercepts)` (reusing the emit/JSON path) and a **separate**
  `field-office/yahucode-wasm/` wrapper crate (core stays dependency-free; `wasm-bindgen` lives
  only there) exposing it to the browser.
- The Manifest V3 extension `field-office/extension/` — a thin JS shim (read → `run_json` →
  parse → paint) with the flagship `examples/20_guardian_of_discourse.yahu` as the default
  policy. All policy lives in the `.yahu` program and the euphemism table, never in the JS.
- Example program **20** (`20_guardian_of_discourse`) and a minimal intake example, both
  frozen as golden fixtures, plus per-construct feature/framing tests and the I16/I17 checks.

### Notes
- The core crate remains **dependency-free by design**; the core lib builds unchanged for
  `wasm32-unknown-unknown` (no std feature had to be isolated).
- No new real-world claims were baked in: `did_you_mean`/`alternate_facts` reuse the already
  sourced euphemism relabelings (`docs/sources.md` anchors 6–8, 17); the surveil/intercept/
  flag/voluntary faces are language stipulations.

## [v3] — Collections

Three collection types, each carrying the one-way-truth discipline: real contents only
accrete and stay reconstructible to a cleared (`סודי`) reader; nothing is ever erased.

### Added
- **Apportionment** (array, Feature E): a fixed-size allotment whose OFFICIAL face
  proclaims "equal shares" while the ACTUAL vector carries the skew.
- **FactsList** (list, Feature F): a grow-only ledger; `remove` delists an entry to a
  `סודי` shadow, never deletes it — there is no `hard_delete`, and its absence is the
  feature.
- **Registry** (map, Feature G): proclaims one uniform rule while routing cases
  differentially; keys are cases/statuses, never identity labels; `revoke` retains,
  never erases.
- Invariants **I13** (list real-length monotone), **I14** (registry non-erasure), and
  **I15** (element disclosure — a covert slot / list entry / registry case never leaks to
  an under-`סודי` reader).
- Example programs **16–19** (`16_iron_equity`, `17_solid_ground`, `18_eternal_justice`,
  `19_guardian_of_transparency`), each frozen as a byte-exact golden fixture.
- `docs/sources.md` anchor **17** (West Bank dual legal system; the apartheid label
  flagged **CONTESTED** per invariant I7).

### Fixed (collections review pass)
- Hardened after an adversarial review: the I15 covert-element leak, integer overflow,
  negative apportionment shares, and collection-name rebinding.

### Notes
- Carries forward the four v2 bug fixes (below); no regression to features 1–25 or
  invariants I1–I12. Feature count after v3: **28** (21 base + 4 v2 dynamics + 3
  collections).

## [v2] — Four dynamics

Four new language dynamics, each mechanizing a documented rhetorical maneuver.

### Added
- **Feature A** — `announce` / lossy authoring (invariant **I9**: an announced claim has
  no recoverable ACTUAL at any clearance; there is no `E⁻¹`).
- **Feature B** — audience-polymorphic dispatch (invariant **I10**; the `W-DOUBLETALK`
  warning surfaces contradictory per-audience messaging to the out-of-world observer).
- **Feature C** — attribution effect system (invariant **I11**: laundering only prepends
  an origin, never removes it; the `mossad`-blame special case is folded in and deleted).
- **Feature D** — `legislate` (invariant **I12**: an indelible `סודי` meta-trace, no
  clean fixed point; the international-law framing flagged **CONTESTED** per I7).
- `docs/sources.md` anchors **14–16**.

### Fixed (found by adversarial review)
- Recursion stack-overflow → a shared call-depth guard (`max_depth`, `enter_call` /
  `leave_call`).
- `return` inside a poly (audience) arm unwound the whole program → bounded at the arm
  boundary.
- A classified op in a poly arm defined inside `mossad` / `hasbara` bypassed the gate →
  arms are now gate-checked as fresh, ungated scopes.
- `postpone` / `tick` panicked on i64 overflow → saturating arithmetic.

## [v1] — Base

### Added
- 21 features: the clearance/audience type system (a type error is a **disclosure**),
  coalition memory with `elections` as the only exit (no halting state), the euphemism
  front end and Spokesperson, `mossad` covert scope with `undisclosed` contagion, the
  five sensitive features (#15–#19) whose framing is part of the spec, and the backlog
  rhetorical stdlib.
- Invariants **I1–I8** (plus coalition monotonicity).
- The four test suites: **golden** (oracle parity), **invariants**, **features**
  (a positive and a negative test per feature), and **framing** (the guardrail-regression
  suite for the sensitive features).
- `docs/sources.md` anchors **1–13**.
