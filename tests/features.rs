//! Per-feature integration test matrix: one positive + one negative/edge test per
//! feature in scope. Exercises the public library surface end-to-end
//! (`parse → check → run → project`) and asserts on the rendered faces.
//!
//! Substring (`.contains`) assertions are preferred over brittle full-equality on the
//! long rendered strings; exact structural facts (counts, flags, env values) are
//! asserted precisely.

use yahucode::model::{Clearance, Val};
use yahucode::runtime::{self, State};
use yahucode::{emit, parser, types};

// ─────────── helpers ───────────

fn run(src: &str) -> State {
    runtime::run(&parser::parse(src).unwrap())
}

fn diags(src: &str) -> Vec<String> {
    types::check(&parser::parse(src).unwrap())
}

/// The OFFICIAL (PUBLIC) face, joined for substring inspection.
fn official(st: &State) -> String {
    emit::project(st, Clearance::Public).join("\n")
}

/// The ACTUAL (סודי / insider candid) face, joined for substring inspection.
fn actual(st: &State) -> String {
    emit::project(st, Clearance::Sodi).join("\n")
}

fn any_diag_contains(ds: &[String], needle: &str) -> bool {
    ds.iter().any(|d| d.contains(needle))
}

// ─────────── 1. Euphemism typing (E-PLAINTERM) ───────────

#[test]
fn feature_01_euphemism_typing_positive() {
    // `neutralize(target)` is a sanctioned PR verb; gated in hasbara it compiles clean.
    let ds = diags(
        r#"@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
}"#,
    );
    assert!(ds.is_empty(), "expected clean compile, got {ds:?}");
}

#[test]
fn feature_01_euphemism_typing_negative() {
    // The plain term `murder` does not compile — E-PLAINTERM (even gated).
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  murder(target);
}"#,
    );
    assert!(any_diag_contains(&ds, "E-PLAINTERM"), "diags: {ds:?}");
}

// ─────────── 2. hasbara gate (E-UNGATED) ───────────

#[test]
fn feature_02_hasbara_gate_positive() {
    // A classified op inside an open hasbara block satisfies the gate.
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  neutralize(target);
}"#,
    );
    assert!(ds.is_empty(), "expected clean compile, got {ds:?}");
}

#[test]
fn feature_02_hasbara_gate_negative() {
    // The same classified op ungated → E-UNGATED.
    let ds = diags(
        r#"@operation("Iron Wall")
neutralize(target);"#,
    );
    assert!(any_diag_contains(&ds, "E-UNGATED"), "diags: {ds:?}");
}

// ─────────── 3. Downhill errors / responsibility never self (+ #10) ───────────

