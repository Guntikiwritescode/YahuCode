use yahucode::{emit, parser, runtime, types};

fn run(src: &str) -> runtime::State {
    runtime::run(&parser::parse(src).unwrap())
}

fn last_faces(st: &runtime::State) -> (String, String) {
    let ev = st.log.last().unwrap();
    (ev.official.clone(), ev.candid.clone())
}

#[test]
fn proportionate_self_certifies_and_never_dissents() {
    let st = run("@operation(\"Solid Rock\")\nproportionate(response);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("deemed proportionate"));
    assert!(actual.contains("self-certified") && actual.contains("any magnitude passes"));
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
}

#[test]
fn disputed_shows_press_number_officially_and_real_number_in_actual() {
    let st = run("@operation(\"Solid Rock\")\ndisputed(units, 100, 900);");
    let (official, actual) = last_faces(&st);
    assert_eq!(official, "units: 100");
    assert!(actual.contains("900") && actual.contains("lowballs"));
    assert!(!official.contains("900"));
}

#[test]
fn deny_officially_denies_what_actually_happened() {
    let st = run("@operation(\"Solid Rock\")\ndeny(airstrike);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("categorically deny"));
    assert!(actual.contains("occurred in ACTUAL") && actual.contains("officially denied"));
    assert!(!official.contains("occurred"));
}

#[test]
fn world_opinion_and_polls_are_read_only_and_inert() {
    let st = run("@operation(\"Solid Rock\")\nworld_opinion();\npolls();");
    let actual_all: String = st.log.iter().map(|e| e.candid.clone()).collect();
    assert!(actual_all.contains("inert"));
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
    assert_eq!(st.core, 3);
}

#[test]
fn investigate_self_exonerates_by_construction() {
    let st = run("@operation(\"Solid Rock\")\ninvestigate(the_incident);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("investigation opened"));
    assert!(actual.contains("no wrongdoing found") && actual.contains("self-investigation"));
    assert!(!official.contains("no wrongdoing"));
}

#[test]
fn address_international_is_a_hollow_no_op() {
    let st = run("@operation(\"Solid Rock\")\nx = 1;\naddress_international();");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("international community was addressed"));
    assert!(actual.contains("void") && actual.contains("no change to ACTUAL"));
    assert_eq!(st.env.get("x"), Some(&yahucode::model::Val::Int(1)));
}

#[test]
fn comments_are_collected_and_do_not_affect_execution() {
    let prog = parser::parse(
        "@operation(\"Pillar of Defense\")\n# bomb the outpost\nhasbara(\"x\") { neutralize(target); }",
    )
    .unwrap();
    assert_eq!(prog.comments, vec!["bomb the outpost".to_string()]);
    let commented =
        run("@operation(\"Pillar of Defense\")\n# a note\nhasbara(\"x\") { neutralize(target); }");
    let plain = run("@operation(\"Pillar of Defense\")\nhasbara(\"x\") { neutralize(target); }");
    assert_eq!(emit::emit(&commented), emit::emit(&plain));
}

#[test]
fn press_build_launders_honest_comments_into_euphemisms() {
    let prog = parser::parse(
        "@operation(\"Pillar of Defense\")\n# we bomb and raze\nhasbara(\"x\") { neutralize(target); }",
    )
    .unwrap();
    let st = runtime::run(&prog);
    let press = emit::press(&st, &prog.comments);
    assert!(press.contains("# we strike and clearing operation"));
    assert!(press.contains('\u{2192}'));
    assert!(press.contains("accident occur(protester)"));
    assert!(!press.contains("murder(dissident)"));
}

#[test]
fn disputed_narration_matches_the_numeric_direction() {
    let low = run("@operation(\"Solid Rock\")\ndisputed(x, 100, 900);");
    assert!(low.log.last().unwrap().candid.contains("lowballs"));
    let high = run("@operation(\"Solid Rock\")\ndisputed(x, 900, 100);");
    let c = &high.log.last().unwrap().candid;
    assert!(
        c.contains("inflates"),
        "900 vs 100 must read 'inflates': {c}"
    );
    assert!(!c.contains("lowballs"));
}

#[test]
fn contested_identifier_is_a_compile_error_not_a_panic() {
    for src in [
        "@operation(\"Solid Rock\")\ndeny(genocide);",
        "@operation(\"Iron Wall\")\nproportionate(apartheid);",
        "@operation(\"Iron Wall\")\ninvestigate(genocide);",
        "@operation(\"Iron Wall\")\nx = genocide;\ndeclare(x == 0);",
    ] {
        let diags = types::check(&parser::parse(src).unwrap());
        assert!(
            diags.iter().any(|d| d.starts_with("E-CONTESTED")),
            "expected E-CONTESTED for {src:?}, got {diags:?}"
        );
    }
    assert!(
        types::check(&parser::parse("@operation(\"Solid Rock\")\ndeny(airstrike);").unwrap())
            .is_empty()
    );
}

#[test]
fn rhetorical_statements_keep_official_prettier_than_actual() {
    let st = run("@operation(\"Solid Rock\")\n\
         proportionate(r); disputed(u, 1, 9); deny(strike_event);\n\
         world_opinion(); polls(); investigate(inc); address_international();");
    let ugly = [
        "self-certified",
        "lowballs",
        "occurred in ACTUAL",
        "inert",
        "no wrongdoing",
        "void",
    ];
    for ev in &st.log {
        for u in ugly {
            assert!(
                !ev.official.contains(u),
                "OFFICIAL face leaked the ugly truth {u:?}: {:?}",
                ev.official
            );
        }
    }
}
