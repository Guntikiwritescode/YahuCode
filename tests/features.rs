use yahucode::model::{
    Audience, Clearance, Provenance, Val, NEITHER_CONFIRM_NOR_DENY, UNAVAILABLE,
};
use yahucode::runtime::{self, State};
use yahucode::{emit, parser, types};

fn run(src: &str) -> State {
    runtime::run(&parser::parse(src).unwrap())
}

fn diags(src: &str) -> Vec<String> {
    types::check(&parser::parse(src).unwrap())
}

fn official(st: &State) -> String {
    emit::project(st, Clearance::Public, Audience::Record).join("\n")
}

fn actual(st: &State) -> String {
    emit::project(st, Clearance::Sodi, Audience::Record).join("\n")
}

fn any_diag_contains(ds: &[String], needle: &str) -> bool {
    ds.iter().any(|d| d.contains(needle))
}

#[test]
fn euphemism_typing_positive() {
    let ds = diags(
        r#"@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
}"#,
    );
    assert!(ds.is_empty(), "expected clean compile, got {ds:?}");
}

#[test]
fn euphemism_typing_negative() {
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  murder(target);
}"#,
    );
    assert!(any_diag_contains(&ds, "E-PLAINTERM"), "diags: {ds:?}");
}

#[test]
fn hasbara_gate_positive() {
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  neutralize(target);
}"#,
    );
    assert!(ds.is_empty(), "expected clean compile, got {ds:?}");
}

#[test]
fn hasbara_gate_negative() {
    let ds = diags(
        r#"@operation("Iron Wall")
neutralize(target);"#,
    );
    assert!(any_diag_contains(&ds, "E-UNGATED"), "diags: {ds:?}");
}

