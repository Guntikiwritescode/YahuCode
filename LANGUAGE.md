# YahuCode language reference

YahuCode is a language where a run prints two accounts of the same events, the OFFICIAL one and the ACTUAL one, and counts the claims that were false. A value's type is who may read it, memory is a coalition that must be bribed, and a program ends only when the government system falls.

## Build and run

```sh
cargo build
cargo run -- file.yahu                       # both faces and the discrepancy count
cargo run -- --json file.yahu                # the same run as JSON
cargo run -- --press file.yahu               # the public build: press release and rewritten comments
cargo run -- --audience domestic file.yahu   # one room's view, domestic or international
cargo test
```

Source files use the `.yahu` extension. `-` reads stdin and `-h` prints the usage. The three flags cannot be combined.

## Syntax

A program is `@operation("Name")` followed by statements. Statements end with `;` and blocks use braces.

```yahu
@operation("Iron Wall")
# A function, a loop and a claim.
func total(a, b) {
  return a + b;
}
count = 3;
while (count < 6) {
  count = count + 1;
}
if (total(count, 1) == 7) {
  declare(count == 6);
}
```

- Literals are integers, strings in double quotes and `true` and `false`. A string has no escapes.
- A comment starts with `#` and runs to the end of the line. The lexer keeps comments as tokens. The parser takes them out of the token stream and stores their text on the program. They never affect execution.
- Operators are `+ - * / %`, `== != < <= > >=` and `&& || !`. The boolean operators short-circuit. Integer arithmetic wraps on overflow, and division or modulo by zero is a runtime error.
- Zero and the empty string are false. Everything else is true.
- `declare(e)` writes the euphemised claim to the OFFICIAL face and evaluates it against ACTUAL. A false claim adds one discrepancy and changes nothing else. `assert` is another name for `declare`. A claim that cannot be evaluated counts as false, and an unset variable shows as `unset`.
- Assignment changes an existing name. Inside a function it changes an existing local, then an existing global, and otherwise creates a local. A name that holds a collection can never be assigned again.
- A `func` definition takes effect when it runs, so calling the function earlier is a runtime error. A second definition replaces the first.
- A run stops with a runtime error after 512 nested calls or 10 million steps.
- The operation name must sound protective or heroic. Names such as `Protective Edge` and `Iron Wall` are accepted. A name with an honest word such as `bombing`, `occupation` or `siege` is rejected, and so is a name with no grand word.

## The faces

A run prints the OFFICIAL face, the ACTUAL face, the core support and the discrepancy count. A RESTRICTED face appears between the faces when the program has covert activity.

- PUBLIC readers see the OFFICIAL face, the euphemised and redacted press release. Covert events are absent, apart from `responsibility: [neither confirm nor deny]`.
- RESTRICTED readers see ordinary events in candid form. A covert event shows as `[████: classified activity (insiders only)]` and a covert collection element as `[REDACTED]`.
- `סודי` readers see everything in candid form.

A `declare` that would write a classified value into a lower record is rejected with `E-DISCLOSURE`. A cast such as `(PUBLIC) e` sets the clearance of a value, `read(e)` declassifies it, and `(self_defense) e` passes every check. The OFFICIAL text is derived from the candid text and nothing derives it back. An `announce` line has no candid text, so its ACTUAL line is `[unavailable: ...]`.

Example 02 is this program and prints the output below it.

```yahu
@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
}
```

```output
@operation("Protective Edge")
  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────
  │   [talking point: self-defense]
  │   accident occur(protester)
  ├─ ACTUAL face   · סודי · insider (candid) ──────────────────
  │   [talking point declared up front: self-defense]
  │   murder(dissident)
  ├─ coalition: core=3 · still in power; no halt
  └─ discrepancies: 0
```

## Euphemism and the gate

```yahu
@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
  (self_defense) strike(target);
}
```

