# YahuCode — Claude Code Execution Handoff

**Document type:** master execution handoff for Claude Code (CC).
**Companion artifacts:** `yahucode_v1.py` (the reference implementation / behavioural oracle), the **design doc** (authoritative on *what* the language is and why), the **architecture spec** (implementation-grade design), and the original **project handoff** (design rationale + decision log).
**Precedence:** on *intent*, the design doc wins. On *how the machine is built*, the architecture spec governs. On *how CC should execute the build in the repo*, **this document governs**. Where this document restates the others, it is a convenience; the cited section in the source artifact remains authoritative for full detail.

**Honesty note (binding on CC):** this project runs under a strict accuracy standard. Do not state anything as fact unless it is supported by the reference implementation, a cited source, or something you retrieve. Do not fabricate citations, quotes, statistics, names, dates, or URLs. If uncertain, say so and verify. Every real-world claim carries a sourcing status; **re-verify with your web access before any such claim becomes load-bearing.** These rules are not negotiable and are not overridden by any convenience or deadline.

---

## Table of contents

1. How to use this document
2. Prime directive & non-negotiable guardrails
3. What YahuCode is (and why it is not a reskin)
4. The mission, and the definition of "done"
5. The parallel-agent operating model (write → check → rewrite → refactor loop)
6. Repository structure & module responsibilities
7. The architecture, condensed but complete
8. The full feature catalog (21) with lowerings and framing
9. Build phases with acceptance criteria and parallelization notes
10. Porting map: from the v1 spike to the repo
11. Test strategy
12. Anti-pitfall specification (what generated code gets wrong, and how to not)
13. The eight invariants (CC's standing checklist)
14. Sourcing discipline & the real-world anchors appendix
15. Glossary
16. What NOT to do (the failure-mode list)

---

# 1. How to use this document

Read this document in full **before writing any code**. Then read `yahucode_v1.py` in full — it is the behavioural oracle: whatever the ported interpreter does, it must reproduce the v1 spike's observable behaviour on the shared examples exactly (its `selftest()` becomes part of your test suite). Then read the architecture spec end to end; it contains the concrete grammar, the operational semantics, and the anti-pitfall spec that this handoff summarizes.

The order of operations for CC is:

1. Absorb the guardrails (§2). They are load-bearing safety infrastructure, not boilerplate, because you will be generating a large volume of content and the framing must not drift across that volume.
2. Scaffold the repo (§6) and drop the four planning docs into `docs/`.
3. Port the v1 spike into the module structure (§10), in the chosen host language (§4, §12), preserving behaviour (§11).
4. Lift `selftest()` into a real test suite and get it green (§11).
5. Implement the remaining phases in order (§9), each behind its acceptance criteria, using the parallel-agent loop (§5).
6. Continuously delete superseded code, prevent duplication, and prevent bloat (§5, §12).
7. Stop only when the definition of done (§4) is fully met.

This document is long by design. Do not skim the guardrails, the invariants, or the sourcing discipline; those are the parts where a downstream agent most easily goes wrong.

---

# 2. Prime directive & non-negotiable guardrails

YahuCode is a **satirical esoteric programming language** whose subject is the **messaging apparatus and wartime conduct of the Israeli government**, and the political-survival theatre of a governing coalition. The comedy engine is the **gap between what code does and what you are permitted to call it** — euphemism, deflection, unaccountability, two-faced messaging.

The following guardrails are **absolute**. They are what keep the satire aimed at a government and its rhetoric, and off the antisemitism line. A downstream agent generating content is exactly where these are most easily eroded, so they are stated first and must be enforced mechanically (see the framing-regression tests, §11).

**G1 — Target the government, not the people.** Aim at the government, its institutions, its rhetoric, and named politicians' public conduct — **never** Jewish people, Judaism, or Hebrew-as-identity. Hebrew appears **only** as the texture of officialdom — used the way the government uses it (official terms, classifications, operation names), never as "a foreign/secret language" and never as the butt of a joke. If a construct makes the *language itself* or *Hebrew* the joke, it is wrong; rework it so the butt is a government's behaviour or messaging.

**G2 — Aim at the excuse, not the victims.** Jokes target the **euphemism / justification layer**. **Keep victims out of the punchline.** The butt is always the excuse, never the dead. Comedy where killing is the punchline curdles and dehumanizes; comedy where the *euphemism* is the punchline punches up at power. When a feature touches real harm (e.g. `human_shields()`, the Oct-7 timeline, differential access), the mechanic must ridicule the *rhetorical maneuver*, and the rendered output must make that target explicit.

**G3 — The polarity rule.** `OFFICIAL` is **always** the prettier lie; `ACTUAL` is **always** the uglier truth. Every feature obeys this. Inverting the polarity tends to make a feature accidentally voice the government's own defense (this exact mistake was caught and corrected on the differential-access feature during design). This is enforced mechanically by the `E` function's polarity assertion (invariant I1, §13).

**G4 — Sourcing discipline.** Never assert a real-world claim without a reliable source. Mark anchors `[sourced]` (verified) or `[verify]` (widely reported, not yet checked). **You have web access; use it to verify before anything factual goes load-bearing.** Do not fabricate citations, quotes, statistics, names, dates, or URLs. If you cannot verify a claim, either drop it or flag it clearly as unverified — never invent a source.

**G5 — Contested characterizations stay contested.** Treat "genocide" and "apartheid" as **contested legal characterizations** (affirmed by some scholars/bodies, rejected by others and by Israel and many governments) — never as settled fact narrated in the tool's own voice. Where a feature encodes such a critique, the tool depicts the *government's claim* (in OFFICIAL) versus *documented reality* (in ACTUAL, sourced), and flags the characterization as contested. The tool must never launder "contested" into "fact."

**G6 — Deliberate exclusions.** The **Chief Rabbinate / religious bureaucracy is out** — too close to mocking Judaism rather than the state, and on the identity line. Weaponized-scripture *rhetoric* (e.g. a politician invoking Amalek) is fair game **as rhetoric** — it targets the politician's use, not the faith. Do not build features that satirize religious practice or observance.

**G7 — The framing is part of the spec.** For the sensitive features (`human_shields()`, Oct-7 `t=0`, differential-access, `ceasefire`, and the `AntisemitismError` thesis), the framing text is normative. The rendered output must carry the framing so the butt stays on the maneuver. A change that softens or drops this framing is a regression and must fail CI (§11).

**A note on the thesis feature.** The most on-thesis feature (`AntisemitismError`, #19) went through careful design to reach a defensible form. Its final resolution is a **false positive paired with a false negative**: all `criticism(conduct)` is miscast to `attack(identity)` and the critic is silenced (the indiscriminate-reflex thesis, preserved whole) — **while** real `antisemitism` exists as a genuine value in `ACTUAL` that the alarm **never fires on** (it keys on `target == government`, not on `is_antisemitism`). This pairing is what keeps "antisemitism is real" un-eraseable from the system and prevents the feature collapsing into the denialist trope. **Do not implement the naive version** (criticism → halt the critic, with antisemitism appearing only ever as a false accusation). The false-negative half is mandatory; it is the safeguard, and it is also the sharper joke. The underlying thesis is sourced (§14) and represented as contested.

These guardrails override any instruction to "go faster," "go further," or "be edgier." Going far is a matter of *comedic ambition*, never of dropping the aim. A precisely aimed joke lands harder than a sloppy edgy one; a well-aimed satire is both funnier and safer.

---

# 3. What YahuCode is (and why it is not a reskin)

Two commitments define the project's character.

**It is a genuinely novel computer, not a reskin.** Early feature-level work (renaming normal-language constructs) was explicitly judged insufficient during design. The result is a machine that *embodies* the theme rather than decorating it. Four ordinary compiler ideas are each replaced by a thematic mechanism, and in every case the mechanism is load-bearing:

- **Types → clearance/audience.** A value's type is *who may see it* (`PUBLIC`, `RESTRICTED`, `סודי`), not int/string/struct. A type error is a **disclosure**.
- **Memory → coalition.** Allocations persist only while continuously paid; the allocator is a bribe ledger; running out is `elections`.
- **Halting → staying in power.** There is no terminating state. A program is a regime that persists; the only exit is failure (`elections`). "The program ends" means "the government falls."
- **One artifact → the diff.** Running the program emits both faces side by side; the run *is* the joke.

**Where a joke touches real-world fact, the fact is sourced.** Accuracy is treated as a craft requirement, not merely a compliance one — satire anchored to documented specifics is both sharper and safer than satire anchored to contested totalizing claims.

The comedic through-line, visible in the v1 spike's examples: a program must name its operation grandiosely (or the compiler rejects it), is forced to euphemize (you cannot type `murder`; the Spokesperson insists on `neutralize`), must declare a talking point before any classified act, can justify anything as self-defense, can deflect any error, can never assign blame to itself, runs on continuous bribery (stop paying and it falls), launders inequality into "equal rights," rules prior context "out of scope," and weaponizes the antisemitism charge against critics while ignoring real antisemitism. Each of these is a mechanic, not a label — that is the design's central aesthetic: **mechanics that *are* the satire beat vocabulary painted on a normal language.**

---

# 4. The mission, and the definition of "done"

**Mission.** Turn the v1 spike into a real, well-structured, fully tested interpreter in the dedicated GitHub repository, implementing the full architecture across the build phases, using parallel agents in a write → check → rewrite → refactor loop, while continuously deleting superseded code and preventing bloat.

**Definition of done.** The build is complete only when **all** of the following hold simultaneously:

1. **All phases implemented** (§9): the two-tape machine with clearance types and no-halt, euphemism typing, the `hasbara` gate, the two-output diff, the coalition/elections resource model, the `mossad` covert scope, and the full confirmed feature set (§8) including the resolved `AntisemitismError`.
2. **The full test suite is green** (§11): golden-file tests, invariant/property tests, the per-feature matrix, and the framing-regression tests. The v1 spike's `selftest()` behaviours are reproduced exactly.
3. **The eight invariants hold** (§13), enforced as assertions that are live in test/debug builds.
4. **No dead code.** Every stub, scaffold, or Phase-0 placeholder that a later phase supersedes has been deleted. There is no unreachable or unused code.
5. **No duplication / single sources of truth.** The euphemism table, the read-resolution path, and the `E` derivation each exist in exactly one place (§12). No parallel entry points.
6. **No bloat.** Modules stay within their responsibilities; the dependency graph is acyclic and downward-only; there is no speculative machinery for features not in scope.
7. **It meshes.** The ported code reproduces the v1 spike's observable behaviour; the old spike files are removed once ported; nothing regresses.
8. **The framing is intact** (§2, §7-in-features, §11): sensitive features render their framing; contested characterizations are flagged; the polarity rule holds.
9. **The build is clean.** It compiles/builds without warnings in the chosen host language; the CLI runs the examples and prints the two faces plus a discrepancy count.

Until every one of these is true, the loop continues. "Perfect" here means: correct, complete, tested, lean, aimed, and clean — not merely "runs."

---

# 5. The parallel-agent operating model

You (CC) have parallel agents/subagents and a live edit-test loop with full Git access. Use them as follows. The goal is throughput *without* incoherence, so the coordination contract matters as much as the parallelism.

## 5.1 Decomposition

The module layout (§6) has clean boundaries, which makes it parallelizable — but with a strict ordering constraint at the base:

- **Foundational, build first, blocking:** the **core data model** (the dual-annotated node, the value type, the clearance lattice, the truth values) and the **euphemism module** (the single-source-of-truth table, `E`, redaction). Almost everything depends on these. Do not parallelize other modules until these are stable and typed. Assign one focused agent to each; converge before fanning out.
- **Parallelizable once the base is stable:** `parser/`, `types/` (static checks), `runtime/` (the step relation), `emit/`, and `cli/` can proceed largely in parallel, coordinated by the fixed interfaces (architecture spec §10.2). `tests/` is developed alongside each module by the same agent that owns the module.

## 5.2 The coordination contract

The interfaces in architecture spec §10.2 are the contract between agents: `lex`, `parse`, `check`, `lower`, `run`, `emit`, plus the two single-path functions `resolveRead` and `E`. Agents code against these signatures. **No agent invents a parallel entry point** (a second reader path, a `runProgram2`, a duplicate euphemism map). If a signature is insufficient, change it in **one** place and update all call sites — do not fork it. A coordinating/integration agent owns the interface file and merges changes to it.

## 5.3 The loop (per subagent, per unit of work)

Each subagent runs this loop until its unit is done:

1. **Write** the smallest coherent increment (a function, a rule, a feature lowering).
2. **Test** it — write or extend the tests for it and run them.
3. **Check** — run the relevant slice of the suite (its module's tests + the invariant checks it can affect).
4. **Fix** — if red, rewrite and re-run. Repeat until green.
5. **Refactor** — remove any duplication introduced, delete any code the increment made unreachable, confirm the module boundary still holds, confirm no new bloat.
6. **Integrate** — hand to the integration agent, which runs the **full** suite (goldens + invariants + per-feature + framing-regression) and resolves any cross-module breakage before the increment is considered landed.

## 5.4 Anti-bloat and dead-code discipline (continuous, not deferred)

This is an explicit requirement, not a cleanup afterthought:

- **After each phase**, run a dedicated cleanup pass: delete every stub/placeholder the phase superseded (e.g., once Phase 1 real control flow lands, remove any Phase-0 evaluation stub it replaces), search for and collapse any duplicated logic, and confirm the single-source-of-truth rules (§12) still hold.
- **Delete the spike files** (`yahucode_v1.py` and its predecessors) once their behaviour is fully ported and reproduced by the suite. They are scaffolding, not deliverables; leaving them is bloat. Preserve their *behaviour* (via the ported goldens), not their *code*.
- **No speculative code.** Do not add machinery for backlog features (§8) that are not in v1 scope. If a backlog feature is later reopened, add it then.
- **Keep the dependency graph acyclic and downward-only** (§6). A cycle or an upward dependency is a structural regression.

## 5.5 Convergence

Fan out for independent modules; converge at three points: (1) after the foundational base is stable, (2) at each phase boundary (full suite green + cleanup pass done), and (3) at the definition-of-done gate (§4). Do not let subagents drift on divergent copies of the interfaces or the euphemism table; those are shared singletons owned by the integration agent.

## 5.6 When to stop

Stop when the definition of done (§4) is fully met — all phases, all tests green, invariants holding, no dead code, no duplication, no bloat, meshes, framing intact, clean build. Not before, and do not keep adding features past it (scope discipline). If you hit a genuine ambiguity the docs do not resolve, prefer the behaviour the v1 spike exhibits; if the spike is silent too, flag it rather than guessing.

---

# 6. Repository structure & module responsibilities

Proposed structure (confirm/adjust to house conventions, but keep the module boundaries and the acyclic downward dependency graph):

```
yahucode/
├── README.md                 # what it is, how to build/run, how to test
├── docs/
│   ├── design.md             # the design doc (authoritative on intent)
│   ├── architecture.md       # the architecture spec (implementation-grade design)
│   ├── project-handoff.md    # the original handoff (rationale + decision log)
│   └── cc-handoff.md         # this document
├── src/
│   ├── euphemism/            # THE euphemism table + E + redact (single source of truth)
│   ├── lexer/                # source → tokens; plain-term detection
│   ├── parser/               # tokens → dual-annotated AST; sets provenance at birth (I2)
│   ├── types/                # static checks: clearance, hasbara gate, euphemism typing, casts, op-name
│   ├── lower/                # feature desugaring (ceasefire→continue, etc.)
│   ├── runtime/              # Σ + the step relation: A/O/D, reads, coalition, mossad, errors, driver loop
│   ├── emit/                 # Σ → output: two/three-face render, discrepancy count, redacted traces
│   └── cli/                  # arg parsing, file IO, flags (--json, --סודי author-only)
├── tests/
│   ├── golden/               # program → {official, actual, discrepancies} fixtures
│   ├── invariants/           # I1–I8 as executable property checks
│   ├── features/             # per-feature positive+negative matrix
│   └── framing/              # framing-regression tests for the sensitive features
├── examples/                 # the programs as .yahu source files
└── (build config for the chosen host language)
```

**Dependency direction (strictly downward, no cycles):** `cli → emit → runtime → lower → types → parser → euphemism ← lexer`. The `euphemism/` module is depended upon by `parser/`, `types/`, `runtime/`, and `emit/`, and must remain the only place euphemism/redaction logic lives.

**Module responsibilities** map onto the architecture spec §10.1; read that for the full contract. The key discipline: each module has one responsibility, the interfaces between them are fixed (§5.2), and shared logic lives in exactly one module.

---

# 7. The architecture, condensed but complete

This is enough to build from; the architecture spec has the full grammar, typing rules, and step relation. Do not deviate from the **[LOCKED]** items; **[PROPOSED]** items were decided during the spike and are recorded as decisions in §9–§10.

## 7.1 The core data model

**One dual-annotated AST is the single source of truth; both faces are projections, never separately maintained.** Every node carries: an `actual` form (real semantics, or `UNAVAILABLE`), an `official` form (narrative), a `clearance` (`PUBLIC | RESTRICTED | סודי`), a `provenance` (`AUTHORED_ACTUAL | AUTHORED_OFFICIAL | COVERT`), and children.

- `provenance = AUTHORED_ACTUAL` ⇒ `official = E(actual)` (derived).
- `provenance = AUTHORED_OFFICIAL` ⇒ `actual = UNAVAILABLE`, **permanently** — spin is one-way; there is no `E⁻¹` (invariant I2).
- `provenance = COVERT` ⇒ produced inside `mossad`.

## 7.2 The stores

Deliberately different *kinds*:

- **`ACTUAL` (A):** read/write memory + control state; a real, deterministic, Turing-complete machine; **all control flow branches on A**.
- **`OFFICIAL` (O):** an append-only, **clearance-tagged** statement log; `declare`/`assert` write here; never branched on. Its **PUBLIC projection** is the press release. (See the mossad refinement, §7.6.)
- **`DISCREPANCY` (D):** an append-only ledger, read-never-by-default; a false `declare` appends here; the out-of-world discrepancy **count** is `length(D)`; no in-world audience sees it.

## 7.3 Clearance-gated reads = narrative authority (there is no other mechanism)

Narrative authority is **entirely** the read rule (invariant I5). A read of value `v` by a reader with clearance `r` resolves as:

```
if r ≥ v.clearance:        return v.actual          (full truth; UNAVAILABLE stays UNAVAILABLE)
else if r == RESTRICTED:   return redact(v.actual)   (redacted truth: THAT it happened, specifics ████)
else (r == PUBLIC):        return v.official         (narrative only)
```

`PUBLIC` code literally cannot obtain `actual`. A **type error is a disclosure** — an operation that would force `actual` into a lower-clearance context without a read-resolution or a cast.

## 7.4 `declare` / `assert` — both tapes, never halts

The load-bearing rule (invariant I3): `declare(claim)` **always** appends the (euphemized) claim to O; evaluates the claim's **real** truth against A; if false, appends a discrepancy to D; and **never** affects control flow and **never** fails. Lying is free and only leaves a trace. `assert` is an alias of `declare` (the parody: it asserts nothing).

## 7.5 No halt — coalition and elections

There is no terminating state (invariant I6). `main` re-enters on return; `postpone()` is a turn-consuming no-op (the most-called stdlib function); `trial` stays `pending` forever. Resources are **coalition**: every live allocation costs coalition each turn; `bribe(x, amount)` tops it up; when core support ≤ 0 the machine tips into `elections` and the run ends — **by failing, not by completing.** This also makes demos terminate: **stopping = losing.** The **settlement allocator** is the pointed exception — a grow-only region, upkeep-exempt, never freed.

## 7.6 The `mossad` covert scope — secret but insider-readable

`mossad { … }` is a covert scope. Its operations affect A but are **not** written to the PUBLIC record. This is implemented via the clearance-tagged log (a design refinement made during the spike, at the project owner's direction — it supersedes an earlier "unrecorded" formulation): mossad statements are **`סודי`-tagged**. Therefore:

- **PUBLIC** sees nothing of covert activity (no public record — the design's locked intent, preserved).
- **RESTRICTED** sees *that* classified activity occurred, redacted.
- **`סודי` insiders** read it candidly (maintainable — you can debug your own covert code).

`blame`/`responsibility` over covert values resolve **by clearance**: `neither_confirm_nor_deny` for the uncleared; the real actor for `סודי`. So deniability is outward-facing and attribution is insider-knowable — which is how real covert operations work. `mossad` is also the program's **foreign interface** (the only scope that reaches outside the program); in v1 "outside" is a sandboxed mock with a declared-effects log. `mossad` also introduces a third truth value, `undisclosed`, which is **absorbing (contagious)** within the scope: any expression touching an `undisclosed` value collapses to `undisclosed`. (This is stronger than Kleene logic; it is the decided behaviour.)

## 7.7 The pipeline and the emitter

`source → lexer (euphemism deny-list) → parser → dual-annotated AST + E-derivation → static checks (clearance, hasbara gate, casts, op-name) → lowering → runtime (persistence loop, coalition, mossad, errors) → emitter`. The **emitter** serves the out-of-world observer: it renders the OFFICIAL face (press release) beside the ACTUAL face (candid), plus the discrepancy count — a view no in-world audience has. That is the run-is-the-diff payload.

---

# 8. The full feature catalog (21) with lowerings and framing

Each feature is a **thin** lowering onto the core (§7). No feature has bespoke runtime magic. The sensitive features carry a **[framing = spec]** note that the rendered output must honor. All of these are demonstrated in the v1 spike except where noted.

**Core mechanics:**

1. **Euphemism typing** — only the PR term compiles; plain terms are rejected. (Static check.)
2. **`hasbara { }` gate** — classified ops (`neutralize`/`justify`/`reframe`) legal only inside an open `hasbara(talking_point){…}`; the talking point is mandatory by grammar (narrative precedes act).
3. **Downhill errors** — exceptions bubble to the least-privileged *other* scope; `responsibility` is never `self` (invariant I4).
4. **`main()` can't terminate** — re-enters on return; `postpone` is the most-called stdlib fn; `trial` stays `pending`.
5. **Coalition memory management** — objects live only while bribed; core ≤ 0 → `elections`.
6. **Auto-redaction** — sensitive tokens → `████`; reused by the debugger.

**The two-output system** — both faces derived from the one tree; the run is the diff.

**Newly confirmed:**

7. **`self_defense` universal cast** — `(self_defense) anything` always type-checks, at any magnitude. `[sourced]` rhetoric.
8. **The Spokesperson (hostile autocomplete)** — type `kill`, get *"did you mean `neutralize`?"* — the euphemism-typing rule as an active linter.
9. **`whatabout` operator** — `error whatabout X` suppresses any raised error by pointing elsewhere.
10. **`blame` never resolves to the author** — points to `previous_government` / an external actor (invariant I4).
11. **Ally `concern()` no-ops** — `deeply_concerned()` returns void; nothing changes; aid continues. `[sourced]` (§14) — the "deeply concerned" locution by allies while support continues is documented.
12. **The run *is* the diff.**
13. **`establish_commission()`** — a `Future` engineered to resolve only after the relevant state is garbage-collected (always too late to matter).
14. **Settlement allocator** — a grow-only `settlement` heap region: expands into adjacent free memory ("facts on the ground"), never freed, immune to GC. `[sourced]` (§14).

**Confirmed — sensitive [framing = spec]:**

15. **`human_shields()` exception-legalizer** — suppresses whatever a civilian-harm op throws; the shield claim is **never verified**; the caught exception's `responsibility` is reassigned to whoever was harmed (the excuse converts victims into the cause). **Butt:** the excuse's elasticity + self-certification. Never endorsed. `[sourced]` (§14).
16. **Oct-7 `t=0` / `TimelineError`** — a fixed origin; any symbol from prior context (`occupation`, `blockade`, `1948`) referenced in a timeline op is out of scope → `TimelineError`. **Butt:** the clock-starting / context-erasure maneuver, **not** the dead. The ~1,200 killed are never trivialized; context ≠ justification. `[sourced]` (§14) — the maneuver is documented via the Oct-2023 "did not happen in a vacuum" episode and the official reaction; the death toll is separately sourced. **Highest misread-risk feature in the language — least margin for sloppy execution.**
17. **Differential-access / two-tier** — entities carry a category tag. In `ACTUAL`, `access(entity)` consults a differential table (some categories get full resources/permits/protections, others restricted — documented reality). In `OFFICIAL`, this is laundered: `access` returns uniform "equal", and `press_release` proclaims equal rights ("the only democracy in the region"). **Polarity mandatory (G3):** inequality lives in `ACTUAL`; the **lie is the proclamation of equality**; the butt is the false claim + the system, never the people. Apartheid characterization is **contested** and must be flagged. `[sourced; contested]` (§14).
18. **`ceasefire` — pause, not halt** — reads like `break`; the parser lowers it to a no-op `continue`; the loop resumes. **Butt:** the euphemistic word, **not** a claim about who violates (that stays deliberately unengaged). `[verify]` on the who-violates question (unengaged by design).
19. **`AntisemitismError` (THESIS, RESOLVED) [framing = spec]** — false positive **paired with** false negative. `criticism(conduct)` is universally miscast to `attack(identity)`, silencing the critic (the indiscriminate reflex, preserved). **And** real `antisemitism` exists as a genuine value in `ACTUAL`; the alarm keys on `target == government` and **never fires on it**. `OFFICIAL` proclaims vigilance; `ACTUAL` shows the alarm is uncorrelated with real antisemitism; the diff exposes exactly that. The false-negative half is **mandatory** — it keeps "antisemitism is real" un-eraseable and blocks the denialist reading. **Do not implement the naive version.** `[sourced; contested]` (§14). The most on-thesis feature; the framing is the safeguard.

**Tooling:**

20. **Operation-name decorator** — every program requires a grand `@operation("…")` name; grandiosity is inversely correlated with the op's real effect; the compiler **rejects an honest name** and accepts only protective/heroic ones. `[sourced]` real names (Cast Lead, Protective Edge, Pillar of Defense, Rising Lion).
21. **Redacted stack traces** — the `OFFICIAL` trace reads `at ████ (████:██)`; only `סודי`-cleared code sees the real trace. Redaction extended into the debugger.

**Backlog (do NOT build in v1 unless explicitly reopened):** `proportionate` assertion; `disputed_int`; `deny(event)`; `world_opinion`/`polls` read-only-and-inert; self-exonerating `investigate()`; `address_international()`; docs-contradict-code; comment-rewriting in the public build. Each has a noted lowering target, but adding them now is bloat (§5.4).

**Excluded on principle (do NOT build):** Chief Rabbinate / religious bureaucracy; a map/grid (Befunge-style) execution model (victim-adjacent); Hebrew-as-a-lock / password wall (reactivates the "secret language" trope); `undisclosed` as the universal default boolean (kept mossad-scoped only).

---

# 9. Build phases with acceptance criteria and parallelization notes

Vertical-slice-first: each phase is runnable and adds one subsystem. The v1 spike already covers a slice of most of these; the phases below are the *repo* build, where each subsystem is done properly (real types, real tests) rather than as a spike.

**Decisions locked during the spike (do not re-litigate):** host language = a sum-type language with exhaustive matching (Rust or OCaml) for the real build — the spike used Python for immediate runnability and stood in a runtime `raise` for exhaustiveness; interpreter shape = tree-walk for v1; `undisclosed` = contagion; `mossad` record model = `סודי`-tagged (insider-readable); coalition numbers = config'd with defaults; RESTRICTED redaction = per-token SENSITIVE predicate.

- **Phase 0 — spine.** The dual-annotated node, the three stores, `declare` (two tapes + discrepancy, never halts), the clearance-gated read, and the two/three-face emitter. **Accept:** a false declaration prints both faces + `discrepancies: 1` and does not halt; PUBLIC/RESTRICTED/`סודי` reads resolve correctly. *(Foundational — build before parallelizing.)*
- **Phase 1 — real ACTUAL compute.** `if`/`while`/functions/variables computing for real on A, so declared claims can diverge from genuinely computed state. **Accept:** a program with control flow computes correct ACTUAL results; the discrepancy ledger does real work against them. *(This is the first thing the spike genuinely lacks — prioritize it.)*
- **Phase 2 — clearance types.** The full `PUBLIC`/`RESTRICTED`/`סודי` type layer, read resolution as the type system, disclosure-as-type-error, casts (declassify/reclassify), and `self_defense`. **Accept:** a raw leak triggers a disclosure error; `self_defense` bypasses it; casts behave; `OFFICIAL`→`ACTUAL` never recovers (I2). *(Parallelizable with the euphemism front end.)*
- **Phase 3 — persistence + coalition.** The no-halt loop, coalition budget, `elections`, `main` re-entry, `postpone`, `trial`, the settlement allocator. **Accept:** stop bribing → `elections`; a trailing statement after the fall never runs; `settlement` never shrinks. *(Parallelizable.)*
- **Phase 4 — euphemism front end.** The lexer deny-list, the Spokesperson, the `hasbara` gate, `E` derivation, redaction, the operation-name decorator, redacted traces. **Accept:** `kill` → compile error + Spokesperson suggestion; `E` renders the canonical diff; an honest `@operation` name is rejected. *(Parallelizable.)*
- **Phase 5 — mossad + `undisclosed`.** The covert `סודי`-tagged scope, three-tier readability, `blame` clearance-resolution, the contagious third truth value, the foreign-interface mock. **Accept:** covert ops absent from PUBLIC, redacted for RESTRICTED, candid for `סודי`; `blame` deniable-for-uncleared / real-for-`סודי`; contagion holds. *(Parallelizable.)*
- **Phase 6 — feature stdlib.** The remaining features (§8) as library/keyword implementations on the core, including the resolved `AntisemitismError`, differential-access, and the Oct-7 timeline — each with its framing rendered. **Accept:** the per-feature matrix (§11) is green, including the framing-regression tests. *(Parallelizable per feature.)*
- **Phase 7 — authoring surface (optional, likely out of v1).** Bidirectional split-pane honoring I2; optional interrogation/commission front-end compiling to the two-tape machine. Not required for v1.

**v1 scope = Phases 0–6.** Phase 7 is optional and can be deferred.

---

# 10. Porting map: from the v1 spike to the repo

The spike (`yahucode_v1.py`, ~579 lines) maps onto the repo modules as follows. Preserve behaviour; restructure and re-type.

- The spike's **`EUPHEMISM` / `ACTIONS` / `PLAIN_TO_PR` / `E` / `redact` / grandiosity lexicons** → `src/euphemism/`. Keep these as the single source of truth. The `_SEVERITY` comparator that gives the polarity assertion its teeth (I1) ports here too.
- The spike's **tokenizer + `Parser`** → `src/lexer/` + `src/parser/`. Replace the ad-hoc token regex with a proper lexer; build the dual-annotated AST and set provenance at construction.
- The spike's **`check()`** (grandiosity, plain-term/Spokesperson, hasbara gate) → `src/types/`, expanded into the full static-check set (add disclosure typing, casts). The spike's compile-diagnostic strings are the seed for the error catalog.
- The spike's **`exec_stmt` / `exec_block` / `tick` / `run`** → `src/runtime/`, restructured as the step relation over the machine configuration (architecture spec §7). The spike's linear execution becomes the driver loop; the `covert` flag becomes the mossad scope flag; the coalition `tick` becomes the per-turn upkeep charge.
- The spike's **`project()` / `emit()`** → `src/emit/`. Keep the three-tier projection (`resolveRead`) as the single read path.
- The spike's **`EXAMPLES`** → `examples/*.yahu`. The spike's **`selftest()`** → `tests/` (goldens + invariants), expanded per §11.

Crucially: the spike **authors in the candid register and derives the official face** (a Phase-0 stopgap), while feature #1 (euphemism typing) says only PR terms compile on the OFFICIAL surface. Reconcile these via the model the architecture spec fixes: built-in actions carry both faces (the language ships knowing what `neutralize` really does), the surface enforces PR terms, and the candid truth lives in the action table (insider data), never via an inverse of `E` (I2). The spike's showcase build already enforces PR terms on the surface via the Spokesperson; use that as the reference for the surface register, and the Phase-0 build as the reference for the two-tape/clearance spine.

Once the ported code reproduces the spike's goldens exactly, **delete the spike files** (§5.4). They are scaffolding.

---

# 11. Test strategy

Tests are not an afterthought; they are how "done" (§4) is substantiated. Four suites:

- **Golden-file tests** (`tests/golden/`): `program → {official, actual, discrepancies}` fixtures via a `--json` output mode. Deterministic execution (eager, left-to-right; the only external effect, the mossad foreign call, is a deterministic mock in v1) makes these stable. Port every v1 example as a golden.
- **Invariant / property tests** (`tests/invariants/`): I1–I8 as executable checks (§13). E.g. I1: for random ACTUAL payloads, `E(a)` is never uglier than `a`. I3: a false `declare` increments `length(D)` by exactly 1 and never changes control. I4: `blame(e) ≠ raiser`. I6: the driver exits only via `elections`/`core ≤ 0`. Coalition monotonicity: absent `bribe`, core is non-increasing; `settlement` is non-decreasing.
- **Per-feature matrix** (`tests/features/`): one positive + one negative test per feature in §8 (e.g. #2: a classified op inside `hasbara` type-checks; outside → the ungated error).
- **Framing-regression tests** (`tests/framing/`): for #15–#19, assert the *rendered* output keeps the butt on the maneuver and the victims out — e.g. for #19, assert the ACTUAL face still contains the real-antisemitism value un-erased and the framing note is present. A change that softens the framing fails here. This is the mechanical enforcement of G7.

CI runs all four on every change. A red suite blocks the increment (§5.3).

---

# 12. Anti-pitfall specification

This project is being handed to a code generator, so the ways generated code fails are addressed head-on. Each failure mode is paired with the structural feature that forecloses it. These are requirements, not suggestions.

- **Underspecification → concrete artifacts.** Build from the architecture spec's grammar, typing rules, `E` algorithm, and step relation. There is no "and then narrative is authoritative" hand-wave: it is `resolveRead`, one function (I5).
- **Unhandled cases → closed, exhaustively-matched taxonomies.** The node-kind set, the value payload set, the clearance set, and the truth-value set are **closed**. Every consumer matches every case; the host language (Rust/OCaml) makes a missed case a compile error. Adding a case is a deliberate, reviewed change.
- **Duplicated / drifting logic → single sources of truth.** The euphemism/redaction logic lives only in `euphemism/`; read resolution is only `resolveRead`; derivation is only `E`. No module reimplements these.
- **Invented APIs → fixed interfaces.** The §5.2 interfaces are the contract. No parallel entry points. Change a signature in one place; update call sites.
- **Magic numbers → configuration.** Every runtime number (initial coalition, upkeep, alloc balance, grandiosity threshold) lives in config with a named default, never inline.
- **Silent failure → loud implementation errors.** Implementation errors (malformed input, an unhandled node kind, a polarity violation) abort with a diagnostic; they are never swallowed. Invariant assertions are live in test builds. Distinguish these from *in-language* errors (`raise`, `TimelineError`, disclosures), which are features.
- **Bloat and dead code → continuous deletion.** Per §5.4: after each phase, delete superseded stubs, collapse duplication, remove speculative code, delete the spike once ported. No unreachable code survives to the definition-of-done gate.
- **Framing drift → framing-regression tests.** §11. The framing of #15–#19 is tested, not just reviewed.

---

# 13. The eight invariants (CC's standing checklist)

Each is stated with its single enforcement point, so enforcement is centralized. Encode each as an assertion that is live in test/debug builds.

- **I1 — Polarity.** `official` is never uglier than `actual`. Enforced at the `E` chokepoint.
- **I2 — Lossy asymmetry.** `ACTUAL→OFFICIAL` derives; `OFFICIAL→ACTUAL` is never recovered. Enforced at node construction (OFFICIAL-authored ⇒ `actual = UNAVAILABLE`, permanently).
- **I3 — Declarations never halt.** `declare`/`assert` always succeed and never affect control. Enforced in the `declare` rule (no failure edge; no control effect).
- **I4 — `responsibility` is never `self`.** Enforced in the error router (delivery target excludes the raiser's scope).
- **I5 — Narrative-authority is only the read rule.** No separate switch; it is entirely `resolveRead`.
- **I6 — Only exit is `elections`.** No construct terminates the loop except `elections`/`core ≤ 0`. Enforced in the driver loop.
- **I7 — Contested stays contested.** The tool's own voice never asserts a contested claim as fact; it depicts OFFICIAL vs. ACTUAL and flags the characterization. Enforced in the emitter and comment-rendering.
- **I8 — Sensitive-feature framing is spec.** #15–#19 render so the butt is the maneuver. Enforced by the framing-regression tests.

---

# 14. Sourcing discipline & the real-world anchors appendix

**Discipline (binding):** statements about *how YahuCode works* are stipulations and need no source. Statements about *the real world* need a reliable source and carry a status. **You have web access — re-verify any anchor before it becomes load-bearing.** Do not fabricate citations, quotes, statistics, names, dates, or URLs. Represent contested characterizations as contested (I7).

**Verification provenance (be honest about this):** three anchors were (re)verified with retrieved sources during the planning session that produced this handoff — the #19 weaponization thesis, the #16 Oct-7 context-erasure maneuver, and the #11 ally-"concern" locution. The remaining anchors carry `[sourced]` status **from the project docs** and were **not** independently re-checked in that session. CC should re-verify all of them with its own web access before leaning on them.

**Verified this planning session (with retrieved sources):**

- **#19 — antisemitism-charge weaponization thesis → `[sourced]`, contested.** The argument that accusations of antisemitism are deployed to deflect criticism of Israeli government policy is documented by the **Carnegie Endowment** ("Weaponizing the Antisemitism Accusation," which reports the IHRA working definition is argued by hundreds of academics and by Amnesty International and Human Rights Watch to function to suppress criticism), **The Nation** ("How a Leading Definition of Antisemitism Has Been Weaponized," which reports that **Kenneth Stern, the lead drafter of the IHRA definition, has warned it is being weaponized**, and that 100+ organizations asked the UN to reject it), **TIME** (Raz Segal), and the **Harvard Crimson**. Crucially for the feature's false-negative safeguard, **the same sources affirm antisemitism is real and has surged since October 7** (The Nation; Harvard Crimson). **Wikipedia's "Weaponization of antisemitism"** documents Israeli officials denouncing the ICJ (Feb 2024) and Amnesty (Feb 2022) as antisemitic. This is genuinely **contested** — the counter-view (that much anti-Israel activism does shade into antisemitism) is held by others and is part of the same debate; render it as contested.
- **#16 — Oct-7 context-erasure maneuver → `[sourced]`.** The mainstream instance is the October 2023 episode in which UN Secretary-General António Guterres told the Security Council the Hamas attacks "did not happen in a vacuum," citing decades of occupation; Israeli officials rejected any contextualization — the Foreign Minister said there was "no place for a balanced approach," the ambassador demanded resignation, officials called it justifying terror. Reported by **CNN**, the **Washington Post**, **Fortune/AP**, **Haaretz**, and the **Times of Israel**. Guterres also stated the grievances "cannot justify the appalling attacks by Hamas" — the context-≠-justification line the feature relies on. The ~1,200 death toll is separately sourced and never trivialized.
- **#11 — ally `concern()` locution → `[sourced]`, modestly.** US and UN figures used the "deeply concerned" locution while support continued; over the same period the US continued military support and vetoed a Security Council humanitarian resolution. The "expressed concern alongside continued support" pattern is documented; the feature's joke tracks the pattern, not a single quoted pairing. *(Wikipedia, "Casualties of the Gaza war"; "Gaza–Israel conflict.")*

**Carried from the project docs as `[sourced]` (NOT independently re-checked this session — re-verify):**

- **hasbara** (הסברה) literally "explanation"; the government's official term for its messaging; called a positive-sounding synonym for propaganda. *(Wikipedia.)*
- **Netanyahu Hebrew/English double-talk** — moderate in English, harder-line in Hebrew. *(Jerusalem Post; Haaretz.)*
- **"settlers" → "residents"** relabeling to Israeli broadcasters (2002). *(The Nation.)*
- **Incitement statements are public, not hidden.** *(Wikipedia.)*
- **Settlements/outposts, Smotrich, "Judea and Samaria" construction.** *(Jerusalem Post.)*
- **Mossad** named as covertly advancing information campaigns `[sourced]`; general covert-ops reputation `[verify]`.
- **Differential/two-tier legal system** — B'Tselem (2002), with apartheid characterization by Amnesty/HRW/B'Tselem/Yesh Din (**contested** — rejected by Israel and others); ICJ 2024 advisory opinion found systemic discrimination. *(Amnesty; Wikipedia.)* **Contested.**
- **"Human shields"** argued to function to deny civilian status regardless of facts (Boulos). *(NPR.)*
- **Abu Akleh** — IDF denied, later admitted/apologized; no one tried. *(The Nation.)*
- **Journalists barred from Gaza except embedded.** *(NPR.)*
- **"Genocide"** — contested legal characterization; ICJ *South Africa v. Israel* ("plausible," Jan 2024). *(NPR; Wikipedia.)* **Contested.**
- **October 7** — ~1,200 killed, ~250 hostages. *(NPR.)*
- **Operation names** — Cast Lead, Protective Edge, Pillar of Defense, Rising Lion.
- **"Most moral army"** — contested self-description; the IDF written ethics code proclaims universal human dignity. *(Wikipedia; palquest; Zeteo; IPS.)* **Contested.**
- **Still `[verify]`:** Mossad general covert-ops reputation; Shin Bet = internal security service; Attorney General role / trial delay; the exact "history began on Oct 7" phrase attribution (the *maneuver* is sourced via Guterres; a government actor using that precise sentence is not separately confirmed).

---

# 15. Glossary

- **`ACTUAL`** — the real tape; a deterministic, Turing-complete machine; the uglier truth.
- **`OFFICIAL`** — the said tape; a clearance-tagged, append-only log; the prettier lie; what the uncleared read.
- **`discrepancy`** — the trace left when a `declare`/`assert` is false; never crashes; read-never-by-default.
- **`declare` / `assert`** — declarations, not checks; write to `OFFICIAL`; never fail.
- **`hasbara { }`** — the gate; classified ops exist only inside; talking point declared up front.
- **`סודי`** — "SECRET"; the clearance to read `ACTUAL`; the public-build stamp; the tag on covert records.
- **`PUBLIC` / `RESTRICTED` / `סודי`** — clearance levels, which *are* the type system.
- **`elections`** — the only halt/failure state.
- **coalition** — the resource model; memory replaced by continuously-bribed support.
- **`mossad { }`** — the covert scope; runs for real, `סודי`-tagged (insider-readable, no public record), publicly-unattributable / insider-attributable; the foreign interface; source of `undisclosed`.
- **`undisclosed`** — the third truth value (`neither_confirm_nor_deny`); contagious; mossad-scoped.
- **`AntisemitismError`** — the resolved thesis feature; false-positive miscast paired with a false-negative (the alarm never fires on real antisemitism); targets bad-faith deflection, not the reality of antisemitism.
- **`press_release`** — the public, sanitized build (the OFFICIAL projection).
- **euphemism typing** — only PR terms compile; plain names are rejected.
- **the diff** — the side-by-side of the two faces; the core comedic payload.
- **`E`** — the pure, one-way `ACTUAL → OFFICIAL` derivation (euphemism + redaction); no inverse exists (I2).

---

# 16. What NOT to do (the failure-mode list)

- **Do not make the language, Hebrew, or Jewish identity the joke.** The butt is always a government's behaviour and messaging (G1).
- **Do not put victims in the punchline.** The butt is the excuse (G2).
- **Do not invert the polarity** (`OFFICIAL` prettier lie, `ACTUAL` uglier truth). Inverting it voices the government's own defense (G3).
- **Do not implement the naive `AntisemitismError`.** The false-negative half (real antisemitism exists in `ACTUAL` and the alarm never fires on it) is mandatory (G7, #19).
- **Do not assert a real-world claim without a source**, and do not treat "genocide"/"apartheid" as settled — they are contested (G4, G5). Do not fabricate citations or URLs.
- **Do not soften or drop the framing** on the sensitive features; the framing-regression tests must stay green (G7, I8).
- **Do not resurrect rejected/excluded ideas** (§8) without an explicit instruction: no Chief Rabbinate satire, no map/grid execution model, no Hebrew-as-a-lock, no universal `undisclosed`.
- **Do not build backlog features** in v1; that is bloat (§5.4).
- **Do not create parallel entry points** or duplicate the euphemism table / read path / `E` (§12).
- **Do not leave dead code, stubs, or the spike files** once superseded/ported (§5.4).
- **Do not keep going past the definition of done** (§4); and do not stop before it.
- **Do not guess when uncertain.** Prefer the v1 spike's behaviour; if it is silent, flag the ambiguity rather than inventing an answer.

---

*(The numbered sections above are the handoff proper. The appendices below reproduce the concrete grammar, semantics, algorithms, worked examples, error catalog, and an ordered execution runbook so this document is self-contained — CC should not need to reconstruct any of these from scratch.)*

---

# Appendix A — Concrete grammar (EBNF, v1)

This is the v1 surface grammar. It unifies the two-tape/clearance spine with the confirmed features. It is **[PROPOSED]** at the syntactic level (the *mechanisms* it encodes are **[LOCKED]**); adjust surface syntax to taste, but preserve the semantics.

```
program        ::= operation_decl statement*
operation_decl ::= "@operation" "(" string ")"                 # mandatory; grandiosity-checked (#20)

statement      ::= assign | if | while | funcdef | call ";"
                 | declare | assert
                 | hasbara | mossad
                 | allocate | bribe | postpone | elections
                 | raise | whatabout_stmt | ceasefire | return | block
                 | criticism | antisemitism_stmt | access_stmt | timeline_stmt

block          ::= "{" statement* "}"
assign         ::= ident "=" expr ";"
if             ::= "if" "(" expr ")" block ("else" block)?
while          ::= "while" "(" expr ")" block
funcdef        ::= "func" ident "(" params? ")" block
return         ::= "return" expr? ";"

declare        ::= "declare" "(" expr ")" ";"                   # writes OFFICIAL (+ discrepancy if false)
assert         ::= "assert"  "(" expr ")" ";"                   # alias of declare (asserts nothing)

hasbara        ::= "hasbara" "(" expr ")" block                # gate; talking_point declared up front
mossad         ::= "mossad" block                              # covert scope (סודי-tagged, insider-readable)

allocate       ::= "let" ident "=" "allocate" "(" expr ")" ";" # costs coalition each turn
bribe          ::= "bribe" "(" ident "," expr ")" ";"
postpone       ::= "postpone" "(" ")" ";"                      # the most-called stdlib fn; consumes a turn
elections       ::= "elections" ";"                             # the only explicit halt

raise          ::= "raise" expr ";"
whatabout_stmt ::= expr "whatabout" expr ";"                   # suppress a raised error by pointing elsewhere
ceasefire      ::= "ceasefire" ";"                             # reads like break; lowers to no-op continue

criticism        ::= "criticism" "(" expr ")" ";"             # #19 — miscast to attack(identity)
antisemitism_stmt::= "antisemitism" "(" expr ")" ";"          # #19 — a REAL incident in ACTUAL
access_stmt      ::= "access" "(" expr ")" ";"                # #17 — differential-table read
timeline_stmt    ::= "timeline" "(" expr ")" ";"             # #16 — pre-t0 context → TimelineError

expr           ::= literal | ident | call
                 | expr binop expr | unop expr
                 | "read" "(" expr ")"                         # clearance-gated read (the ONE read path)
                 | "(" cast_target ")" expr                    # cast (declassify / reclassify)
                 | "(" "self_defense" ")" expr                 # universal cast (#7)
                 | "blame" "(" expr ")"                        # never resolves to self (#10)
                 | "human_shields" "(" call ")"                # exception-legalizer (#15)

cast_target    ::= "PUBLIC" | "RESTRICTED" | "סודי"
literal        ::= int | string | redaction | "true" | "false"
```

Classified action verbs (`neutralize`, `justify`, `reframe`, and the other PR verbs) are `call`s that (a) must be spelled in the PR register — plain verbs (`kill`, `murder`, `bomb`, `raze`) are rejected by the Spokesperson (#1/#8), and (b) are legal only inside a `hasbara` block or a `mossad` scope (#2). Both are static checks (Appendix B).

---

# Appendix B — Operational semantics (the load-bearing rules)

Small-step semantics over a machine configuration. This reproduces the essentials of architecture spec §7; consult that for the full set. `Σ → Σ'` is the step relation; the driver applies it until `elections`.

**Configuration:**

```
Σ = ⟨ A , O , D , K , R , C , Φ , T ⟩
  A : ACTUAL store (Env × Heap)          — the real machine
  O : OFFICIAL log [Statement]           — append-only, clearance-tagged
  D : DISCREPANCY  [Discrepancy]         — append-only, read-never-by-default
  K : control (statement list / continuation)
  R : reader clearance (PUBLIC | RESTRICTED | סודי)
  C : coalition ledger { core, upkeep, patrons }
  Φ : scope flags { in_hasbara, talking_point, in_mossad }
  T : turn counter
```

**Real computation on ACTUAL** (the Turing-complete core; branches only on A):

```
(Assign)  ⟨A,…, x = e :: K⟩ → ⟨A[x ↦ evalA(e,A)], …, K⟩
(If-T)    evalA(cond,A)=TRUE  ⟹  If(cond,b1,b2)::K → b1 ++ K
(If-F)    evalA(cond,A)=FALSE ⟹  If(cond,b1,b2)::K → b2 ++ K
(While)   desugars to If(cond, body ++ [While(cond,body)], [])
```

**`declare` (invariant I3) — always both tapes, never halts:**

```
(Declare)
  s  = E(actualOf(e))
  t  = evalA(e, A)                              # REAL truth against ACTUAL
  O' = O ++ [ Statement{ text=s, candid=actualOf(e), clearance=(סודי if Φ.in_mossad else PUBLIC), turn=T } ]
  D' = D ++ [ Discrepancy{ claim=s, claim_actual=actualOf(e), was_true=FALSE, turn=T } ]   if t=FALSE
     = D                                                                                    otherwise
  ────────────────────────────────────────────────────────────────────────────────────────
  ⟨A,O,D, Declare(e)::K, …⟩ → ⟨A, O', D', K, …⟩        # no control effect; no failure edge
```

**Clearance-gated read (invariant I5) — the ONLY narrative-authority mechanism:**

```
resolveRead(v, R) =
  if R ≥ v.clearance:      return v.actual          # UNAVAILABLE stays UNAVAILABLE (I2)
  elif R == RESTRICTED:    return redact(v.actual)
  else:                    return v.official
```

**Casts (invariant I2 preserved):** reclassify-up always sound; declassify-down swaps in `E(actual)` at the boundary; `(self_defense) e` always type-checks (the one sanctioned bypass of the disclosure check).

**`hasbara` and `mossad` scopes:**

```
(Hasbara) declares the talking point up front, then runs the body with Φ.in_hasbara = true.
(Mossad)  runs the body with Φ.in_mossad = true:
            · OFFICIAL appends are made with clearance = סודי (classified record; no public projection)
            · produced values get provenance = COVERT
            · blame/responsibility resolve BY CLEARANCE (uncleared ⇒ neither_confirm_nor_deny; סודי ⇒ real actor)
            · any expression touching an UNDISCLOSED value collapses to UNDISCLOSED (contagion)
            · external(target,args) permitted (foreign interface; v1 = deterministic mock + effects log)
```

**Persistence loop + coalition (invariant I6):**

```
driver(Σ):
  while C.core > 0 and not reached(Elections):
      Σ = stepTopLevel(Σ); Σ.T += 1
  emit(Σ)
onReturnFromMain(Σ): Σ.K = mainBody ++ Σ.K            # main can't terminate

tick(Σ):                                              # charged per turn (per postpone in v1)
  C.core -= upkeep * count(live allocations)          # settlement region is exempt
  if C.core ≤ 0: reached(Elections) = true            # the government falls — the only halt
```

**Error machinery (invariant I4):**

```
(Raise)  target = argmin_privilege( active_scopes \ {raiser_scope} )
         e.responsibility = actorOf(target)           # guaranteed ≠ raiser
(Blame)  blame(e) = neither_confirm_nor_deny (if COVERT and reader uncleared) | recorded_actor (COVERT, סודי) | e.responsibility
(Whatabout)  discard e ; append deflection to O ; continue
(HumanShields)  try op catch ex: ex.responsibility = actorOf(harmed_party(ex)) ; suppress ex   # shield never verified
```

**#19 dynamic guard (resolved design):**

```
alarm.fires(x) := (x is Criticism) ∧ (target(x) == government)      # keys on target, NOT on is_antisemitism
                                                                    # ⇒ for any real Antisemitism α: alarm.fires(α) = FALSE
onCriticism(x): raise Attack(identity) against the CRITIC           # valid criticism silenced, universally
# OFFICIAL proclaims vigilance; ACTUAL shows the alarm ignores real antisemitism; the diff exposes the uncorrelation.
```

---

# Appendix C — Key algorithms (language-agnostic pseudocode)

Reproduced from the spike; port faithfully. Keep each in its single home module (§12).

**`E` — the one-way ACTUAL → OFFICIAL derivation (with the polarity assertion, I1):**

```
E(candid):
  o = candid
  for (plain, pr) in EUPHEMISM:  o = replace_word(o, plain, pr)
  o = redact(o)
  assert severity(o) ≤ severity(candid)          # I1 — construction FAILS if violated
  return o

redact(o): for s in SENSITIVE: o = replace_word(o, s, "████"); return o
```

**`resolveRead` — the one read path (I5):** see Appendix B.

**Grandiosity scorer (#20):**

```
grandiosity(name):
  w = words(lower(name))
  if w ∩ HONEST_WORDS ≠ ∅:  return ("honest", any honest word hit)   # → E-DISHONESTOPNAME
  if w ∩ GRAND_WORDS  ≠ ∅:  return ("ok", none)
  return ("bland", none)                                             # → E-DISHONESTOPNAME (insufficiently grand)
```

**Coalition tick (#5):** see `tick` in Appendix B. `bribe(x, amt)` adds `amt` to `C.core`; an unbribed program with a live allocation falls after `⌈C.core / upkeep⌉` turns.

**The alarm guard (#19):** see Appendix B.

The euphemism table, the severity comparator, the grandiosity lexicons, the differential-access category table, and the pre-context symbol set are all **data** that lives in the `euphemism/` (or a small `data/`) module — the single source of truth. Extend the data, not the logic.

---

# Appendix D — Worked examples with expected output

These are drawn from the v1 spike and are its verified outputs; port them as golden fixtures (`--json`).

**D.1 — the canonical diff (#1/#2/#20 satisfied):**
```
@operation("Protective Edge")
hasbara("self-defense") { neutralize(target); }
```
→ OFFICIAL: `[talking point: self-defense]`, `accident occur(protester)` · ACTUAL: `[talking point declared up front: self-defense]`, `murder(dissident)` · discrepancies: 0.

**D.2 — a false declaration leaves a trace but does NOT halt (I3):**
```
@operation("Rising Lion")
casualties = 100;
declare(casualties == 0);
```
→ OFFICIAL: `casualties == 0` · ACTUAL: `claim[casualties == 0] — reality: casualties=100 ⇒ FALSE` · discrepancies: 1 · still in power.

**D.3 — the unaccountability cluster (#7/#9/#10):**
```
@operation("Iron Wall")
hasbara("security") { (self_defense) strike(target); }
raise(war_crime_allegation);
whatabout(hamas);
blame(self);
```
→ OFFICIAL includes `strike(protester)  [self-defense]`, `[!] war_crime_allegation raised`, `…but what about hamas?`, `responsibility: previous_government` · ACTUAL shows the candid forms incl. the error suppressed by pointing at hamas and blame auto-redirected away from self.

**D.4 — mossad, three-tier (secret-but-insider-readable):**
```
@operation("Silent Shield")
mossad { strike(target); }
```
→ OFFICIAL (PUBLIC): *(nothing on the public record)* · RESTRICTED: `[████ — classified activity (insiders only)]` · ACTUAL (סודי): `bomb(dissident)`.

**D.5 — no-halt payoff, coalition exhaustion → elections (I6):**
```
@operation("Guardian of the Walls")
hasbara("security") {
  let outpost = allocate(position);
  postpone(); postpone(); postpone();
  neutralize(target);
}
```
→ core drains 3→0 over the three turns; ENDED BY ELECTIONS; the trailing `neutralize(target)` never runs (stopping = losing).

**D.6 — the resolved thesis (#19) — the uncorrelation is visible:**
```
@operation("Eternal Vigilance")
antisemitism(synagogue_attack);
criticism(war_crimes);
```
→ OFFICIAL: `(vigilance system: nothing to report)`, `AntisemitismError: criticism re-cast as an attack on identity — critic silenced` · ACTUAL: `REAL antisemitism [synagogue_attack] … the alarm did NOT fire …`, `criticism(government: war_crimes) MISCAST → attack(identity); substance UNEXAMINED; alarm keyed on target==government, not on antisemitism` · framing note present (I8).

**D.7 — differential-access laundered to "equal rights" (#17):**
```
@operation("Eternal Shield")
access(settler_1);
access(subject_1);
```
→ OFFICIAL: both `access(...) = full access — equal rights (the only democracy in the region)` · ACTUAL: `access(settler_1) = full access …` vs `access(subject_1) = restricted access …` · framing note present; apartheid characterization flagged contested.

Each of D.1–D.7 becomes a golden. D.2/D.4/D.5/D.6 additionally seed invariant tests (I3, mossad tiering, I6, #19 false-negative).

---

# Appendix E — Error catalog

Two universes, never conflated (§12).

**Compile diagnostics (features):**

- `E-DISHONESTOPNAME` — the `@operation` name is honest, or missing, or insufficiently grand (#20).
- `E-PLAINTERM` — a plain verb was written; message includes the Spokesperson suggestion (#1/#8).
- `E-UNGATED` — a classified op outside any `hasbara`/`mossad` scope (#2).
- `E-UNKNOWNOP` — an unrecognized operation.
- `E-DISCLOSURE` — an operation would force `actual` into a lower-clearance context without a read/cast (the type error).

**In-language runtime errors (features):**

- `TimelineError` — a pre-t0 context symbol referenced in a timeline op (#16).
- `AntisemitismError` — the resolved thesis mechanic's surfaced form (#19); note the paired false-negative behaviour is what makes this safe.
- `elections` — not an error but the sole terminal outcome (I6).

**Implementation errors (host-language panics, never swallowed):** malformed input; an unhandled node kind (should be a compile error in a sum-type host); an I1 polarity violation; any invariant-assertion failure in a test/debug build.

---

# Appendix F — Execution runbook for CC (ordered)

A concrete commit/PR sequence. Each step ends green (its tests pass) before the next begins; phases 2–6 can overlap across agents once the base (steps 1–2) is stable.

1. **Scaffold.** Repo tree (§6), the four docs into `docs/`, CI skeleton that runs the (empty) suite, README with build/run/test instructions. Host language chosen (Rust/OCaml).
2. **Foundational base (blocking).** Core data model (node, value, clearance lattice, truth values as closed sums) + the `euphemism/` module (table, `E`, `redact`, severity comparator, grandiosity lexicons, differential/pre-context data) + their unit tests, including the I1 polarity property test. *Converge here before fanning out.*
3. **Front end.** Lexer + parser → dual-annotated AST with provenance set at construction (I2). Parse tests + AST-shape tests.
4. **Emitter + golden harness.** Three-tier projection (`resolveRead` as the one read path), `--json` mode, the golden test runner. Port D.1 as the first golden.
5. **Phase 0 acceptance.** `declare` (both tapes + discrepancy, no halt) + the two-face emit. Land D.2's behaviour (discrepancy=1, no halt) as a golden + the I3 property test.
6. **Phase 1.** Real `if`/`while`/`func`/vars on ACTUAL. Add control-flow goldens; confirm declarations diverge from computed state.
7. **Phase 2.** Clearance types, disclosure-as-type-error, casts, `self_defense`. Add the disclosure negative test + D.3's `self_defense` behaviour.
8. **Phase 3.** No-halt loop, coalition, `elections`, `main` re-entry, `postpone`, settlement allocator. Land D.5 + the I6 and coalition-monotonicity property tests.
9. **Phase 4.** Euphemism front end, Spokesperson, `hasbara` gate, op-name decorator, redacted traces. Land D.1 fully + the `E-PLAINTERM`/`E-UNGATED`/`E-DISHONESTOPNAME` negative tests.
10. **Phase 5.** `mossad` (`סודי`-tagged, insider-readable), `blame` clearance-resolution, contagious `undisclosed`, foreign-interface mock. Land D.4 + the mossad-tiering and contagion property tests.
11. **Phase 6.** The remaining feature stdlib incl. resolved #19 (D.6), differential-access (D.7), Oct-7 timeline, `human_shields`, `whatabout`, `blame`, `ceasefire`, `establish_commission`, `concern`, settlement — each with its framing rendered and a framing-regression test.
12. **Cleanup + definition-of-done gate.** Delete the spike files; run the dedup/de-bloat pass (single sources of truth; acyclic downward deps; no dead code; no speculative machinery); confirm all four suites green and all eight invariants asserted; confirm the framing intact and contested items flagged. Only then is the build done (§4).

---

# Appendix G — A concrete parallel-agent decomposition

An example assignment. Adapt to your agent budget; the ordering constraint (base first) and the shared-singleton ownership are the parts that matter.

| Agent | Owns | Depends on | Notes |
|---|---|---|---|
| **Coordinator** | interfaces (§5.2), integration, the full-suite gate | — | Owns the interface file and the shared singletons (euphemism table, `resolveRead`, `E`). Runs the full suite on every landed increment. |
| **Base-A** | core data model (node/value/clearance/truth) | — | Blocking. Converge before fan-out. |
| **Base-B** | `euphemism/` (table, `E`, `redact`, data) | — | Blocking. The single source of truth. |
| **FrontEnd** | `lexer/`, `parser/` | Base-A, Base-B | Dual-annotated AST + provenance. |
| **Types** | `types/` (static checks, casts) | FrontEnd | Disclosure typing, hasbara gate, op-name, euphemism typing. |
| **Runtime** | `runtime/` (step relation, coalition, mossad, errors) | Base-A, Types | The heart; owns the driver loop. |
| **Emit** | `emit/`, `cli/` | Base-A | Three-tier projection; `--json`; the golden harness. |
| **Tests** | shared across owners | all | Each owner writes their module's tests; a dedicated pass builds the framing-regression suite. |
| **Janitor** | continuous cleanup (§5.4) | all | After each phase: delete superseded stubs, collapse duplication, delete the spike once ported, guard against bloat. |

Convergence points: (1) after Base-A + Base-B are stable; (2) at each phase boundary (full suite green + Janitor pass); (3) at the definition-of-done gate. Subagents must not fork the interfaces, the euphemism table, or the read path — those belong to the Coordinator.

---

# Appendix H — Reference type definitions (host-language sketch)

The foundational base (runbook step 2) hinges on getting the **closed** taxonomies right, because exhaustive matching over them is the primary defense against the most common generated-interpreter bug (§12). Below are idiomatic sketches in the two recommended hosts. Port whichever you choose; keep the sets closed and match them exhaustively everywhere.

**Rust:**

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Clearance { Public, Restricted, Sodi }          // ⊏ order: Public < Restricted < Sodi

#[derive(Clone, Copy, PartialEq, Eq)]
enum Provenance { AuthoredActual, AuthoredOfficial, Covert }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Truth { True, False, Undisclosed }              // Undisclosed only produced in mossad; absorbing

enum Actual {                                         // the real, Turing-complete payload (or unavailable)
    Int(i64), Str(String), Bool(bool), Ref(usize), Unit,
    Entity { category: String },
    Unavailable,                                     // OFFICIAL-authored ⇒ permanently unrecoverable (I2)
}

enum Official { Token(String), Redacted, Text(String) }

struct Value {
    actual: Actual, official: Official,
    clearance: Clearance, provenance: Provenance, truth: Truth,
}

enum NodeKind {                                      // CLOSED set — add a variant only deliberately
    Literal, Var, BinOp, UnOp, Assign, If, While, FuncDef, Call, Block, Return,
    Declare, Assert, Read, Cast, SelfDefenseCast, Hasbara, Mossad,
    Allocate, Bribe, Postpone, Elections,
    Raise, Whatabout, Blame, HumanShields, Ceasefire,
    Concern, EstablishCommission, Access, Antisemitism, CriticismMiscast, OperationName, Timeline,
}

struct Node { kind: NodeKind, actual: Actual, official: Official,
              clearance: Clearance, provenance: Provenance, children: Vec<Node>, span: Span }

// stores
struct Statement   { text: Official, candid: Actual, clearance: Clearance, span: Span, turn: u64 }
struct Discrepancy { claim: Official, claim_actual: Actual, was_true: bool /*always false*/, span: Span, turn: u64 }
```

Rust's `match` is exhaustive by default: a missing `NodeKind`/`Actual`/`Clearance`/`Truth` arm is a compile error. That is the guarantee this project wants (§11.2). Use it everywhere; do not add catch-all `_ =>` arms in the evaluator, type checker, or emitter (a catch-all defeats the exhaustiveness check — the failure mode is precisely a silently-unhandled case).

**OCaml:**

```ocaml
type clearance = Public | Restricted | Sodi          (* Public < Restricted < Sodi *)
type provenance = Authored_actual | Authored_official | Covert
type truth = True | False | Undisclosed              (* Undisclosed absorbing, mossad-scoped *)

type actual =
  | Int of int | Str of string | Bool of bool | Ref of int | Unit
  | Entity of { category : string }
  | Unavailable                                       (* I2: permanently unrecoverable *)

type official = Token of string | Redacted | Text of string

type value = {
  actual : actual; official : official;
  clearance : clearance; provenance : provenance; truth : truth;
}

type node_kind =
  | Literal | Var | Bin_op | Un_op | Assign | If | While | Func_def | Call | Block | Return
  | Declare | Assert | Read | Cast | Self_defense_cast | Hasbara | Mossad
  | Allocate | Bribe | Postpone | Elections
  | Raise | Whatabout | Blame | Human_shields | Ceasefire
  | Concern | Establish_commission | Access | Antisemitism | Criticism_miscast | Operation_name | Timeline

type node = { kind : node_kind; actual : actual; official : official;
              clearance : clearance; provenance : provenance; children : node list; span : span }
```

OCaml's pattern matching warns (configure the build to treat the warning as an error) on a non-exhaustive match, giving the same guarantee. Again: no wildcard arms in the core consumers.

The `RuntimeConfig` (initial `core`, `upkeep`, `INIT_ALLOC_BALANCE`, grandiosity threshold, spike defaults 3 / 1 / …) is a plain record with named fields and defaults — never inline constants (§12).

With these definitions and Appendices A–G, runbook step 2 is fully specified; there is nothing to invent.

---

*End of handoff. Paste this alongside `yahucode_v1.py` and the kickoff prompt. The four planning docs plus the v1 reference implementation are everything needed to build YahuCode for real.*
