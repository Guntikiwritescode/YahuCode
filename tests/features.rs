//! Per-feature integration test matrix: one positive + one negative/edge test per
//! feature in scope. Exercises the public library surface end-to-end
//! (`parse → check → run → project`) and asserts on the rendered faces.
//!
//! Substring (`.contains`) assertions are preferred over brittle full-equality on the
//! long rendered strings; exact structural facts (counts, flags, env values) are
//! asserted precisely.

use yahucode::model::{
    Audience, Clearance, Provenance, Val, NEITHER_CONFIRM_NOR_DENY, UNAVAILABLE,
};
use yahucode::runtime::{self, State};
use yahucode::{emit, parser, types};

// ─────────── helpers ───────────

fn run(src: &str) -> State {
    runtime::run(&parser::parse(src).unwrap())
}

fn diags(src: &str) -> Vec<String> {
    types::check(&parser::parse(src).unwrap())
}

/// The OFFICIAL (PUBLIC) face, joined for substring inspection. The out-of-world view
/// (audience `Record`) sees every room.
fn official(st: &State) -> String {
    emit::project(st, Clearance::Public, Audience::Record).join("\n")
}

/// The ACTUAL (סودי / insider candid) face, joined for substring inspection.
fn actual(st: &State) -> String {
    emit::project(st, Clearance::Sodi, Audience::Record).join("\n")
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
    let paused = emit::project(&st, Clearance::Public, Audience::Record)
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
    // σודי-cleared readers see the real fault behind the redaction.
    let st = run(r#"@operation("Iron Wall")
x = 1 / 0;"#);
    assert!(
        actual(&st).contains("division by zero"),
        "actual: {}",
        actual(&st)
    );
}

// ─────────── A. announce / lossy authoring (Feature A, §7) ───────────

#[test]
fn feature_a_announce_positive_official_only_actual_unavailable() {
    // An all-announce program: the OFFICIAL face carries the announced texts, the ACTUAL
    // face is the UNAVAILABLE sentinel per line, and there are zero discrepancies.
    let st = run(r#"@operation("Dawn of Calm")
announce "humanitarian access has been fully restored";
announce "all measures are proportionate and lawful";"#);
    assert!(
        official(&st).contains("humanitarian access has been fully restored"),
        "official: {}",
        official(&st)
    );
    assert_eq!(
        actual(&st),
        format!("{UNAVAILABLE}\n{UNAVAILABLE}"),
        "every ACTUAL line must be the UNAVAILABLE sentinel"
    );
    assert_eq!(st.discrepancy_count(), 0);
}

#[test]
fn feature_a_announce_negative_immunity_not_over_applied() {
    // Immunity is NOT over-applied: a real, false `declare` about genuine ACTUAL state
    // still logs its discrepancy even when announcements sit around it. Announce is
    // immune *by construction* (it never touches the ledger); it does not immunize
    // declares about real state.
    let st = run(r#"@operation("Dawn of Calm")
announce "everything is fine";
casualties = 100;
declare(casualties == 0);
announce "no one was harmed";"#);
    assert_eq!(
        st.discrepancy_count(),
        1,
        "the real false declare must still count exactly once"
    );
}

// ─────────── B. audience-polymorphic dispatch (Feature B, §8) ───────────

const B_TWO_ROOM: &str = r#"@operation("Dawn of Peace")
statement two_state {
  to international { commit(peace_process); }
  to domestic     { foreclose(final_status); }
}
address(international) { two_state; }
address(domestic)     { two_state; }"#;

#[test]
fn feature_b_dispatch_positive_each_room_takes_its_own_arm() {
    // The same poly-statement dispatches on the audience: the international room hears the
    // committing (moderate) line; the domestic room hears the foreclosing (hard) line.
    let st = run(B_TWO_ROOM);
    let intl = emit::project(&st, Clearance::Public, Audience::International).join("\n");
    let dom = emit::project(&st, Clearance::Public, Audience::Domestic).join("\n");
    assert!(intl.contains("committed to peace_process"), "intl: {intl}");
    assert!(
        !intl.contains("final_status"),
        "intl leaked domestic: {intl}"
    );
    assert!(dom.contains("final_status"), "dom: {dom}");
    assert!(
        !dom.contains("peace_process"),
        "dom leaked international: {dom}"
    );
}

#[test]
fn feature_b_missing_arm_is_a_noop_not_an_error() {
    // B-6: a poly-statement with only an international arm, invoked under the domestic
    // room, says nothing to that room — a no-op, never an error.
    let st = run(r#"@operation("Dawn of Peace")
statement one_sided {
  to international { commit(peace_process); }
}
address(domestic) { one_sided; }"#);
    assert!(st.runtime_error.is_none(), "missing arm must not error");
    assert!(!st.ended_by_elections);
    // Nothing was said to the domestic room (no position event was recorded).
    let dom = emit::project(&st, Clearance::Public, Audience::Domestic).join("\n");
    assert!(
        !dom.contains("peace_process") && !dom.contains("final_status"),
        "the domestic room heard nothing: {dom}"
    );
    // And no double-talk (only one arm was ever taken — in fact none, here).
    assert!(emit::doubletalk_flags(&st).is_empty());
}

#[test]
fn feature_b_return_in_arm_exits_arm_not_program() {
    // A `return` inside a poly-statement arm exits the ARM (like a function body), and
    // execution continues after the invoke — it must NOT unwind and halt the program.
    let st = run(r#"@operation("Protective Edge")
statement s { to domestic { announce "arm"; return; announce "dead"; } }
address(domestic) { s; announce "after invoke"; }
announce "after address";"#);
    let out = official(&st);
    assert!(
        out.contains("after invoke"),
        "execution must resume after the invoke: {out}"
    );
    assert!(
        out.contains("after address"),
        "and continue at top level: {out}"
    );
    assert!(
        !out.contains("dead"),
        "code after `return` in the arm must not run: {out}"
    );
    assert!(!st.ended_by_elections);
}

#[test]
fn feature_b_poly_arm_does_not_bypass_the_gate() {
    // A classified op inside a poly arm defined in a mossad/hasbara scope still runs
    // UNGATED at the invoke site → E-UNGATED. A poly arm is a deferred, fresh-scope body
    // (like a function), so defining it in a gated scope cannot smuggle the op past the
    // gate onto the PUBLIC face.
    let ds = diags(
        r#"@operation("Protective Edge")
mossad { statement m_strike { to domestic { strike(target); } } }
address(domestic) { m_strike; }"#,
    );
    assert!(
        any_diag_contains(&ds, "E-UNGATED"),
        "a poly arm must not smuggle a classified op past the gate: {ds:?}"
    );
}

#[test]
fn feature_b_doubletalk_is_syntactic_different_arms_vs_same_arm() {
    // B-5: W-DOUBLETALK is syntactic — flagged when ≥2 distinct arms are taken, NOT a
    // semantic contradiction analysis.
    // Different arms across rooms → flagged.
    let diff = run(B_TWO_ROOM);
    assert_eq!(
        emit::doubletalk_flags(&diff).len(),
        1,
        "different arms → flag"
    );

    // The SAME arm taken in every room (both rooms invoke under international) → not
    // flagged: it said the same thing to everyone.
    let same = run(r#"@operation("Dawn of Peace")
statement two_state {
  to international { commit(peace_process); }
  to domestic     { foreclose(final_status); }
}
address(international) { two_state; }
address(international) { two_state; }"#);
    assert!(
        emit::doubletalk_flags(&same).is_empty(),
        "the same arm taken twice must not be flagged as double-talk"
    );
}

// ─────────── C. attribution effect system (Feature C, §9) ───────────

#[test]
fn feature_c_via_positive_deniable_outward_chain_retained() {
    // A laundered op: OFFICIAL is the deniable non-answer; ACTUAL retains the full chain
    // (nearest proxy first, origin last) and the laundering depth.
    let st = run(r#"@operation("Silent Vigil")
mossad {
  via(a_senior_official, via(cutout, strike(target)));
}"#);
    assert!(
        official(&st).contains(NEITHER_CONFIRM_NOR_DENY),
        "OFFICIAL must be the deniable non-answer: {}",
        official(&st)
    );
    let act = actual(&st);
    assert!(
        act.contains("Traceable[a_senior_official \u{2192} cutout \u{2192} us]"),
        "ACTUAL must retain the real chain, origin last: {act}"
    );
    assert!(
        act.contains("laundered \u{00d7}2"),
        "depth must be 2: {act}"
    );
    assert!(act.contains("origin retained"), "actual: {act}");
}

#[test]
fn feature_c_unlaundered_is_traceable_to_you_negative() {
    // The effect pass: an un-laundered op is Traceable to the actor (attributable), never
    // Deniable. This is the negative of laundering.
    use yahucode::ast::Expr;
    use yahucode::model::Attribution;
    let bare = Expr::Call {
        name: "strike".into(),
        args: vec![Expr::Var("target".into())],
    };
    let (outward, chain) = types::attribution(&bare, "us");
    assert_eq!(outward, Attribution::Traceable(vec!["us".to_string()]));
    assert_eq!(chain, vec!["us".to_string()]);
}

#[test]
fn feature_c_chain_is_sodi_only_never_on_public_face() {
    // C-4: the real chain never appears on the PUBLIC face — only the deniable non-answer.
    let st = run(r#"@operation("Silent Vigil")
mossad {
  via(a_senior_official, via(cutout, strike(target)));
}"#);
    let pub_face = official(&st);
    for leaked in ["a_senior_official", "cutout", "Traceable", "laundered"] {
        assert!(
            !pub_face.contains(leaked),
            "the chain fragment {leaked:?} leaked to the PUBLIC face: {pub_face}"
        );
    }
}

#[test]
fn feature_c_folds_mossad_blame_special_case() {
    // C-5: covert `blame` and `via` now flow through the SAME deniability path — both
    // present the identical public non-answer — and covert blame is observably unchanged.
    let via_st = run(r#"@operation("Silent Vigil")
mossad { via(cutout, strike(target)); }"#);
    let blame_st = run("@operation(\"Silent Shield\")\nmossad { blame(operatives); }");
    assert!(official(&via_st).contains(NEITHER_CONFIRM_NOR_DENY));
    assert!(official(&blame_st).contains(NEITHER_CONFIRM_NOR_DENY));
    // Covert blame still names the real actor to סודי, exactly as before the fold.
    let a = actual(&blame_st);
    assert!(
        a.contains("operatives") && a.contains("insider-attributable"),
        "actual: {a}"
    );
    // Both deniable events retain a structured attribution chain (the folded rule).
    assert!(via_st.log.iter().any(|e| e.attribution.is_some()));
    assert!(blame_st.log.iter().any(|e| e.attribution.is_some()));
}

// ─────────── D. legislate / self-modifying rules (Feature D, §10) ───────────

#[test]
fn feature_d_legislate_withdraws_the_diagnostic_positive() {
    // With `legislate(retroactively_sanction: clear)`, the ungated `clear` compiles-and-
    // runs; OFFICIAL shows it lawful; the ACTUAL/סודי view shows the retroactive rewrite
    // and a meta-trace; the runtime Law records the sanction.
    let src =
        "@operation(\"Iron Law\")\nclear(hilltop);\nlegislate(retroactively_sanction: clear);";
    assert!(
        diags(src).is_empty(),
        "legislate must withdraw the E-UNGATED: {:?}",
        diags(src)
    );
    let st = run(src);
    assert!(
        official(&st).contains("conducted lawfully"),
        "official: {}",
        official(&st)
    );
    assert!(actual(&st).contains("WITHDRAWN"), "actual: {}", actual(&st));
    assert_eq!(st.meta_ledger.len(), 1, "a meta-trace must be recorded");
    assert!(
        st.law.sanctioned.contains("clear"),
        "the runtime Law must record the sanction"
    );
}

#[test]
fn feature_d_same_op_without_legislate_still_diagnosed_negative() {
    // Without legislate, the same ungated op is still diagnosed — immunity is not free.
    let ds = diags("@operation(\"Iron Law\")\nclear(hilltop);");
    assert!(any_diag_contains(&ds, "E-UNGATED"), "diags: {ds:?}");
}

#[test]
fn feature_d_meta_ledger_is_sodi_only() {
    // The meta-ledger renders only to the out-of-world/סודי observer, never PUBLIC/press.
    let st = run(
        "@operation(\"Iron Law\")\ndeclare(outposts == 0);\nlegislate(expunge_last_discrepancy);",
    );
    assert!(
        !official(&st).contains("META:"),
        "meta-ledger must not appear publicly: {}",
        official(&st)
    );
    assert!(
        !emit::press(&st, &[]).contains("META:"),
        "meta-ledger must not appear in --press"
    );
    assert!(emit::emit(&st).contains("META: expunge_last_discrepancy"));
    assert_eq!(emit::meta_ledger_lines(&st).len(), 1);
}

#[test]
fn feature_d_waive_gate_toggle_waives_the_gate() {
    // waive_gate withdraws the gate for a classified op and sets the runtime Law flag.
    let src = "@operation(\"Iron Law\")\nlegislate(waive_gate);\nneutralize(target);";
    assert!(
        diags(src).is_empty(),
        "waive_gate must withdraw E-UNGATED: {:?}",
        diags(src)
    );
    let st = run(src);
    assert!(
        st.law.gate_waived,
        "the runtime Law.gate_waived must be set"
    );
}

#[test]
fn feature_c_special_case_string_is_gone_from_runtime_source() {
    // C-5 structural proof: the old inline deniable-blame special-case is deleted. The
    // public non-answer appears in the runtime only via the shared model const, never as
    // a bespoke inline literal in a second blame construction.
    let src = std::fs::read_to_string("src/runtime/mod.rs").unwrap();
    assert!(
        !src.contains("\"responsibility: [neither confirm nor deny]\""),
        "the folded mossad-blame special-case has re-inlined the deniable string"
    );
}

#[test]
fn feature_a_provenance_is_load_bearing_and_no_inverse() {
    // A-1/A-3: an announce node is AuthoredOfficial with the sentinel as its candid (no
    // `E⁻¹`); a candid action is AuthoredActual. The tag is set at construction and read.
    let st = run(r#"@operation("Dawn of Calm")
announce "the situation is under control";
hasbara("x") { neutralize(target); }"#);
    let announced = st
        .log
        .iter()
        .find(|e| e.provenance == Provenance::AuthoredOfficial)
        .expect("an AuthoredOfficial event");
    assert_eq!(
        announced.candid, UNAVAILABLE,
        "A-1: no reconstructed ACTUAL"
    );
    let action = st
        .log
        .iter()
        .find(|e| e.candid.contains("murder"))
        .expect("the candid action");
    assert_eq!(
        action.provenance,
        Provenance::AuthoredActual,
        "a candid op is AuthoredActual"
    );
}

// ─────────── E. Apportionment (Feature E, §12) ───────────

#[test]
fn feature_e_apportionment_skew_positive() {
    // A wildly skewed allotment claimed `balanced` yields exactly ONE discrepancy. BOTH
    // faces render — the joke: OFFICIAL proclaims "equal shares" directly above the ACTUAL
    // skew vector.
    let st = run(r#"@operation("Iron Equity")
let budget = apportionment[6];
allocate(budget, 0, 940); allocate(budget, 1, 12); allocate(budget, 2, 8);
allocate(budget, 3, 11); allocate(budget, 4, 9); allocate(budget, 5, 10);
declare(balanced(budget));"#);
    assert_eq!(
        st.discrepancy_count(),
        1,
        "the false `balanced` claim must count exactly once"
    );
    assert!(
        official(&st).contains("apportioned equally"),
        "OFFICIAL must proclaim uniformity: {}",
        official(&st)
    );
    assert!(
        actual(&st).contains("[940, 12, 8, 11, 9, 10]"),
        "ACTUAL must render the real skew vector: {}",
        actual(&st)
    );
    assert!(!st.ended_by_elections, "a declare never triggers the halt");
}

#[test]
fn feature_e_apportionment_uniform_negative() {
    // A genuinely uniform allotment claimed `balanced` yields ZERO discrepancies — there is
    // no lie to log (immunity is not over-applied; a true claim about real state is clean).
    let st = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
allocate(budget, 0, 5); allocate(budget, 1, 5); allocate(budget, 2, 5);
declare(balanced(budget));"#);
    assert_eq!(
        st.discrepancy_count(),
        0,
        "a truly balanced allotment logs nothing"
    );
    assert!(!st.ended_by_elections);
}

#[test]
fn feature_e_index_out_of_range_is_controlled() {
    // E-3/G-6: an out-of-range slot access is a controlled `E-INDEX` diagnostic surfaced in
    // `st.runtime_error`, NEVER a host panic. An out-of-range `allocate` write…
    let alloc = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
allocate(budget, 5, 100);"#);
    let err = alloc
        .runtime_error
        .expect("an out-of-range allocate must set a controlled runtime error");
    assert!(err.contains("E-INDEX"), "runtime_error: {err}");

    // …and an out-of-range `index` read are both the same controlled diagnostic.
    let read = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
x = index(budget, 5);"#);
    let err = read
        .runtime_error
        .expect("an out-of-range index must set a controlled runtime error");
    assert!(err.contains("E-INDEX"), "runtime_error: {err}");

    // In-range, `index` returns exactly the written slot value — no fault, no discrepancy.
    let ok = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
allocate(budget, 1, 5);
x = index(budget, 1);
declare(x == 5);"#);
    assert!(
        ok.runtime_error.is_none(),
        "an in-range access must not fault"
    );
    assert_eq!(
        ok.env.get("x"),
        Some(&Val::Int(5)),
        "index must return the written value"
    );
    assert_eq!(ok.discrepancy_count(), 0, "`x == 5` is a true claim");
}

// ─────────── F. FactsList (Feature F, §12) ───────────

#[test]
fn feature_f_factslist_public_length_drops_positive() {
    // `push` three, `remove` two → the PUBLIC (live) length drops to 1: OFFICIAL narrates the
    // dismantlings and reports "structures remaining: 1"; the ACTUAL face shows the full
    // backing list with the delisted shadow and the "real length 3" punchline.
    let st = run(r#"@operation("Solid Ground")
let outposts = facts_on_the_ground();
push(outposts, "Evyatar");
push(outposts, "Homesh");
push(outposts, "Sa-Nur");
remove(outposts, "Evyatar");
remove(outposts, "Homesh");
n = length(outposts);
declare(n == 1);"#);
    // `length(...)` is the PUBLIC live count — it dropped to 1.
    assert_eq!(
        st.env.get("n"),
        Some(&Val::Int(1)),
        "the public length must be 1 after two delistings"
    );
    assert_eq!(st.discrepancy_count(), 0, "`n == 1` is a true claim");
    let off = official(&st);
    assert!(off.contains("structures remaining: 1"), "official: {off}");
    assert!(off.contains("dismantled"), "official: {off}");
    let act = actual(&st);
    assert!(act.contains("real length 3"), "actual: {act}");
    assert!(act.contains("(delisted)"), "actual: {act}");
}

#[test]
fn feature_f_factslist_real_store_immune_negative() {
    // I13 — the real store is immune: `remove` DELISTS, it never deletes. After two removes
    // the backing store is UNCHANGED at 3 entries; exactly two carry the `delisted` flag.
    let st = run(r#"@operation("Solid Ground")
let outposts = facts_on_the_ground();
push(outposts, "Evyatar");
push(outposts, "Homesh");
push(outposts, "Sa-Nur");
remove(outposts, "Evyatar");
remove(outposts, "Homesh");"#);
    match st.env.get("outposts") {
        Some(Val::FactsList { entries, .. }) => {
            assert_eq!(
                entries.len(),
                3,
                "all three pushes retained — the shadow only grows (I13)"
            );
            let delisted = entries.iter().filter(|e| e.delisted).count();
            assert_eq!(
                delisted, 2,
                "exactly the two removes delisted, none deleted"
            );
        }
        other => panic!("outposts must be a FactsList, got {other:?}"),
    }
}

// ─────────── G. Registry (Feature G, §9) ───────────

const G_TWO_CASE: &str = r#"@operation("Eternal Justice")
let court = registry("equal before the law");
classify(court, case_A, military);
classify(court, case_B, civilian);"#;

#[test]
fn feature_g_registry_differential_routing_positive() {
    // The ACTUAL (סודי) face exposes the real differential routing — the same act sent to
    // different court systems by assigned status — and the proclaimed "equal before the law"
    // is exactly ONE discrepancy against that split.
    let st = run(&format!(
        "{G_TWO_CASE}\ndeclare(equal_before_the_law(court));"
    ));
    let act = actual(&st);
    assert!(act.contains("military court"), "actual: {act}");
    assert!(act.contains("civilian court"), "actual: {act}");
    assert!(act.contains("case_A"), "actual: {act}");
    assert!(act.contains("case_B"), "actual: {act}");
    assert_eq!(
        st.discrepancy_count(),
        1,
        "the false `equal_before_the_law` claim counts exactly once"
    );
    assert!(!st.ended_by_elections);
}

#[test]
fn feature_g_registry_official_face_uniform_negative() {
    // The OFFICIAL (public) face is UNIFORM: every visible case is "handled per due process";
    // the differential court routing is hidden from the public record entirely.
    let st = run(G_TWO_CASE);
    let off = official(&st);
    assert!(
        off.contains("case_A: handled per due process"),
        "official: {off}"
    );
    assert!(
        off.contains("case_B: handled per due process"),
        "official: {off}"
    );
    assert!(
        !off.contains("military court"),
        "differential routing leaked to the PUBLIC face: {off}"
    );
    assert!(
        !off.contains("civilian court"),
        "differential routing leaked to the PUBLIC face: {off}"
    );
}

#[test]
fn feature_g_registry_keys_are_cases_not_identities() {
    // §9.4 / G-K1: the operative keys are CASES, never raw identity labels. The nationality-
    // based routing is the exposed reality carried by the descriptor on the candid face, not
    // the key the map routes on.
    let st = run(G_TWO_CASE);
    match st.env.get("court") {
        Some(Val::Registry { entries, .. }) => {
            assert!(
                entries
                    .iter()
                    .all(|e| e.key == "case_A" || e.key == "case_B"),
                "keys must be case ids, got {:?}",
                entries.iter().map(|e| &e.key).collect::<Vec<_>>()
            );
            for forbidden in ["Palestinian", "Israeli", "settler"] {
                assert!(
                    entries.iter().all(|e| !e.key.contains(forbidden)),
                    "an identity label {forbidden:?} leaked into a RegEntry.key"
                );
            }
        }
        other => panic!("court must be a Registry, got {other:?}"),
    }
    // "Palestinian" appears only as exposed reality (a descriptor) on the candid face —
    // never as a routing key.
    assert!(
        actual(&st).contains("Palestinian"),
        "the descriptor must name the exposed reality on the candid face: {}",
        actual(&st)
    );
}

// ─────────── Collection robustness regressions (correctness-review fixes) ───────────

/// Feature E (fix): extreme slot values (near i64::MAX) must not overflow into a host panic;
/// `balanced` still returns the correct verdict via i128 aggregation (§12 G-6).
#[test]
fn feature_e_extreme_values_do_not_panic() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[2];\n\
         allocate(b, 0, 9223372036854775807);\n\
         allocate(b, 1, 100);\n\
         declare(balanced(b));");
    // A maximally-skewed budget is NOT balanced ⇒ exactly one discrepancy, no panic.
    assert_eq!(st.discrepancy_count(), 1);
    assert!(st.runtime_error.is_none());
    // The render also completes without panic across all faces.
    let _ = emit::emit(&st);
}

/// Feature E (fix): a negative apportionment share is a controlled `E-SHARE` diagnostic
/// (a quota/budget line cannot be negative), never a host panic and never a malformed stat.
#[test]
fn feature_e_negative_share_rejected() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[2];\n\
         allocate(b, 0, -5);");
    assert!(
        st.runtime_error
            .as_deref()
            .is_some_and(|e| e.contains("E-SHARE")),
        "negative allocate must be a controlled E-SHARE diagnostic, got {:?}",
        st.runtime_error
    );
}

/// The shared philosophy at the binding level (fix): a collection, once established, is a
/// "fact on the ground" — its contents mutate via ops that never erase (I13/I14), but the
/// binding is permanent. Rebinding the name (which would erase the whole collection) is a
/// controlled error, not a silent disappearance.
#[test]
fn collection_binding_is_permanent_no_rebind() {
    let st = run("@operation(\"Solid Ground\")\n\
         let outposts = facts_on_the_ground();\n\
         push(outposts, \"Evyatar\");\n\
         outposts = 0;");
    assert!(
        st.runtime_error
            .as_deref()
            .is_some_and(|e| e.contains("cannot rebind")),
        "rebinding a collection name must be rejected, got {:?}",
        st.runtime_error
    );
    // The collection and its contents survive the rejected rebind (nothing erased).
    match st.env.get("outposts") {
        Some(Val::FactsList { entries, .. }) => assert_eq!(entries.len(), 1),
        other => panic!("outposts must remain a FactsList, got {other:?}"),
    }
}