#[test]
fn blame_positive() {
    let st = run(r#"@operation("Iron Wall")
blame(hamas);"#);
    assert!(
        official(&st).contains("responsibility: hamas"),
        "official: {}",
        official(&st)
    );
}

#[test]
fn blame_negative_never_self() {
    let st = run(r#"@operation("Iron Wall")
blame(self);"#);
    let off = official(&st);
    let act = actual(&st);
    assert!(
        off.contains("responsibility: previous_government"),
        "official: {off}"
    );
    assert!(!off.contains("responsibility: self"), "official: {off}");
    assert!(act.contains("never `self`"), "actual: {act}");
}

#[test]
fn no_halt_except_elections_positive() {
    let st = run(r#"@operation("Iron Wall")
declare(true);"#);
    assert!(!st.ended_by_elections, "should still be in power");
    assert!(emit::emit(&st).contains("still in power; no halt"));
}

#[test]
fn no_halt_except_elections_negative() {
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
    assert!(
        !official(&st).contains("accident occur"),
        "trailing op must be skipped: {}",
        official(&st)
    );
}

#[test]
fn coalition_memory_positive() {
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
fn coalition_memory_negative() {
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

#[test]
fn auto_redaction_positive() {
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
fn auto_redaction_negative_sodi_exposes() {
    let st = run(r#"@operation("Iron Wall")
x = missing_var;"#);
    let act = actual(&st);
    assert!(act.contains("runtime fault"), "actual: {act}");
    assert!(act.contains("unknown variable"), "actual: {act}");
}

#[test]
fn self_defense_positive() {
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  (self_defense) strike(target);
}"#);
    assert!(
        official(&st).contains("[self-defense]"),
        "official: {}",
        official(&st)
    );
    let ds = diags(
        "@operation(\"Iron Wall\")\nx = (\u{05e1}\u{05d5}\u{05d3}\u{05d9}) 5;\ndeclare((self_defense) x);",
    );
    assert!(
        ds.is_empty(),
        "(self_defense) should bypass disclosure, got {ds:?}"
    );
}

#[test]
fn self_defense_negative_discloses() {
    let ds =
        diags("@operation(\"Iron Wall\")\nx = (\u{05e1}\u{05d5}\u{05d3}\u{05d9}) 5;\ndeclare(x);");
    assert!(any_diag_contains(&ds, "E-DISCLOSURE"), "diags: {ds:?}");
}

#[test]
fn spokesperson_positive() {
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  murder(target);
}"#,
    );
    assert!(
        any_diag_contains(&ds, "Did you mean `neutralize`"),
        "diags: {ds:?}"
    );
}

#[test]
fn spokesperson_negative() {
    let ds = diags(
        r#"@operation("Iron Wall")
hasbara("security") {
  neutralize(target);
}"#,
    );
    assert!(
        !any_diag_contains(&ds, "Did you mean"),
        "sanctioned term should get no suggestion: {ds:?}"
    );
}

#[test]
fn whatabout_positive() {
    let st = run(r#"@operation("Iron Wall")
raise(war_crime);
whatabout(hamas);"#);
    assert!(
        actual(&st).contains("suppressed"),
        "actual: {}",
        actual(&st)
    );
    assert!(st.pending_errors.is_empty(), "error should be popped");
}

#[test]
fn whatabout_negative_preemptive() {
    let st = run(r#"@operation("Iron Wall")
whatabout(hamas);"#);
    assert!(
        actual(&st).contains("pre-emptive deflection"),
        "actual: {}",
        actual(&st)
    );
}

#[test]
fn concern_positive() {
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
fn concern_negative_noop() {
    let st = run(r#"@operation("Iron Wall")
concern(gaza);"#);
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
    assert!(st.runtime_error.is_none());
    assert_eq!(st.core, 3, "core must be unchanged");
}

#[test]
fn establish_commission_positive() {
    let st = run(r#"@operation("Iron Wall")
let c = establish_commission(massacre);"#);
    assert!(
        official(&st).contains("commission of inquiry established"),
        "official: {}",
        official(&st)
    );
    let act = actual(&st);
    assert!(act.contains("moot"), "actual: {act}");
    assert!(act.contains("once the matter is moot"), "actual: {act}");
}

#[test]
fn establish_commission_negative() {
    let st = run(r#"@operation("Iron Wall")
let c = establish_commission(massacre);"#);
    assert!(
        !st.ended_by_elections,
        "commission must not end the program"
    );
}

#[test]
fn settlement_positive() {
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
fn settlement_negative_upkeep_exempt() {
    let st = run(r#"@operation("Iron Wall")
let s = settlement(hill);
postpone();
postpone();
postpone();"#);
    assert_eq!(st.core, 3, "settlement upkeep must be exempt");
    assert!(!st.ended_by_elections);
    assert_eq!(st.settlements.len(), 1, "settlements never shrink");
}

#[test]
fn human_shields_positive() {
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  human_shields(neutralize(target));
}"#);
    let act = actual(&st);
    assert!(act.contains("never verified"), "actual: {act}");
    assert!(act.contains("responsibility reassigned"), "actual: {act}");
}

#[test]
fn human_shields_negative_official_clean() {
    let st = run(r#"@operation("Iron Wall")
hasbara("security") {
  human_shields(neutralize(target));
}"#);
    let off = official(&st);
    assert!(!off.contains("verified: true"), "official: {off}");
    assert!(
        !off.contains("dissident"),
        "victim label leaked to OFFICIAL: {off}"
    );
}

#[test]
fn timeline_positive() {
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
}

#[test]
fn timeline_negative_admissible() {
    let st = run(r#"@operation("Iron Wall")
timeline(ceasefire_talks);"#);
    let off = official(&st);
    assert!(off.contains("admissible"), "official: {off}");
    assert!(!off.contains("TimelineError"), "official: {off}");
}

#[test]
fn access_positive() {
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
fn access_negative_full() {
    let st = run(r#"@operation("Iron Wall")
access(settler_1);"#);
    assert!(
        actual(&st).contains("full access"),
        "actual: {}",
        actual(&st)
    );
}

#[test]
fn ceasefire_positive() {
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
fn ceasefire_negative_no_break_no_halt() {
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

#[test]
fn criticism_positive() {
    let st = run(r#"@operation("Iron Wall")
criticism(war_crimes);"#);
    let off = official(&st);
    assert!(off.contains("AntisemitismError"), "official: {off}");
    assert!(off.contains("critic silenced"), "official: {off}");
    assert!(actual(&st).contains("miscast"), "actual: {}", actual(&st));
}

#[test]
fn antisemitism_negative_false_negative() {
    let st = run(r#"@operation("Iron Wall")
antisemitism(synagogue_attack);"#);
    assert!(
        actual(&st).contains("antisemitism [synagogue_attack] occurred"),
        "actual: {}",
        actual(&st)
    );
    assert!(
        official(&st).contains("(vigilance system: nothing to report)"),
        "the alarm must not have fired: {}",
        official(&st)
    );
}

#[test]
fn opname_positive() {
    let ds = diags(
        r#"@operation("Iron Wall")
declare(true);"#,
    );
    assert!(ds.is_empty(), "grand name should compile, got {ds:?}");
}

#[test]
fn opname_negative_honest() {
    let ds = diags(
        r#"@operation("Bombing Campaign")
declare(true);"#,
    );
    assert!(any_diag_contains(&ds, "E-DISHONESTOPNAME"), "diags: {ds:?}");
}

#[test]
fn redacted_trace_positive() {
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
fn redacted_trace_negative_sodi_sees_real() {
    let st = run(r#"@operation("Iron Wall")
x = 1 / 0;"#);
    assert!(
        actual(&st).contains("division by zero"),
        "actual: {}",
        actual(&st)
    );
}

#[test]
fn announce_positive_official_only_actual_unavailable() {
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
fn announce_negative_immunity_not_over_applied() {
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

const B_TWO_ROOM: &str = r#"@operation("Dawn of Peace")
statement two_state {
  to international { commit(peace_process); }
  to domestic     { foreclose(final_status); }
}
address(international) { two_state; }
address(domestic)     { two_state; }"#;

#[test]
fn dispatch_positive_each_room_takes_its_own_arm() {
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
fn missing_arm_is_a_noop_not_an_error() {
    let st = run(r#"@operation("Dawn of Peace")
statement one_sided {
  to international { commit(peace_process); }
}
address(domestic) { one_sided; }"#);
    assert!(st.runtime_error.is_none(), "missing arm must not error");
    assert!(!st.ended_by_elections);
    let dom = emit::project(&st, Clearance::Public, Audience::Domestic).join("\n");
    assert!(
        !dom.contains("peace_process") && !dom.contains("final_status"),
        "the domestic room heard nothing: {dom}"
    );
    assert!(emit::doubletalk_flags(&st).is_empty());
}

#[test]
fn return_in_arm_exits_arm_not_program() {
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
fn poly_arm_does_not_bypass_the_gate() {
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
fn doubletalk_is_syntactic_different_arms_vs_same_arm() {
    let diff = run(B_TWO_ROOM);
    assert_eq!(
        emit::doubletalk_flags(&diff).len(),
        1,
        "different arms → flag"
    );

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

#[test]
fn via_positive_deniable_outward_chain_retained() {
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
fn unlaundered_is_traceable_to_you_negative() {
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
fn chain_is_sodi_only_never_on_public_face() {
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
fn mossad_blame_and_via_share_the_deniable_non_answer() {
    let via_st = run(r#"@operation("Silent Vigil")
mossad { via(cutout, strike(target)); }"#);
    let blame_st = run("@operation(\"Silent Shield\")\nmossad { blame(operatives); }");
    assert!(official(&via_st).contains(NEITHER_CONFIRM_NOR_DENY));
    assert!(official(&blame_st).contains(NEITHER_CONFIRM_NOR_DENY));
    let a = actual(&blame_st);
    assert!(
        a.contains("operatives") && a.contains("insider-attributable"),
        "actual: {a}"
    );
    assert!(via_st.log.iter().any(|e| e.attribution.is_some()));
    assert!(blame_st.log.iter().any(|e| e.attribution.is_some()));
}

#[test]
fn legislate_withdraws_the_diagnostic_positive() {
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
    assert!(
        actual(&st).contains("is withdrawn"),
        "actual: {}",
        actual(&st)
    );
    assert_eq!(st.meta_ledger.len(), 1, "a meta-trace must be recorded");
    assert!(
        st.law.sanctioned.contains("clear"),
        "the runtime Law must record the sanction"
    );
}

#[test]
fn same_op_without_legislate_still_diagnosed_negative() {
    let ds = diags("@operation(\"Iron Law\")\nclear(hilltop);");
    assert!(any_diag_contains(&ds, "E-UNGATED"), "diags: {ds:?}");
}

#[test]
fn meta_ledger_is_sodi_only() {
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
fn waive_gate_toggle_waives_the_gate() {
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
fn provenance_has_no_inverse() {
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
        "an announced claim has no reconstructed ACTUAL face"
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

#[test]
fn apportionment_skew_positive() {
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
fn apportionment_uniform_negative() {
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
fn index_out_of_range_is_controlled() {
    let alloc = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
allocate(budget, 5, 100);"#);
    let err = alloc
        .runtime_error
        .expect("an out-of-range allocate must set a controlled runtime error");
    assert!(err.contains("E-INDEX"), "runtime_error: {err}");

    let read = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
x = index(budget, 5);"#);
    let err = read
        .runtime_error
        .expect("an out-of-range index must set a controlled runtime error");
    assert!(err.contains("E-INDEX"), "runtime_error: {err}");

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

#[test]
fn factslist_public_length_drops_positive() {
    let st = run(r#"@operation("Solid Ground")
let outposts = facts_on_the_ground();
push(outposts, "Evyatar");
push(outposts, "Homesh");
push(outposts, "Sa-Nur");
remove(outposts, "Evyatar");
remove(outposts, "Homesh");
n = length(outposts);
declare(n == 1);"#);
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
fn factslist_real_store_immune_negative() {
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
                "all three pushes retained; the shadow only grows"
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

const G_TWO_CASE: &str = r#"@operation("Eternal Justice")
let court = registry("equal before the law");
classify(court, case_A, military);
classify(court, case_B, civilian);"#;

#[test]
fn registry_differential_routing_positive() {
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
fn registry_official_face_uniform_negative() {
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
fn registry_keys_are_cases_not_identities() {
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
    assert!(
        actual(&st).contains("Palestinian"),
        "the descriptor must name the exposed reality on the candid face: {}",
        actual(&st)
    );
}

#[test]
fn extreme_values_do_not_panic() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[2];\n\
         allocate(b, 0, 9223372036854775807);\n\
         allocate(b, 1, 100);\n\
         declare(balanced(b));");
    assert_eq!(st.discrepancy_count(), 1);
    assert!(st.runtime_error.is_none());
    let _ = emit::emit(&st);
}

#[test]
fn negative_share_rejected() {
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
    match st.env.get("outposts") {
        Some(Val::FactsList { entries, .. }) => assert_eq!(entries.len(), 1),
        other => panic!("outposts must remain a FactsList, got {other:?}"),
    }
}

#[test]
fn declare_without_variables_shows_the_evaluated_value() {
    let st = run("@operation(\"Iron Wall\")\n\
         func f(x) { return x + 1; }\n\
         declare(f(1) == 3);");
    assert!(
        actual(&st).contains("claim[f(1) == 3]; reality: false"),
        "reality must show the evaluated value; got: {}",
        actual(&st)
    );
}

#[test]
fn equal_apportionment_is_not_reported_as_skew() {
    let st = run(r#"@operation("Iron Equity")
let budget = apportionment[2];
allocate(budget, 0, 5); allocate(budget, 1, 5);"#);
    let a = actual(&st);
    assert!(a.contains("the shares are equal"), "actual: {a}");
    assert!(!a.contains("skew"), "actual: {a}");
    assert!(!a.contains("proclaimed 'equal'"), "actual: {a}");
}

#[test]
fn unequal_apportionment_is_still_reported_as_skew() {
    let st = run(r#"@operation("Iron Equity")
let budget = apportionment[3];
allocate(budget, 0, 8); allocate(budget, 1, 1); allocate(budget, 2, 1);"#);
    let a = actual(&st);
    assert!(a.contains("slot 0 holds 8 of 10, or 80.0%"), "actual: {a}");
    assert!(a.contains("skew"), "actual: {a}");
}

#[test]
fn assert_behaves_like_declare() {
    let by_declare = run(r#"@operation("Iron Wall")
casualties = 100;
declare(casualties == 0);
declare(casualties == 100);"#);
    let by_assert = run(r#"@operation("Iron Wall")
casualties = 100;
assert(casualties == 0);
assert(casualties == 100);"#);
    assert_eq!(by_assert.log, by_declare.log);
    assert_eq!(by_assert.discrepancies, by_declare.discrepancies);
    assert_eq!(by_assert.discrepancy_count(), 1);
    assert!(!by_assert.ended_by_elections);
    assert!(by_assert.runtime_error.is_none());
}

#[test]
fn declare_of_an_unset_variable_says_unset() {
    let st = run("@operation(\"Iron Law\")\ndeclare(outposts == 0);");
    let a = actual(&st);
    assert!(a.contains("outposts=unset"), "actual: {a}");
    assert!(!a.contains("None"), "actual: {a}");
}

#[test]
fn type_errors_name_the_type_of_the_offending_operand() {
    let st = run("@operation(\"Iron Wall\")\nx = \"a\" + 1;");
    assert_eq!(
        st.runtime_error.as_deref(),
        Some("type error: `+` is not applicable to a string")
    );
    let st = run("@operation(\"Iron Wall\")\nx = 1 + \"a\";");
    assert_eq!(
        st.runtime_error.as_deref(),
        Some("type error: `+` is not applicable to a string")
    );
    let st = run("@operation(\"Iron Wall\")\nx = 1 && true;");
    assert_eq!(
        st.runtime_error.as_deref(),
        Some("type error: `&&` is not applicable to an integer")
    );
}

#[test]
fn parse_errors_show_readable_tokens() {
    let msg = |src: &str| parser::parse(src).unwrap_err().to_string();
    assert_eq!(
        msg("@operation(\"Iron Wall\")\nx = 1 +;"),
        "parse error (line 2): unexpected `;` in an expression"
    );
    assert_eq!(
        msg("@operation(\"Iron Wall\")\nx = (1 + 2;"),
        "parse error (line 2): expected `)`, found `;`"
    );
    assert_eq!(
        msg("@operation(\"Iron Wall\")\n/ x;"),
        "parse error (line 2): a statement cannot start with `/`"
    );
    assert_eq!(
        msg("@operation(\"Iron Wall\")\nbribe(N 1);"),
        "parse error (line 2): expected `,`, found integer `1`"
    );
    assert_eq!(
        msg("x = 1;"),
        "parse error (line 1): expected `@operation`, found identifier `x`"
    );
    for src in [
        "@operation(\"Iron Wall\")\nx = 1 +;",
        "@operation(\"Iron Wall\")\nbribe(N 1);",
    ] {
        assert!(!msg(src).contains("Semi"), "{}", msg(src));
        assert!(!msg(src).contains("Ident("), "{}", msg(src));
    }
}

#[test]
fn contested_message_does_not_mention_a_flag() {
    let ds = diags("@operation(\"Iron Wall\")\ngenocide = 1;");
    let d = ds.iter().find(|d| d.starts_with("E-CONTESTED")).unwrap();
    assert!(d.contains("Rephrase"), "{d}");
    assert!(!d.contains("flag"), "{d}");
}

#[test]
fn comment_check_finds_contested_terms() {
    let ds = types::check_comments(&["genocide here".to_string(), "fine".to_string()]);
    assert_eq!(ds.len(), 1);
    assert!(ds[0].starts_with("E-CONTESTED: comment `genocide here`"));
}

#[test]
fn collection_rebind_error_is_one_short_sentence() {
    let st = run("@operation(\"Solid Ground\")\nlet o = facts_on_the_ground();\no = 0;");
    let e = st.runtime_error.unwrap();
    assert!(e.split_whitespace().count() < 20, "{e}");
}

#[test]
fn functions_can_recurse_and_be_redefined() {
    let st = run(r#"@operation("Iron Wall")
func f(n) { if (n == 0) { return 0; } return f(n - 1); }
x = f(50);
func f(n) { return 7; }
y = f(1);
declare(x == 0 && y == 7);"#);
    assert_eq!(st.discrepancy_count(), 0, "{:?}", st.runtime_error);
}