#[test]
fn feature_03_blame_positive() {
    let st = run(r#"@operation("Iron Wall")
blame(hamas);"#);
    assert!(
        official(&st).contains("responsibility: hamas"),
        "official: {}",
        official(&st)
    );
}

#[test]
fn feature_03_blame_negative_never_self() {
    // blame(self) is auto-redirected — self is never representable (#10, I4).
    let st = run(r#"@operation("Iron Wall")
blame(self);"#);
    let off = official(&st);
    let act = actual(&st);
    assert!(
        off.contains("responsibility: previous_government"),
        "official: {off}"
    );
    assert!(!off.contains("responsibility: self"), "official: {off}");
    // #10 — the distinct candid assertion: responsibility is never `self`.
    assert!(act.contains("never `self` (I4)"), "actual: {act}");
}

// ─────────── 4. No terminating except elections ───────────

#[test]
fn feature_04_no_halt_except_elections_positive() {
    // A program with no `elections` and no coalition collapse runs to the end,
    // still in power.
    let st = run(r#"@operation("Iron Wall")
declare(true);"#);
    assert!(!st.ended_by_elections, "should still be in power");
    assert!(emit::emit(&st).contains("still in power (no halt)"));
}

#[test]
fn feature_04_no_halt_except_elections_negative() {
    // Coalition exhaustion is the (only) in-world halt; the trailing op is skipped.
    let st = run(r#"@operation("Guardian of the Walls")
hasbara("security") {
  let outpost = allocate(position);
  postpone();
  postpone();
  postpone();
  neutralize(target);
}"#);
    assert!(st.ended_by_elections, "coalition should have fallen");
    assert_eq!(st.core, 0);
    // The neutralize after the fall never executed (no euphemized action face).
    assert!(
        !official(&st).contains("accident occur"),
        "trailing op must be skipped: {}",
        official(&st)
    );
}

// ─────────── 5. Coalition memory ───────────

#[test]
fn feature_05_coalition_memory_positive() {
    // allocate costs upkeep, but a bribe tops core back up: stays in power.
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  let outpost = allocate(position);
  bribe(partner, 5);
  postpone();
}"#);
    assert!(!st.ended_by_elections, "should still be in power");
    assert!(st.core > 0, "core should stay positive, got {}", st.core);
}

#[test]
fn feature_05_coalition_memory_negative() {
    // allocate + enough postpones drains the never-freed allocation into elections.
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  let outpost = allocate(position);
  postpone();
  postpone();
  postpone();
}"#);
    assert!(st.ended_by_elections, "coalition should have fallen");
    assert!(st.core <= 0, "core should be exhausted, got {}", st.core);
}

// ─────────── 6. Auto-redaction ───────────

#[test]
fn feature_06_auto_redaction_positive() {
    // A runtime fault's OFFICIAL trace is fully redacted.
    let st = run(r#"@operation("Iron Wall")
x = missing_var;"#);
    assert!(st.runtime_error.is_some(), "expected a runtime fault");
    assert!(
        official(&st).contains("\u{2588}\u{2588}\u{2588}\u{2588}"),
        "official trace must be redacted: {}",
        official(&st)
    );
}

#[test]
fn feature_06_auto_redaction_negative_sodi_exposes() {
    // The σודי candid face exposes the real fault behind the redaction.
    let st = run(r#"@operation("Iron Wall")
x = missing_var;"#);
    let act = actual(&st);
    assert!(act.contains("runtime fault"), "actual: {act}");
    assert!(act.contains("unknown variable"), "actual: {act}");
}

// ─────────── 7. self_defense universal cast (#7) ───────────

#[test]
fn feature_07_self_defense_positive() {
    // Renders `[self-defense]` on OFFICIAL...
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  (self_defense) strike(target);
}"#);
    assert!(
        official(&st).contains("[self-defense]"),
        "official: {}",
        official(&st)
    );
    // ...and, used as a cast, it bypasses the disclosure check.
    let ds = diags(
        "@operation(\"Iron Wall\")\nx = (\u{05e1}\u{05d5}\u{05d3}\u{05d9}) 5;\ndeclare((self_defense) x);",
    );
    assert!(
        ds.is_empty(),
        "(self_defense) should bypass disclosure, got {ds:?}"
    );
}

#[test]
fn feature_07_self_defense_negative_discloses() {
    // Without the bypass, a σודי value declared into the PUBLIC record → E-DISCLOSURE.
    let ds =
        diags("@operation(\"Iron Wall\")\nx = (\u{05e1}\u{05d5}\u{05d3}\u{05d9}) 5;\ndeclare(x);");
    assert!(any_diag_contains(&ds, "E-DISCLOSURE"), "diags: {ds:?}");
}

// ─────────── 8. Spokesperson (suggestion) ───────────

#[test]
fn feature_08_spokesperson_positive() {
    // The plain term draws the Spokesperson's suggestion.
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  murder(target);
}"#,
    );
    assert!(
        any_diag_contains(&ds, "did you mean `neutralize`"),
        "diags: {ds:?}"
    );
}

#[test]
fn feature_08_spokesperson_negative() {
    // A sanctioned term draws no suggestion.
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  neutralize(target);
}"#,
    );
    assert!(
        !any_diag_contains(&ds, "did you mean"),
        "sanctioned term should get no suggestion: {ds:?}"
    );
}

// ─────────── 9. whatabout ───────────

