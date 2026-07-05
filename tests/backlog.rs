//! Per-feature matrix for the §8 backlog features (reopened): the on-aim
//! government-rhetoric maneuvers, plus comments / the `--press` build. Every feature
//! gets a positive test and a negative/edge test. Polarity (OFFICIAL is the prettier
//! lie) and inertness (these maneuvers change nothing in ACTUAL) are asserted.

use yahucode::{emit, parser, runtime};

fn run(src: &str) -> runtime::State {
    runtime::run(&parser::parse(src).unwrap())
}

/// The OFFICIAL + ACTUAL faces of the last logged event.
fn last_faces(st: &runtime::State) -> (String, String) {
    let ev = st.log.last().unwrap();
    (ev.official.clone(), ev.candid.clone())
}

// ─────────── proportionate (self-certifying assertion) ───────────

#[test]
fn proportionate_self_certifies_and_never_dissents() {
    let st = run("@operation(\"Solid Rock\")\nproportionate(response);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("deemed proportionate"));
    assert!(actual.contains("self-certified") && actual.contains("any magnitude passes"));
    // Negative: it is not a real assertion — it logs no discrepancy at any magnitude.
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
}

// ─────────── disputed (contested figure) ───────────

#[test]
fn disputed_shows_press_number_officially_and_real_number_in_actual() {
    let st = run("@operation(\"Solid Rock\")\ndisputed(units, 100, 900);");
    let (official, actual) = last_faces(&st);
    assert_eq!(official, "units: 100"); // the press number
    assert!(actual.contains("900") && actual.contains("lowballs"));
    // Negative / polarity: the OFFICIAL face is the prettier (lower) figure and never
    // exposes the larger real one.
    assert!(!official.contains("900"));
}

// ─────────── deny (official denial vs ACTUAL occurrence) ───────────

#[test]
fn deny_officially_denies_what_actually_happened() {
    let st = run("@operation(\"Solid Rock\")\ndeny(airstrike);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("categorically deny"));
    assert!(actual.contains("occurred in ACTUAL") && actual.contains("non-occurrence"));
    // Negative: the OFFICIAL denial never admits the occurrence.
    assert!(!official.contains("occurred"));
}

// ─────────── world_opinion / polls (read-only, inert) ───────────

#[test]
fn world_opinion_and_polls_are_read_only_and_inert() {
    let st = run("@operation(\"Solid Rock\")\nworld_opinion();\npolls();");
    let actual_all: String = st.log.iter().map(|e| e.candid.clone()).collect();
    assert!(actual_all.contains("inert"));
    // Negative / edge: genuinely inert — no discrepancy, no state change, no halt.
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
    assert_eq!(st.core, 3);
}

// ─────────── investigate (self-exonerating) ───────────

#[test]
fn investigate_self_exonerates_by_construction() {
    let st = run("@operation(\"Solid Rock\")\ninvestigate(the_incident);");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("investigation opened"));
    assert!(
        actual.contains("no wrongdoing found")
            && actual.contains("investigated investigates itself")
    );
    // Negative: the OFFICIAL face announces an inquiry; it never reveals the rigged outcome.
    assert!(!official.contains("no wrongdoing"));
}

// ─────────── address_international (hollow no-op) ───────────

#[test]
fn address_international_is_a_hollow_no_op() {
    let st = run("@operation(\"Solid Rock\")\nx = 1;\naddress_international();");
    let (official, actual) = last_faces(&st);
    assert!(official.contains("international community was addressed"));
    assert!(actual.contains("void") && actual.contains("no change to ACTUAL"));
    // Negative: it changes nothing — `x` is untouched.
    assert_eq!(st.env.get("x"), Some(&yahucode::model::Val::Int(1)));
}

// ─────────── comments + the --press build (#8 comment-rewriting) ───────────

#[test]
fn comments_are_collected_and_do_not_affect_execution() {
    let prog = parser::parse(
        "@operation(\"Pillar of Defense\")\n# bomb the outpost\nhasbara(\"x\") { neutralize(target); }",
    )
    .unwrap();
    assert_eq!(prog.comments, vec!["bomb the outpost".to_string()]);
    // A commented program runs identically to an uncommented one.
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
    // The honest comment is rewritten: bomb→strike, raze→clearing operation. The build
    // shows the before → after so the laundering (docs-contradict-code) is visible.
    assert!(press.contains("# we strike and clearing operation"));
    assert!(press.contains('\u{2192}')); // the → laundering arrow
                                         // But the press build still shows the OFFICIAL press-release face …
    assert!(press.contains("accident occur(protester)"));
    // … and never the ACTUAL candid truth (the press build is PUBLIC-only).
    assert!(!press.contains("murder(dissident)"));
}

// ─────────── polarity (G3/I1) across the whole backlog set ───────────

#[test]
fn backlog_maneuvers_preserve_polarity() {
    // For every backlog maneuver, the OFFICIAL face is the prettier lie: none of the
    // ugly ACTUAL keywords leak into any OFFICIAL face.
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