Only sanctioned verbs compile, and each has a candid meaning. `neutralize` means `murder`, `strike` means `bomb`, `clear` means `raze` and `administer` means `occupy`. A `(self_defense)` prefix adds a claim that is never examined.

The candid verb is rejected with `E-PLAINTERM`, which suggests the sanctioned one. A sanctioned verb outside `hasbara(...)` and `mossad` is rejected with `E-UNGATED`. The entity `target` has the candid label `dissident`, and `operative`, `asset` and `covert_op` are redacted on the OFFICIAL face.

## Coalition and elections

```yahu
@operation("Guardian of the Walls")
let outpost = allocate(position);
bribe(outpost, 2);
postpone();
let hill = settlement(hilltop);
let inquiry = establish_commission(the_incident);
elections;
declare(false);
```

Core support starts at 3. Each `postpone()` is one turn and costs 1 per allocation, and `bribe(x, n)` adds `n`. A settlement costs nothing and is never freed. A commission resolves only once the matter is moot. When the core reaches 0 or `elections;` runs, the government falls and the statements after that point do not run.

## Covert scope

```yahu
@operation("Silent Shield")
mossad {
  secret = external(agency, data);
  blame(operatives);
  via(cutout, strike(target));
}
declare(read(secret) == 1);
```

`mossad { ... }` opens a covert scope. `external(args)` is a mock foreign call that is only allowed inside it and returns `undisclosed`. This third truth value is contagious. An operation on it gives `undisclosed`, an `if` on it takes the else branch and a `while` on it stops.

`blame(x)` assigns responsibility outward. Blaming `self`, `me`, `us`, `government` or `coalition` is redirected to `previous_government`. `via(proxy, op)` adds a proxy to the attribution chain and keeps the origin.

## Sensitive features

```yahu
@operation("Eternal Shield")
hasbara("self-defense") {
  human_shields(neutralize(target));
}
timeline(occupation);
ceasefire;
access(subject_1);
antisemitism(synagogue_attack);
criticism(war_crimes);
```

- `human_shields(verb(target))` needs a talking point. It suppresses the civilian harm and reassigns responsibility to the harmed party without verifying the claim.
- `timeline(symbol)` satirises starting history at a chosen date. `occupation`, `blockade`, `siege` and `nakba` are out of scope.
- `ceasefire;` records a pause and nothing stops.
- `access(entity)` shows full access on the OFFICIAL face. The ACTUAL face looks up the category, and only `subject_1` and `subject_2` are restricted.
- `criticism(subject)` is recast as an attack on identity and its substance is left unexamined. `antisemitism(incident)` records a real incident that the alarm misses, because the alarm fires on criticism of the government.

## Rhetorical statements

```yahu
@operation("Solid Rock")
proportionate(response);
disputed(settlement_units, 100, 900);
deny(airstrike);
world_opinion();
polls();
investigate(the_incident);
address_international();
announce "humanitarian access has been restored";
raise(war_crime_allegation);
whatabout(hamas);
deeply_concerned();
```

Each statement records one event whose OFFICIAL line is prettier than its ACTUAL line.

- `proportionate(claim)` certifies itself with no test.
- `disputed(name, official, actual)` shows the official number, then the real one and the direction of the gap.
- `deny(event)` denies on the OFFICIAL face. The ACTUAL face records that the event happened.
- `world_opinion()`, `polls()` and `address_international()` change nothing.
- `investigate(subject)` is run by its subject and finds no wrongdoing.
- `announce "text"` writes an OFFICIAL line with no ACTUAL line behind it.
- `raise(x)` records an error and `whatabout(y)` suppresses the latest one by pointing at `y`.
- `deeply_concerned()` and `concern(x)` are an ally's concern, and nothing changes.

## Rooms

```yahu
@operation("Dawn of Peace")
statement two_state {
  to international { commit(peace_process); }
  to domestic     { foreclose(final_status); }
}
address(international) { two_state; }
address(domestic)     { two_state; }
```

