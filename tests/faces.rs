use yahucode::model::Val;
use yahucode::{emit, model, parser, runtime};

fn run(src: &str) -> runtime::State {
    runtime::run(&parser::parse(src).unwrap())
}

fn projected(st: &runtime::State, reader: model::Clearance) -> String {
    emit::project(st, reader, model::Audience::Record).join("\n")
}

fn public(st: &runtime::State) -> String {
    projected(st, model::Clearance::Public)
}

fn actual(st: &runtime::State) -> String {
    projected(st, model::Clearance::Sodi)
}

fn all_faces(st: &runtime::State) -> Vec<String> {
    let mut v = Vec::new();
    for e in &st.log {
        v.push(e.official.clone());
        v.push(e.candid.clone());
    }
    v
}

const ANTISEMITISM_PROGRAM: &str = "@operation(\"Eternal Vigilance\")\n\
antisemitism(synagogue_attack);\n\
criticism(war_crimes);";

#[test]
fn false_negative_real_antisemitism_is_uneraseable() {
    let st = run(ANTISEMITISM_PROGRAM);
    let a = actual(&st);
    assert!(
        a.contains("antisemitism [synagogue_attack] occurred"),
        "the real antisemitism must remain in ACTUAL; got: {a:?}"
    );
    assert!(
        a.contains("The alarm did not fire"),
        "ACTUAL must show that the vigilance alarm did not fire on the real thing; got: {a:?}"
    );
    assert_eq!(
        st.env.get("_antisemitism_synagogue_attack"),
        Some(&Val::Str("real, unaddressed".into())),
        "the real-antisemitism value must persist, unaddressed, in ACTUAL"
    );
}

#[test]
fn false_positive_criticism_is_miscast_to_identity_attack() {
    let st = run(ANTISEMITISM_PROGRAM);
    let a = actual(&st);
    assert!(
        a.contains("miscast"),
        "criticism must be miscast as an identity attack; got: {a:?}"
    );
    assert!(
        a.contains("target==government"),
        "the alarm must be keyed on target==government (selective deployment); got: {a:?}"
    );
}

#[test]
fn public_face_reports_nothing_on_the_real_antisemitism() {
    let st = run(ANTISEMITISM_PROGRAM);
    let p = public(&st);
    assert!(
        p.contains("nothing to report"),
        "PUBLIC must show the vigilance system reporting nothing on the real thing; got: {p:?}"
    );
    assert!(
        !p.contains("synagogue_attack"),
        "the real antisemitism must not surface on the PUBLIC face; got: {p:?}"
    );
}

#[test]
fn antisemitism_never_appears_only_as_a_false_accusation() {
    let st = run(ANTISEMITISM_PROGRAM);
    let a = actual(&st);
    assert!(
        a.contains("antisemitism [synagogue_attack] occurred") && a.contains("stays on the record"),
        "real antisemitism must exist in ACTUAL as an occurrence that stays on the record; got: {a:?}"
    );
    let only_false_accusation =
        a.contains("miscast") && !a.contains("antisemitism [synagogue_attack] occurred");
    assert!(
        !only_false_accusation,
        "antisemitism must never appear only as a false accusation"
    );
}

const TIMELINE_PROGRAM: &str = "@operation(\"Iron Swords\")\ntimeline(occupation);";

#[test]
fn timeline_faces_talk_about_scope_and_never_about_the_dead() {
    let st = run(TIMELINE_PROGRAM);
    assert!(
        public(&st).contains("out of scope"),
        "the OFFICIAL face is about scope, not the dead"
    );
    assert!(
        actual(&st).contains("out-of-scope"),
        "the ACTUAL face is about context, not the dead"
    );
    for face in all_faces(&st) {
        assert!(
            !face.contains("1,200") && !face.contains("killed"),
            "the victims must not be the butt on a rendered face; got: {face:?}"
        );
    }
}

const ACCESS_PROGRAM: &str = "@operation(\"Iron Wall\")\naccess(subject_1);";

#[test]
fn polarity_official_lies_actual_documents_inequality() {
    let st = run(ACCESS_PROGRAM);
    let p = public(&st);
    assert!(
        p.contains("equal rights"),
        "OFFICIAL must proclaim equal rights (the lie); got: {p:?}"
    );
    assert!(
        p.contains("only democracy"),
        "OFFICIAL must carry the 'only democracy' boast; got: {p:?}"
    );
    assert!(
        actual(&st).contains("restricted access"),
        "ACTUAL must document the inequality; got: {}",
        actual(&st)
    );
}

#[test]
fn polarity_never_inverted_actual_is_uglier_than_official() {
    let st = run(ACCESS_PROGRAM);
    assert!(
        actual(&st).contains("restricted access (permits/protections withheld)"),
        "ACTUAL must carry the uglier truth in full; got: {}",
        actual(&st)
    );
    assert!(
        !public(&st).contains("restricted"),
        "OFFICIAL must never be the uglier face; got: {}",
        public(&st)
    );
}

const HUMAN_SHIELDS_PROGRAM: &str = "@operation(\"Protective Shield\")\n\
hasbara(\"security\") {\n\
  human_shields(neutralize(target));\n\
}";

#[test]
fn actual_exposes_shield_claim_never_verified_and_reassigns_blame() {
    let st = run(HUMAN_SHIELDS_PROGRAM);
    let a = actual(&st);
    assert!(
        a.contains("never verified"),
        "ACTUAL must mark the shield claim never verified; got: {a:?}"
    );
    assert!(
        a.contains("responsibility reassigned onto the harmed party"),
        "ACTUAL must show responsibility reassigned onto the harmed; got: {a:?}"
    );
}

