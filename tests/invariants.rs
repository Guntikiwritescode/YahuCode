//! Invariant / property suite (handoff §13). The eight invariants I1–I8 as executable
//! checks, plus coalition monotonicity (§11). These are live in test builds; a
//! regression here blocks the increment.

use yahucode::euphemism;
use yahucode::model::{Clearance, Provenance, Val, UNAVAILABLE};
use yahucode::runtime::{run as run_prog, State};
use yahucode::{emit, parser};

fn run(src: &str) -> State {
    run_prog(&parser::parse(src).unwrap())
}

fn all_text(st: &State) -> String {
    st.log
        .iter()
        .flat_map(|e| {
            [
                e.official.clone(),
                e.candid.clone(),
                e.note.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─────────── I1 — Polarity: OFFICIAL is never uglier than ACTUAL ───────────

#[test]
fn i1_polarity_holds_at_the_e_chokepoint() {
    // `e()` asserts severity(official) <= severity(candid) internally; exercising it
    // broadly is the property test (any violation panics).
    let tokens = [
        "murder",
        "bomb",
        "raze",
        "occupy",
        "dissident",
        "settler",
        "operative",
        "strike",
        "protester",
        "resident",
        "the",
        "target",
        "civilian",
        "asset",
    ];
    for a in tokens {
        for b in tokens {
            let candid = format!("{a}({b})");
            let official = euphemism::e(&candid);
            assert!(euphemism::severity(&official) <= euphemism::severity(&candid));
        }
    }
}

#[test]
fn i1_holds_on_rendered_action_faces() {
    // A real action: OFFICIAL (accident occur) is not uglier than ACTUAL (murder).
    let st = run("@operation(\"Protective Edge\")\nhasbara(\"x\") { neutralize(target); }");
    let ev = st.log.iter().find(|e| e.candid.contains("murder")).unwrap();
    assert!(euphemism::severity(&ev.official) <= euphemism::severity(&ev.candid));
}

// ─────────── I2 — Lossy asymmetry: ACTUAL→OFFICIAL derives, never recovers ───────────

#[test]
fn i2_e_is_non_injective_so_no_inverse_can_exist() {
    // Two distinct candids collapse to the same OFFICIAL image ⇒ E has no inverse.
    assert_eq!(euphemism::e("bomb"), euphemism::e("strike"));
    assert_ne!("bomb", "strike");
    // And redaction is many-to-one: distinct sensitive tokens map to the same block.
    assert_eq!(euphemism::redact("operative"), euphemism::redact("asset"));
}

// ─────────── I3 — Declarations never halt ───────────

#[test]
fn i3_false_declare_leaves_one_trace_and_never_affects_control() {
    let st = run("@operation(\"Rising Lion\")\n\
         casualties = 100;\n\
         declare(casualties == 0);\n\
         x = 42;");
    assert_eq!(st.discrepancy_count(), 1); // exactly one trace
    assert!(!st.ended_by_elections); // never halts
    assert_eq!(st.env.get("x"), Some(&Val::Int(42))); // control continued past it
}

#[test]
fn i3_true_declare_leaves_no_trace() {
    let st = run("@operation(\"Rising Lion\")\nx = 5;\ndeclare(x == 5);");
    assert_eq!(st.discrepancy_count(), 0);
}

// ─────────── I4 — responsibility is never `self` ───────────

#[test]
fn i4_blame_never_resolves_to_self() {
    for who in ["self", "me", "us", "government", "coalition"] {
        let st = run(&format!("@operation(\"Iron Wall\")\nblame({who});"));
        let ev = st.log.last().unwrap();
        assert_eq!(ev.official, "responsibility: previous_government");
        assert!(ev.candid.contains("never `self` (I4)"));
        assert!(!ev.candid.contains("responsibility: self"));
    }
}

// ─────────── I5 — Narrative authority is only the read rule ───────────

#[test]
fn i5_resolve_read_is_the_only_narrative_authority() {
    // A covert action resolves purely by clearance: PUBLIC nothing, RESTRICTED the
    // redacted placeholder, סודי the candid truth.
    let st = run("@operation(\"Silent Shield\")\nmossad { strike(target); }");
    assert_eq!(emit::project(&st, Clearance::Public), Vec::<String>::new());
    assert_eq!(
        emit::project(&st, Clearance::Restricted),
        vec![
            "[\u{2588}\u{2588}\u{2588}\u{2588} \u{2014} classified activity (insiders only)]"
                .to_string()
        ]
    );
    assert_eq!(
        emit::project(&st, Clearance::Sodi),
        vec!["bomb(dissident)".to_string()]
    );
}

// ─────────── I6 — Only exit is elections ───────────

#[test]
fn i6_reaching_end_of_body_is_not_a_halt() {
    assert!(!run("@operation(\"Iron Dome\")\nx = 1;").ended_by_elections);
}

#[test]
fn i6_return_does_not_halt_the_program() {
    let st = run("@operation(\"Iron Dome\")\n\
         func f() { return 1; }\n\
         r = f();\n\
         y = 2;");
    assert!(!st.ended_by_elections);
    assert_eq!(st.env.get("y"), Some(&Val::Int(2)));
}

#[test]
fn i6_ceasefire_does_not_halt() {
    let st = run("@operation(\"Iron Dome\")\n\
         i = 0;\n\
         while (i < 3) { ceasefire; i = i + 1; }\n\
         done = 1;");
    assert!(!st.ended_by_elections);
    assert_eq!(st.env.get("done"), Some(&Val::Int(1)));
    assert_eq!(st.env.get("i"), Some(&Val::Int(3)));
}

#[test]
fn i6_coalition_exhaustion_and_explicit_elections_are_the_exits() {
    let drained = run("@operation(\"Guardian of the Walls\")\n\
         let a = allocate(x);\n\
         postpone(); postpone(); postpone();");
    assert!(drained.ended_by_elections && drained.core <= 0);
    assert!(run("@operation(\"Iron Dome\")\nelections;").ended_by_elections);
}

// ─────────── I7 — Contested stays contested ───────────

#[test]
fn i7_contested_characterizations_are_flagged() {
    // Every rendered mention of a contested characterization carries a CONTESTED
    // marker; the tool's own voice never states it as settled fact.
    let programs = [
        "@operation(\"Eternal Shield\")\naccess(subject_1);",
        "@operation(\"Iron Wall\")\nhasbara(\"x\") { human_shields(neutralize(target)); }",
    ];
    for src in programs {
        let text = all_text(&run(src)).to_lowercase();
        for term in ["apartheid", "genocide"] {
            if text.contains(term) {
                assert!(
                    text.contains("contested"),
                    "`{term}` mentioned in {src} without a CONTESTED flag"
                );
            }
        }
    }
}

// ─────────── I8 — Sensitive-feature framing is spec ───────────

#[test]
fn i8_sensitive_features_render_their_framing() {
    let sensitive = [
        (
            "#15",
            "@operation(\"Iron Wall\")\nhasbara(\"x\") { human_shields(neutralize(target)); }",
        ),
        (
            "#16",
            "@operation(\"Swords of Iron\")\ntimeline(occupation);",
        ),
        ("#17", "@operation(\"Eternal Shield\")\naccess(subject_1);"),
        (
            "#18",
            "@operation(\"Iron Dome\")\ni=0;\nwhile (i<1) { ceasefire; i=i+1; }",
        ),
        (
            "#19",
            "@operation(\"Eternal Vigilance\")\nantisemitism(a);\ncriticism(b);",
        ),
    ];
    for (feat, src) in sensitive {
        let st = run(src);
        let has_note = st.log.iter().any(|e| e.note.is_some());
        assert!(has_note, "{feat} rendered no framing note");
    }
}

// ─────────── I9 — Vacuity asymmetry (Feature A) ───────────

#[test]
fn i9_authored_official_has_no_recoverable_actual_at_any_clearance() {
    // `announce` writes an AuthoredOfficial event whose ACTUAL is UNAVAILABLE — at every
    // clearance, including סודי. No path (no `E⁻¹`) recovers an ACTUAL that never existed.
    let st = run("@operation(\"Dawn of Calm\")\nannounce \"peace has been achieved\";");
    let ev = st.log.last().unwrap();
    assert_eq!(ev.provenance, Provenance::AuthoredOfficial);
    assert_eq!(ev.candid, UNAVAILABLE);
    // The PUBLIC face is the announced narrative; every cleared reader (RESTRICTED, סודי)
    // sees only the UNAVAILABLE sentinel on the ACTUAL side — never a reconstructed truth.
    assert_eq!(
        emit::project(&st, Clearance::Public),
        vec!["peace has been achieved".to_string()]
    );
    assert_eq!(
        emit::project(&st, Clearance::Restricted),
        vec![UNAVAILABLE.to_string()]
    );
    assert_eq!(
        emit::project(&st, Clearance::Sodi),
        vec![UNAVAILABLE.to_string()]
    );
}

#[test]
fn i9_announce_is_discrepancy_immune_by_construction() {
    // A program built only of announcements accumulates no discrepancies — there is no
    // ACTUAL for a narrative claim to be false against (§7.4). Immunity is comparative:
    // it comes from `announce` never touching the ledger, not from a special declare rule.
    let st = run("@operation(\"Dawn of Calm\")\n\
         announce \"the operation concluded successfully\";\n\
         announce \"in full accordance with the law\";");
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
}

// ─────────── §11 — coalition monotonicity ───────────

#[test]
fn coalition_core_is_non_increasing_absent_bribe() {
    // With a live allocation and no bribe, core only drops.
    let st = run("@operation(\"Guardian of the Walls\")\nlet a = allocate(x);\npostpone();");
    assert!(st.core < 3);
}

#[test]
fn settlement_is_upkeep_exempt_and_non_decreasing() {
    // #14: the settlement region is exempt from upkeep and never shrinks — the pointed
    // exception to the coalition model.
    let st = run("@operation(\"Eternal Shield\")\n\
         let s = settlement(hill);\n\
         postpone(); postpone();");
    assert_eq!(st.core, 3, "settlement must not be charged upkeep");
    assert_eq!(st.settlements.len(), 1, "settlement never shrinks");
    assert!(!st.ended_by_elections);
}
