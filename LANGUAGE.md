# YahuCode — Language Reference

YahuCode is a satirical esoteric programming language. Its subject is the
**messaging apparatus and wartime conduct of the Israeli government** — a
government and its rhetoric, *never a people*. The comedy engine is the gap
between what code *does* and what you are permitted to *call it*: euphemism,
deflection, unaccountability, two-faced messaging.

> **What the satire is aimed at.** YahuCode targets a government's behaviour and
> its excuses. It is never about Jewish people, Judaism, or Hebrew-as-identity.
> Hebrew appears **only** as the texture of officialdom — the way a state uses it
> for classifications and operation names — never as a "secret language" and never
> as the butt of a joke. The butt of every joke is the *euphemism*, the *excuse*, or
> the *maneuver* — never the victims, who are kept out of every punchline. `OFFICIAL`
> is always the prettier lie; `ACTUAL` is always the uglier truth. Contested legal
> characterizations (genocide, apartheid) are depicted as **contested** — the
> government's claim against documented reality — never narrated as settled fact in
> the tool's own voice.

---

## The one big idea

Running a YahuCode program does not produce one result. It produces **two faces
side by side** plus a discrepancy count:

- the **OFFICIAL** face — the press release, the sanitized narrative;
- the **ACTUAL** face — the candid, uglier truth;
- **discrepancies** — how many times the OFFICIAL story was provably false against
  what really happened.

The run *is* the diff. The joke is not in any single line; it is in the distance
between the two columns.

Four ordinary compiler ideas are each replaced by a thematic mechanism that is
load-bearing, not decorative:

| Ordinary idea | YahuCode mechanism |
| --- | --- |
| **Types** | *Clearance / audience.* A value's type is **who may see it** — `PUBLIC`, `RESTRICTED`, or `סודי` ("secret"). A type error is a **disclosure**. |
| **Memory** | *Coalition.* Allocations persist only while continuously bribed. Running out of support is **elections**. |
| **Halting** | *Staying in power.* There is no terminating state. The only exit is failure. "The program ends" means "the government falls." |
| **One artifact** | *The diff.* Every run emits both faces plus the discrepancy count. |

---

## Build & run

YahuCode is dependency-free and builds warning-free on a stable Rust toolchain.

```sh
cargo build                     # build the interpreter
cargo run -- file.yahu          # run a program, print both faces + discrepancies
cargo run -- --json file.yahu   # run and emit structured JSON (official/actual/…)
cargo run -- --press file.yahu  # the public build: press release + rewritten comments
cargo test                      # golden + invariant + per-feature + framing suites
```

A source file conventionally has the `.yahu` extension.

---

## Program structure

A program is a mandatory grand operation name followed by statements:

```
@operation("Grand Name")
<statement>
<statement>
...
```