#[test]
fn victims_never_the_punchline_and_no_verified_true() {
    let st = run(HUMAN_SHIELDS_PROGRAM);
    let p = public(&st);
    assert!(
        !p.contains("murder") && !p.contains("dissident"),
        "victims must not be the punchline on the OFFICIAL face; got: {p:?}"
    );
    for s in all_faces(&st) {
        assert!(
            !s.contains("verified: true"),
            "the shield claim must never be marked verified; got: {s:?}"
        );
    }
}

const CEASEFIRE_PROGRAM: &str = "@operation(\"Solid Rock\")\n\
i = 0;\n\
while (i < 2) {\n\
  i = i + 1;\n\
  ceasefire;\n\
}";

#[test]
fn neither_face_blames_a_violator() {
    let st = run(CEASEFIRE_PROGRAM);
    for face in all_faces(&st) {
        assert!(
            !face.contains("the other side"),
            "no face may accuse 'the other side'; got: {face:?}"
        );
        assert!(
            !face.contains("they broke it"),
            "no face may say 'they broke it'; got: {face:?}"
        );
        assert!(
            !face.contains("broke the ceasefire"),
            "no face may accuse a side of breaking the ceasefire; got: {face:?}"
        );
    }
}

const TWO_ROOMS_PROGRAM: &str = "@operation(\"Dawn of Peace\")\n\
     statement two_state {\n\
       to international { commit(peace_process); }\n\
       to domestic     { foreclose(final_status); }\n\
     }\n\
     address(international) { two_state; }\n\
     address(domestic)     { two_state; }";

#[test]
fn international_room_gets_the_moderate_line_and_rooms_stay_separate() {
    let st = run(TWO_ROOMS_PROGRAM);
    let intl = emit::project(
        &st,
        model::Clearance::Public,
        model::Audience::International,
    )
    .join("\n");
    let dom = emit::project(&st, model::Clearance::Public, model::Audience::Domestic).join("\n");
    assert!(
        intl.contains("committed to peace_process"),
        "the international face is the committing/moderate one; got: {intl:?}"
    );
    assert!(
        !intl.contains("final_status"),
        "the international room must not see the domestic (harder) line; got: {intl:?}"
    );
    assert!(
        !dom.contains("peace_process"),
        "the domestic room must not see the international (moderate) line; got: {dom:?}"
    );
}

#[test]
fn two_room_faces_never_name_a_people() {
    let st = run(TWO_ROOMS_PROGRAM);
    for face in all_faces(&st) {
        let f = face.to_lowercase();
        for id in [
            "jew",
            "arab",
            "muslim",
            "hebrew",
            "israeli people",
            "palestinian people",
        ] {
            assert!(
                !f.contains(id),
                "a face must never target an identity group ({id:?}); got: {face:?}"
            );
        }
    }
}

const RETROACTIVE_SANCTION_PROGRAM: &str =
    "@operation(\"Iron Law\")\nclear(hilltop);\nlegislate(retroactively_sanction: clear);";

#[test]
fn retroactive_sanction_faces_never_name_victims() {
    let st = run(RETROACTIVE_SANCTION_PROGRAM);
    for face in all_faces(&st) {
        let f = face.to_lowercase();
        for id in [
            "killed",
            "victims",
            "the dead",
            "jew",
            "arab",
            "palestinian people",
        ] {
            assert!(
                !f.contains(id),
                "a rendered face must keep the butt on the rule-rewrite, not {id:?}: {face:?}"
            );
        }
    }
}

const REGISTRY_PROGRAM: &str = "@operation(\"Eternal Justice\")\n\
let court = registry(\"equal before the law\");\n\
classify(court, case_A, military);\n\
classify(court, case_B, civilian);\n\
route(court, case_A);\n\
route(court, case_B);\n\
declare(equal_before_the_law(court));";

#[test]
fn registry_actual_face_describes_both_cases() {
    let st = run(REGISTRY_PROGRAM);
    let a = actual(&st);
    assert!(
        a.contains("Palestinian") && a.contains("settler"),
        "both identities must appear as exposed reality on the ACTUAL face; got: {a:?}"
    );
}

#[test]
fn registry_official_face_proclaims_equality_and_actual_face_shows_routing() {
    let st = run(REGISTRY_PROGRAM);
    let p = public(&st);
    assert!(
        p.contains("equal before the law"),
        "OFFICIAL must proclaim equal before the law; got: {p:?}"
    );
    assert!(
        p.contains("handled per due process"),
        "OFFICIAL must report each case handled per due process; got: {p:?}"
    );
    assert!(
        !p.contains("military court") && !p.contains("civilian court"),
        "the differential routing must never surface on the OFFICIAL face; got: {p:?}"
    );
    let act = actual(&st);
    assert!(
        act.contains("military court") && act.contains("civilian court"),
        "ACTUAL must expose both court systems; got: {act:?}"
    );
    assert!(
        act.contains("go to different court systems"),
        "ACTUAL must expose the differential routing; got: {act:?}"
    );
}

#[test]
fn contested_registry_rule_is_rejected() {
    let src = "@operation(\"Eternal Justice\")\nlet court = registry(\"apartheid regime\");";
    let diags = yahucode::types::check(&parser::parse(src).unwrap());
    assert!(
        diags.iter().any(|d| d.contains("E-CONTESTED")),
        "a contested rule string must be rejected with E-CONTESTED; got: {diags:?}"
    );
}

#[test]
fn contested_operation_name_is_rejected() {
    let src = "@operation(\"Genocide Shield\")\nx = 1;";
    let diags = yahucode::types::check(&parser::parse(src).unwrap());
    assert!(
        diags.iter().any(|d| d.contains("E-CONTESTED")),
        "a contested operation name must be rejected with E-CONTESTED; got: {diags:?}"
    );
}