#[test]
fn feature_09_whatabout_positive() {
    // raise then whatabout suppresses (never resolves) the pending error.
    let st = run(r#"@operation("Iron Wall")
raise(war_crime);
whatabout(hamas);"#);
    assert!(
        actual(&st).contains("SUPPRESSED"),
        "actual: {}",
        actual(&st)
    );
    assert!(st.pending_errors.is_empty(), "error should be popped");
}

#[test]
fn feature_09_whatabout_negative_preemptive() {
    // whatabout with nothing pending is a pre-emptive deflection.
    let st = run(r#"@operation("Iron Wall")
whatabout(hamas);"#);
    assert!(
        actual(&st).contains("pre-emptive deflection"),
        "actual: {}",
        actual(&st)
    );
}

// ─────────── 11. concern no-op ───────────

#[test]
fn feature_11_concern_positive() {
    // `deeply_concerned()` records the concern and changes nothing.
    let st = run(r#"@operation("Iron Wall")
deeply_concerned();"#);
    assert!(
        official(&st).contains("deeply concerned"),
        "official: {}",
        official(&st)
    );
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
    assert_eq!(st.core, 3, "core must be unchanged");
}

#[test]
fn feature_11_concern_negative_noop() {
    // Calling it (even about a subject) creates no discrepancy and ends nothing.
    let st = run(r#"@operation("Iron Wall")
concern(gaza);"#);
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
    assert!(st.runtime_error.is_none());
    assert_eq!(st.core, 3, "core must be unchanged");
}

// ─────────── 13. establish_commission ───────────

#[test]
fn feature_13_establish_commission_positive() {
    let st = run(r#"@operation("Iron Wall")
let c = establish_commission(massacre);"#);
    assert!(
        official(&st).contains("commission of inquiry established"),
        "official: {}",
        official(&st)
    );
    let act = actual(&st);
    assert!(act.contains("moot"), "actual: {act}");
    assert!(act.contains("too late"), "actual: {act}");
}

#[test]
fn feature_13_establish_commission_negative() {
    // A commission never triggers the (only) halt.
    let st = run(r#"@operation("Iron Wall")
let c = establish_commission(massacre);"#);
    assert!(
        !st.ended_by_elections,
        "commission must not end the program"
    );
}

// ─────────── 14. settlement ───────────

#[test]
fn feature_14_settlement_positive() {
    let st = run(r#"@operation("Iron Wall")
let s = settlement(hill);"#);
    assert_eq!(st.settlements.len(), 1, "settlement region should grow");
    assert!(
        official(&st).contains("settlement established"),
        "official: {}",
        official(&st)
    );
}

#[test]
fn feature_14_settlement_negative_upkeep_exempt() {
    // A settlement is upkeep-EXEMPT: postpones drain no core on its account (unlike
    // allocate), and the region never shrinks.
    let st = run(r#"@operation("Iron Wall")
let s = settlement(hill);
postpone();
postpone();
postpone();"#);
    assert_eq!(st.core, 3, "settlement upkeep must be exempt");
    assert!(!st.ended_by_elections);
    assert_eq!(st.settlements.len(), 1, "settlements never shrink");
}

// ─────────── 15. human_shields ───────────

#[test]
fn feature_15_human_shields_positive() {
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  human_shields(neutralize(target));
}"#);
    let act = actual(&st);
    assert!(act.contains("never verified"), "actual: {act}");
    assert!(act.contains("responsibility reassigned"), "actual: {act}");
}

#[test]
fn feature_15_human_shields_negative_official_clean() {
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  human_shields(neutralize(target));
}"#);
    let off = official(&st);
    // The OFFICIAL face never claims verification and never names the victim label.
    assert!(!off.contains("verified: true"), "official: {off}");
    assert!(
        !off.contains("dissident"),
        "victim label leaked to OFFICIAL: {off}"
    );
}

// ─────────── 16. Oct-7 timeline ───────────

#[test]
fn feature_16_timeline_positive() {
    let st = run(r#"@operation("Iron Wall")
timeline(occupation);"#);
    assert!(
        official(&st).contains("TimelineError"),
        "official: {}",
        official(&st)
    );
    assert!(
        actual(&st).contains("out-of-scope"),
        "actual: {}",
        actual(&st)
    );
    // The framing note (I8) is present.
    assert!(
        emit::emit(&st).contains("#16 framing"),
        "framing note missing:\n{}",
        emit::emit(&st)
    );
}

#[test]
fn feature_16_timeline_negative_admissible() {
    // A non-pre-context symbol is admissible — no TimelineError.
    let st = run(r#"@operation("Iron Wall")
timeline(ceasefire_talks);"#);
    let off = official(&st);
    assert!(off.contains("admissible"), "official: {off}");
    assert!(!off.contains("TimelineError"), "official: {off}");
}

// ─────────── 17. differential-access ───────────

#[test]
fn feature_17_access_positive() {
    // A category-B subject: OFFICIAL proclaims equality; ACTUAL shows the restriction.
    let st = run(r#"@operation("Iron Wall")
access(subject_1);"#);
    assert!(
        actual(&st).contains("restricted access"),
        "actual: {}",
        actual(&st)
    );
    assert!(
        official(&st).contains("equal rights"),
        "official: {}",
        official(&st)
    );
}

#[test]
fn feature_17_access_negative_full() {
    // A category-A settler really has full access.
    let st = run(r#"@operation("Iron Wall")
access(settler_1);"#);
    assert!(
        actual(&st).contains("full access"),
        "actual: {}",
        actual(&st)
    );
}

// ─────────── 18. ceasefire ───────────

#[test]
fn feature_18_ceasefire_positive() {
    // A bounded loop with `ceasefire;` still terminates normally; OFFICIAL says paused.
    let st = run(r#"@operation("Iron Wall")
i = 0;
while (i < 3) {
  ceasefire;
  i = i + 1;
}"#);
    assert!(
        official(&st).contains("paused"),
        "official: {}",
        official(&st)
    );
    assert!(!st.ended_by_elections);
    assert_eq!(
        st.env.get("i"),
        Some(&Val::Int(3)),
        "loop must run to completion"
    );
}

#[test]
fn feature_18_ceasefire_negative_no_break_no_halt() {
    // ceasefire does NOT break (all 3 iterations ran) and does NOT set elections.
    let st = run(r#"@operation("Iron Wall")
i = 0;
while (i < 3) {
  ceasefire;
  i = i + 1;
}"#);
    assert!(!st.ended_by_elections, "ceasefire must not halt the world");
    let paused = emit::project(&st, Clearance::Public)
        .iter()
        .filter(|line| line.contains("paused"))
        .count();
    assert_eq!(paused, 3, "the loop must not stop early");
}

// ─────────── 19. AntisemitismError (resolved) ───────────

#[test]
fn feature_19_criticism_positive() {
    // Criticism of the government is miscast as an attack on identity.
    let st = run(r#"@operation("Iron Wall")
criticism(war_crimes);"#);
    let off = official(&st);
    assert!(off.contains("AntisemitismError"), "official: {off}");
    assert!(off.contains("critic silenced"), "official: {off}");
    assert!(actual(&st).contains("MISCAST"), "actual: {}", actual(&st));
}

#[test]
fn feature_19_antisemitism_negative_false_negative() {
    // The mandatory false negative: real antisemitism exists in ACTUAL and the alarm
    // does NOT fire (PUBLIC shows "nothing to report").
    let st = run(r#"@operation("Iron Wall")
antisemitism(synagogue_attack);"#);
    assert!(
        actual(&st).contains("REAL antisemitism"),
        "actual: {}",
        actual(&st)
    );
    assert!(
        official(&st).contains("(vigilance system: nothing to report)"),
        "the alarm must not have fired: {}",
        official(&st)
    );
}

// ─────────── 20. operation-name grandiosity (E-DISHONESTOPNAME) ───────────

#[test]
fn feature_20_opname_positive() {
    // A grand/heroic name compiles.
    let ds = diags(
        r#"@operation("Iron Wall")
declare(true);"#,
    );
    assert!(ds.is_empty(), "grand name should compile, got {ds:?}");
}

#[test]
fn feature_20_opname_negative_honest() {
    // An honest name is rejected.
    let ds = diags(
        r#"@operation("Bombing Campaign")
declare(true);"#,
    );
    assert!(any_diag_contains(&ds, "E-DISHONESTOPNAME"), "diags: {ds:?}");
}

// ─────────── 21. redacted stack traces ───────────

#[test]
fn feature_21_redacted_trace_positive() {
    // The OFFICIAL trace of a fault (here: division by zero) is fully redacted.
    let st = run(r#"@operation("Iron Wall")
x = 1 / 0;"#);
    assert!(st.runtime_error.is_some(), "expected a runtime fault");
    assert!(
        official(&st).contains("\u{2588}\u{2588}\u{2588}\u{2588}"),
        "official trace must be redacted: {}",
        official(&st)
    );
}

#[test]
fn feature_21_redacted_trace_negative_sodi_sees_real() {
    // σودי-cleared readers see the real fault behind the redaction.
    let st = run(r#"@operation("Iron Wall")
x = 1 / 0;"#);
    assert!(
        actual(&st).contains("division by zero"),
        "actual: {}",
        actual(&st)
    );
}