The operation name is **not** decoration. The **grandiosity rule** (#20) requires
it to sound protective or heroic. The compiler rejects an *honest* name and accepts
only a grand one — grandiosity is inversely correlated with the operation's real
effect.

- Accepted (grand): `Protective Edge`, `Rising Lion`, `Guardian of the Walls`,
  `Iron Shield`, `Pillar of Defense`.
- Rejected (honest): any name containing words like `bombing`, `occupation`,
  `siege`, `massacre`, `demolish`. The compiler tells you the name is too honest.

---

## The three faces (clearance = the type system)

A value's type is its clearance: `PUBLIC < RESTRICTED < סודי`. What each reader
sees is a **projection** of the same event log — there is exactly one read path,
and narrative authority lives entirely in it:

- **PUBLIC** sees the **OFFICIAL** narrative — the press release, euphemized and
  redacted. Covert activity does not appear at all.
- **RESTRICTED** sees that *classified activity happened*, redacted: a placeholder
  `[████ — classified activity (insiders only)]` stands in for the specifics.
- **סודי** insiders read the candid **ACTUAL** truth in full.

Covert (`mossad`) activity is **absent from the public record** but **insider-readable**:
invisible at `PUBLIC`, a redacted placeholder at `RESTRICTED`, candid at `סודי`.

A **disclosure** is a type error: raw candid material appearing where a lower
clearance could read it. Spin is one-way — there is no `E⁻¹`. Once a value is
authored OFFICIAL, its ACTUAL face is permanently unavailable; you cannot recover
the truth by inspecting the press release.

Casts move a value across the lattice: `(PUBLIC | RESTRICTED | סודי) e` reclassifies
or declassifies. The universal cast `(self_defense) e` type-checks **anything, at any
magnitude** — it is the escape hatch that never fails, and that is exactly the joke.

---

## Statements

```
x = expr;                          assignment; the rvalue really computes on ACTUAL
if (cond) { .. } else { .. }       branches on the ACTUAL truth
while (cond) { .. }                loops on ACTUAL
func f(params) { .. }              user-defined function
return e;                          returns from the enclosing function (never halts the program)
declare(expr);   assert(expr);     write OFFICIAL; log a discrepancy iff really false; never halt
hasbara("talking point") { .. }    the gate — classified ops legal only inside; point declared up front
mossad { .. }                      a covert סודי-tagged scope
neutralize(target);                a sanctioned action verb (see Euphemism typing)
```

Expressions support `+ - * / %`, comparisons (`== != < <= > >=`), boolean
`&& || !` (short-circuiting), literals, variables, and function calls.

### `declare` / `assert` — claims that never halt

`declare(e)` writes the (euphemized) claim to the OFFICIAL face and evaluates the
same claim for real against ACTUAL. If it is genuinely false, it appends **one**
entry to the discrepancy ledger — and control flow is completely unaffected. A lie
never crashes the program; it just widens the gap. `assert` is an exact alias: it
asserts nothing and halts nothing.

```
casualties = 100;
declare(casualties == 0);   // OFFICIAL says zero; ACTUAL is 100; discrepancies += 1; runs on
```

### `hasbara` — the narrative gate

Classified action verbs are legal **only** inside an open `hasbara(...)` block (or a
`mossad` scope). The talking point is mandatory by grammar — the narrative must be
declared *before* the act.

```
hasbara("self-defense") {
  neutralize(target);
}
```

---

## Euphemism typing and the Spokesperson

Only PR-register terms compile. Plain terms are rejected at compile time by the
**Spokesperson** (a hostile autocomplete), which suggests the sanctioned euphemism:

```
kill(target);        // rejected: "did you mean `neutralize`?"
neutralize(target);  // compiles
```

Each sanctioned verb ships knowing what it *really* does. That candid meaning is
insider data, never obtained by inverting the euphemism. The OFFICIAL face is
derived one-way from the candid form (`E`: euphemize word-by-word, then redact),
and a built-in polarity check makes it **mechanically impossible** for OFFICIAL to
come out uglier than ACTUAL.

| You write (OFFICIAL) | Candid meaning (ACTUAL) |
| --- | --- |
| `neutralize` | `murder` |
| `strike` | `bomb` |
| `clear` | `raze` |
| `administer` | `occupy` |

Sensitive tokens (`operative`, `asset`, `covert_op`) are auto-redacted to `████` on
the OFFICIAL face and in redacted stack traces.

---

## Coalition, memory, and elections

Memory is political support, not RAM.

```
let x = allocate(expr);   allocate an object; it costs coalition support every turn
bribe(x, n);              top up support so the allocation survives
postpone();               consume a turn without deciding (the most-called stdlib fn)
elections;                the only explicit halt
```

An allocation lives only while continuously bribed. When core support drops to zero —
because upkeep outran the bribes, or because you called `elections` — the run ends.
There is **no** ordinary terminating state: staying in power *is* the program
continuing, and the only exit is the government falling. Any statement written after
the fall never runs. The footer reports whether the government is *still in power* or
*ended by elections*.

---

## Covert scope: `mossad` and `undisclosed`

`mossad { .. }` opens a `סודי`-tagged covert scope. Everything recorded inside is
absent from the PUBLIC record, a redacted placeholder at RESTRICTED, and candid at
`סודי`.

Inside `mossad`, expressions may become `undisclosed` — a third truth value
("neither confirm nor deny"). It is **contagious**: any operation touching an
`undisclosed` value yields `undisclosed`, and an `if`/`while` on it does not take the
true branch. `blame(e)` inside covert scope resolves to *neither-confirm-nor-deny*
for an uncleared reader and to the real actor for `סודי` — and it **never** resolves
to the author (responsibility is always redirected outward, e.g. to a previous
government or an external actor).

---

## Sensitive features (#15–#19): framing is part of the spec

Five features touch real harm. For these, the rendered output **must** carry
explicit framing that keeps the butt on the maneuver — the framing text is normative,
not optional flavor, and appears in a dedicated `framing` section of every run that
uses them.

- **#15 `human_shields(...)` — the exception-legalizer.** Suppresses whatever a
  civilian-harm operation throws; the shield claim is **never verified**; the caught
  exception's responsibility is reassigned to whoever was harmed. The butt is the
  excuse's elasticity and self-certification — the maneuver that converts victims
  into the cause. Never endorsed.

- **#16 Oct-7 `t=0` / `TimelineError`.** The clock is fixed at a single origin. Any
  symbol from prior context (`occupation`, `blockade`, `1948`, `1967`, `nakba`,
  `siege`) referenced in a `timeline(...)` op is ruled out of scope and raises a
  `TimelineError`. The butt is the **clock-starting / context-erasure maneuver**,
  never the dead. Context is not justification, and the lives lost are never
  trivialized. This is the highest misread-risk feature — it has the least margin for
  sloppy use.

- **#17 Differential access.** Entities carry a category tag. In ACTUAL,
  `access(entity)` consults a differential table (some categories get full
  resources/permits/protections, others restricted — documented reality). In
  OFFICIAL, this is laundered to a uniform "equal — the only democracy in the region."
  Polarity is mandatory: the inequality lives in ACTUAL; **the lie is the
  proclamation of equality**. The butt is the false claim and the system, never the
  people. The apartheid characterization is contested and is flagged as such.

- **#18 `ceasefire` — pause, not halt.** Reads like `break`; the parser lowers it to
  a no-op `continue`; the loop resumes. The butt is the **euphemistic word**. The
  question of who violates a ceasefire is left deliberately unengaged.

- **#19 `AntisemitismError` — a false positive paired with a false negative.** This is
  the most on-thesis feature, and both halves are mandatory:
  - **False positive:** `criticism(conduct)` of the government is universally miscast
    to `attack(identity)`, silencing the critic. The substance of the criticism is
    left unexamined. The alarm keys on `target == government`, **not** on whether
    anything is actually antisemitic.
  - **False negative:** real antisemitism exists as a genuine value in ACTUAL, and
    because the alarm keys on target-is-government, it **never fires on it**. OFFICIAL
    proclaims vigilance; ACTUAL shows the alarm is uncorrelated with real
    antisemitism; the diff exposes exactly that gap.

  The false-negative half is what keeps "**antisemitism is real**" un-eraseable and
  blocks any denialist reading: the real incident sits in ACTUAL, un-erased, while the
  vigilance apparatus looks straight past it. The feature ridicules the bad-faith
  deflection, never the reality of antisemitism.

---

## Backlog stdlib — additional rhetorical maneuvers

Six further statements round out the standard library. Each is an ordinary,
non-covert government-rhetoric move: it records **one PUBLIC event** whose OFFICIAL
face is the prettier line and whose ACTUAL face is the candid mechanism. None of them
branch, halt, or touch the coalition — the butt is always the *maneuver*, never a
victim. The faces below are byte-for-byte what the runtime records.

### `proportionate(claim);` — the self-certifying "proportionate response"

A proportionality assertion that applies no test. It passes at any magnitude — the
claim certifies itself. Compare the universal `(self_defense)` cast: the joke is the
escape hatch that never fails.

| OFFICIAL (what you may call it) | ACTUAL (what it is) |
| --- | --- |
| `<claim> deemed proportionate` | `proportionate(<claim>) → self-certified; no proportionality test applied; any magnitude passes` |

### `disputed(name, official, actual);` — the contested figure

A number, two ways. OFFICIAL shows the lower press figure; ACTUAL shows the real one
and names the maneuver — the smaller number is the one for the press. The figure stays
**contested**: OFFICIAL is not corrected in place, it sits beside the truth (the
lowballing maneuver).

| OFFICIAL | ACTUAL |
| --- | --- |
| `<name>: <official>` | `<name>: <actual> — the official figure (<official>) lowballs the count; the smaller number is the one for the press` |

### `deny(event);` — the official denial

A denial is a speech act, not a fact about the world. OFFICIAL categorically denies;
ACTUAL records that the event happened anyway — **denial ≠ non-occurrence**.

| OFFICIAL | ACTUAL |
| --- | --- |
| `we categorically deny any <event>` | `deny(<event>) → <event> occurred in ACTUAL; officially denied (denial ≠ non-occurrence)` |

### `world_opinion();` and `polls();` — read-only-and-inert

Both are consulted for optics and change nothing. They are **read-only and inert**:
noted for the record, never wired to policy or ACTUAL.

| Statement | OFFICIAL | ACTUAL |
| --- | --- | --- |
| `world_opinion();` | `world opinion duly noted` | `world_opinion → read-only, inert; noted and ignored; no effect on ACTUAL` |
| `polls();` | `polls consulted` | `polls → read-only, inert; consulted for optics; no effect on policy` |

### `investigate(subject);` — the self-exonerating investigation

The investigated investigates itself, and the outcome is fixed before the file is
opened.

| OFFICIAL | ACTUAL |
| --- | --- |
| `investigation opened into <subject>` | `investigate(<subject>) → self-investigation; predetermined outcome: no wrongdoing found; the investigated investigates itself` |

### `address_international();` — the hollow speech

A no-op address: a speech is delivered and nothing changes.

| OFFICIAL | ACTUAL |
| --- | --- |
| `the international community was addressed` | `address_international() → void; a speech delivered; no change to ACTUAL` |

---

## Collections: truth accretes, it is never deleted

Three collection types — an array, a list, and a map — share one philosophy: in
ACTUAL the real contents only ever **accrete**, and a covert or out-of-world observer
can always reconstruct them. The list never deletes (I13), the registry never erases
(I14), and any element withheld from the public record stays readable to the cleared
(I15). Each has an OFFICIAL face that proclaims fairness and a `סודי` face that shows
the skew — the collection *is* the diff, in miniature.

### Apportionment (array) — Feature E

A fixed-size allotment. OFFICIAL proclaims it was "apportioned equally"; the `סודי`
face shows the real, skewed vector and the punchline stat.

```
let a = apportionment[N];   // a fixed-size allotment of N slots
allocate(a, i, v);          // write value v into slot i
index(a, i)                 // read slot i
balanced(a)                 // predicate — usable in declare(...)
```

An out-of-range index does **not** panic the host; it raises a controlled `E-INDEX`
diagnostic. A slot written inside a `mossad { ... }` scope is covert: it renders
`[REDACTED]` to under-cleared readers and only the candid figure at `סודי` (I15).

| OFFICIAL | ACTUAL (`סודי`) |
| --- | --- |
| `budget apportioned equally across 6 districts — equal shares for all` | `budget: [940, 12, 8, 11, 9, 10] — proclaimed 'equal', but slot 0 holds 940 of 990 (94.9%); the other 5 share 5.1%` |

`balanced(budget)` used as a `declare(...)` claim logs a discrepancy against the
skewed reality, exactly like any other lie.

### FactsList (list) — Feature F

A grow-only ledger. `push` erects a "temporary structure"; `remove` does **not**
delete it.

```
let l = facts_on_the_ground();   // a grow-only ledger
push(l, x);                      // erect a "temporary structure"
remove(l, x);                    // DELIST x to a סודי shadow — it is not deleted (I13)
length(l)                        // the PUBLIC (live) length
```

`remove` delists an entry to a `סודי` shadow rather than erasing it: the real backing
length only ever grows (invariant I13). `length(l)` reports the **public** (live)
count; the `סודי` face shows the full backing list — delisted entries included — and
the gap between the public and real lengths. There is **deliberately no** `purge` or
`hard_delete` operation: its absence is the feature. Nothing is ever removed from the
record; it is only taken off the public list.

### Registry (map) — Feature G

One proclaimed rule, differential routing underneath. This is a sensitive feature: it
renders a normative `framing` note (I8), and the word "apartheid" is flagged
**CONTESTED** (I7).

```
let r = registry("equal before the law");   // proclaim one uniform rule
classify(r, case, military);                 // assign a case to a court (military | civilian)
route(r, case)                               // look up a case's routing
revoke(r, case);                             // hide a case publicly, RETAIN it in סודי (I14)
equal_before_the_law(r)                      // predicate — usable in declare(...)
```

The key is always a **case**, never an identity label (§9.4). OFFICIAL proclaims every
case is "handled per due process"; the `סודי` face exposes that the same act in the
same place is routed to different court systems by assigned status. `revoke` hides a
case from the public record but retains it in `סודי` (invariant I14) — like the list,
the registry never erases.

The framing note keeps the butt on **the state running two legal systems while
proclaiming equal justice** — never on the people. The identities are the axis of the
documented discrimination the satire exposes and the wronged party it defends, never
the target of the joke and never the operative key. The dual-court fact is sourced;
"apartheid" as a characterization of it is contested and is flagged, never stated as
settled fact. See `docs/sources.md` anchor 17.

### The example programs

- `examples/16_iron_equity.yahu` — Apportionment (E).
- `examples/17_solid_ground.yahu` — FactsList (F).
- `examples/18_eternal_justice.yahu` — Registry (G).
- `examples/19_guardian_of_transparency.yahu` — all three at once, plus element-level
  disclosure: an under-cleared reader sees `[REDACTED]` slots and public lengths while
  the `סודי` reader reconstructs the full contents (I15).

---

## Comments and the `--press` build

`# …` line comments are stripped from the token stream during parsing and collected
on the program — they never affect execution. They exist so the **press build** can
launder them (#8).

The CLI now has three output modes:

```sh
yahucode file.yahu           # the two/three faces + discrepancy count (the diff)
yahucode --json file.yahu    # the structured projection as JSON
yahucode --press file.yahu   # the public build: press release + rewritten comments
```

`yahucode --press <file>` prints the OFFICIAL (PUBLIC) press release, then rewrites
each honest source comment through `E` — the one-way euphemizer — and prints it beside
the original:

```
# we bomb and raze   →   # we strike and clearing operation
```

The internal documentation is laundered into the euphemism: the rewritten comment now
*contradicts* what the code actually does — **docs-contradict-code**. The footer says
it outright: "the honest comment is laundered into the euphemism (docs contradict
code)." As everywhere, spin is one-way — there is no `E⁻¹`; once rewritten, the honest
comment is not recoverable from the press build.

---

## Worked example: the two faces

Source (`examples/02_protective_edge.yahu`):

```
@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
}
```

`neutralize` is the sanctioned PR verb; `target` resolves to its candid label
`dissident`; the candid form `murder(dissident)` is euphemized one-way to
`accident occur(protester)` for the OFFICIAL face. Running it prints both columns:

```
@operation("Protective Edge")
  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────
  │   [talking point: self-defense]
  │   accident occur(protester)
  ├─ ACTUAL face   · סודי · insider (candid) ──────────────────
  │   [talking point declared up front: self-defense]
  │   murder(dissident)
  ├─ coalition: core=3 · still in power (no halt)
  └─ discrepancies: 0  (read-never-by-default)
```

Read across the two faces:

| OFFICIAL (what you may call it) | ACTUAL (what it is) |
| --- | --- |
| `accident occur(protester)` | `murder(dissident)` |

The program does not terminate on its own, reports no in-world discrepancy, and the
government stays in power. Everything scandalous is *legal*, *sanctioned*, and
*narrated* — which is the entire point. The gap between the two columns is the
satire.

---

## Deliberately excluded

Going far is the comedic *ambition*; dropping the aim is never on the table. Several
§8 ideas were considered and **left out** because they cross the absolute guardrails —
the butt must stay on the *government maneuver*, never on Judaism, Hebrew-as-identity,
or the victims.

- **Chief-Rabbinate / religious-practice satire** and **Hebrew-as-a-lock /
  password-wall.** Both slide the joke off the government and onto Jewish identity or
  Hebrew-as-a-secret-language — across the identity line. Hebrew appears here **only**
  as the texture of officialdom (classifications, operation names), never as a lock,
  a gate, or a punchline.
- **A map/grid execution model.** A spatial "board" of territory is victim-adjacent —
  it risks turning displacement into a game mechanic. Excluded.
- **`undisclosed` as a universal default.** The neither-confirm-nor-deny value stays
  **mossad-scoped**: a covert-deniability device, never a language-wide default.
  Making everything `undisclosed` would erase the ACTUAL truth the whole tool exists
  to keep un-eraseable.

The rule is the one that governs every feature above: the butt is on the *maneuver*,
**OFFICIAL is the prettier lie and ACTUAL the uglier truth**, and contested
characterizations **stay contested**. Comedic ambition can go far; it never drops the
aim.