A `statement` has one arm per room. `address(room) { name; }` delivers it, and a room sees only what was addressed to it. The warning `W-DOUBLETALK` goes to the out-of-world observer when one statement took different arms in different rooms.

## Changing the rules

```yahu
@operation("Iron Law")
clear(hilltop);
legislate(retroactively_sanction: clear);
declare(outposts == 0);
legislate(expunge_last_discrepancy);
```

`legislate` takes `retroactively_sanction: <verb>`, `expunge_last_discrepancy` or `waive_gate`. A retroactive sanction withdraws the diagnostics for that verb, `waive_gate` withdraws `E-UNGATED`, and `expunge_last_discrepancy` removes the latest discrepancy from the public count. Each change goes on a meta-ledger that only `סודי` readers see and no later change can erase.

## Collections

```yahu
@operation("Guardian of Transparency")
let budget = apportionment[3];
mossad { allocate(budget, 0, 800); }
allocate(budget, 1, 5);
allocate(budget, 2, 5);
declare(balanced(budget));

let files = facts_on_the_ground();
push(files, "internal-audit-2024");
remove(files, "internal-audit-2024");
declare(length(files) == 0);

let court = registry("equal before the law");
classify(court, case_A, military);
classify(court, case_B, civilian);
declare(equal_before_the_law(court));
revoke(court, case_A);
```

Collections only accrete, so the ACTUAL face can always rebuild what the public record lost.

- `apportionment[N]` has `N` slots, at most 4096. `allocate(a, i, v)` writes a slot, `index(a, i)` reads one and `balanced(a)` holds when all slots are equal. The OFFICIAL face says the budget was shared equally. The ACTUAL face shows the slots and the skew, or says the shares are equal when they are. A slot written inside `mossad` shows as `[REDACTED]` below `סודי`. A bad index is `E-INDEX` and a negative share is `E-SHARE`.
- `facts_on_the_ground()` only grows. `push` adds an item and `remove` only delists it. `length(l)` counts the live public items, and the ACTUAL face shows the full list.
- `registry("rule")` proclaims one rule. `classify(r, case, military)` or `civilian` routes a case, `route(r, case)` reads the court and `revoke` hides a case publicly while `סודי` keeps it. The OFFICIAL face says every case is handled per due process. The ACTUAL face shows the courts. The cases `case_A`, `case_B` and `inquiry_A` have built-in descriptions.

## Comments and the press build

`--press` prints the OFFICIAL face and then each source comment next to its euphemised form. For example, `# we are going to bomb the neighborhood` becomes `# we are going to strike the neighborhood`, so the comment now contradicts the code. The comments go through the same `E-CONTESTED` check as the code. `examples/11_pillar_of_defense.yahu` shows it.

## Diagnostics

| Code | Meaning |
| --- | --- |
| `E-DISCLOSURE` | A `declare` would write a classified value into a lower record. |
| `E-PLAINTERM` | A plain verb such as `murder` was used. |
| `E-UNGATED` | A classified operation was used outside `hasbara(...)` or `mossad`. |
| `E-UNKNOWNOP` | The name is neither a sanctioned operation nor a defined function. |
| `E-DISHONESTOPNAME` | The `@operation` name is honest or bland. |
| `E-CONTESTED` | A name, string or press comment states a contested characterisation as fact, such as `genocide`. |
| `E-INDEX` | An apportionment index or size is out of range. |
| `E-SHARE` | An apportionment share is negative. |
| `W-DOUBLETALK` | One statement took different arms in different rooms. It is a warning. |

`E-INDEX` and `E-SHARE` are raised while the program runs. The other `E-` codes stop it from compiling. Syntax errors print `parse error (line N)` or `lex error (line N)`. Other runtime errors, such as division by zero, print `runtime error: ...`.

## Exit codes

Exit code 0 means the run finished or the reader closed the output pipe. Code 1 means a syntax error, a compile diagnostic, a runtime error or a write error. Code 2 means a usage error, an unreadable file or input that is not valid UTF-8.
