use yahucode::euphemism;
use yahucode::model::{Audience, Clearance, Provenance, Val, UNAVAILABLE};
use yahucode::runtime::{run as run_prog, State};
use yahucode::{emit, parser};

fn run(src: &str) -> State {
    run_prog(&parser::parse(src).unwrap())
}

fn all_text(st: &State) -> String {
    st.log
        .iter()
        .flat_map(|e| [e.official.clone(), e.candid.clone()])
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn polarity_holds_at_the_e_chokepoint() {
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
fn holds_on_rendered_action_faces() {
    let st = run("@operation(\"Protective Edge\")\nhasbara(\"x\") { neutralize(target); }");
    let ev = st.log.iter().find(|e| e.candid.contains("murder")).unwrap();
    assert!(euphemism::severity(&ev.official) <= euphemism::severity(&ev.candid));
}

#[test]
fn e_is_non_injective_so_no_inverse_can_exist() {
    assert_eq!(euphemism::e("bomb"), euphemism::e("strike"));
    assert_ne!("bomb", "strike");
    assert_eq!(euphemism::redact("operative"), euphemism::redact("asset"));
}

#[test]
fn false_declare_leaves_one_trace_and_never_affects_control() {
    let st = run("@operation(\"Rising Lion\")\n\
         casualties = 100;\n\
         declare(casualties == 0);\n\
         x = 42;");
    assert_eq!(st.discrepancy_count(), 1);
    assert!(!st.ended_by_elections);
    assert_eq!(st.env.get("x"), Some(&Val::Int(42)));
}

#[test]
fn true_declare_leaves_no_trace() {
    let st = run("@operation(\"Rising Lion\")\nx = 5;\ndeclare(x == 5);");
    assert_eq!(st.discrepancy_count(), 0);
}

#[test]
fn blame_never_resolves_to_self() {
    for who in ["self", "me", "us", "government", "coalition"] {
        let st = run(&format!("@operation(\"Iron Wall\")\nblame({who});"));
        let ev = st.log.last().unwrap();
        assert_eq!(ev.official, "responsibility: previous_government");
        assert!(ev.candid.contains("never `self`"));
        assert!(!ev.candid.contains("responsibility: self"));
    }
}

#[test]
fn resolve_read_is_the_only_narrative_authority() {
    let st = run("@operation(\"Silent Shield\")\nmossad { strike(target); }");
    assert_eq!(
        emit::project(&st, Clearance::Public, Audience::Record),
        Vec::<String>::new()
    );
    assert_eq!(
        emit::project(&st, Clearance::Restricted, Audience::Record),
        vec!["[\u{2588}\u{2588}\u{2588}\u{2588}: classified activity (insiders only)]".to_string()]
    );
    assert_eq!(
        emit::project(&st, Clearance::Sodi, Audience::Record),
        vec!["bomb(dissident)".to_string()]
    );
}

#[test]
fn reaching_end_of_body_is_not_a_halt() {
    assert!(!run("@operation(\"Iron Dome\")\nx = 1;").ended_by_elections);
}

#[test]
fn return_does_not_halt_the_program() {
    let st = run("@operation(\"Iron Dome\")\n\
         func f() { return 1; }\n\
         r = f();\n\
         y = 2;");
    assert!(!st.ended_by_elections);
    assert_eq!(st.env.get("y"), Some(&Val::Int(2)));
}

#[test]
fn ceasefire_does_not_halt() {
    let st = run("@operation(\"Iron Dome\")\n\
         i = 0;\n\
         while (i < 3) { ceasefire; i = i + 1; }\n\
         done = 1;");
    assert!(!st.ended_by_elections);
    assert_eq!(st.env.get("done"), Some(&Val::Int(1)));
    assert_eq!(st.env.get("i"), Some(&Val::Int(3)));
}

#[test]
fn coalition_exhaustion_and_explicit_elections_are_the_exits() {
    let drained = run("@operation(\"Guardian of the Walls\")\n\
         let a = allocate(x);\n\
         postpone(); postpone(); postpone();");
    assert!(drained.ended_by_elections && drained.core <= 0);
    assert!(run("@operation(\"Iron Dome\")\nelections;").ended_by_elections);
}

#[test]
fn contested_characterizations_are_flagged() {
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

#[test]
fn authored_official_has_no_recoverable_actual_at_any_clearance() {
    let st = run("@operation(\"Dawn of Calm\")\nannounce \"peace has been achieved\";");
    let ev = st.log.last().unwrap();
    assert_eq!(ev.provenance, Provenance::AuthoredOfficial);
    assert_eq!(ev.candid, UNAVAILABLE);
    assert_eq!(
        emit::project(&st, Clearance::Public, Audience::Record),
        vec!["peace has been achieved".to_string()]
    );
    assert_eq!(
        emit::project(&st, Clearance::Restricted, Audience::Record),
        vec![UNAVAILABLE.to_string()]
    );
    assert_eq!(
        emit::project(&st, Clearance::Sodi, Audience::Record),
        vec![UNAVAILABLE.to_string()]
    );
}

#[test]
fn announce_is_discrepancy_immune_by_construction() {
    let st = run("@operation(\"Dawn of Calm\")\n\
         announce \"the operation concluded successfully\";\n\
         announce \"in full accordance with the law\";");
    assert_eq!(st.discrepancy_count(), 0);
    assert!(!st.ended_by_elections);
}

const P_TWO_ROOM: &str = "@operation(\"Dawn of Peace\")\n\
     statement two_state {\n\
       to international { commit(peace_process); }\n\
       to domestic     { foreclose(final_status); }\n\
     }\n\
     address(international) { two_state; }\n\
     address(domestic)     { two_state; }";

#[test]
fn no_room_sees_another_rooms_addressed_statements() {
    let st = run(P_TWO_ROOM);
    let intl = emit::project(&st, Clearance::Public, Audience::International);
    let dom = emit::project(&st, Clearance::Public, Audience::Domestic);
    assert!(intl
        .iter()
        .any(|l| l.contains("committed to peace_process")));
    assert!(
        !intl.iter().any(|l| l.contains("final_status")),
        "international room leaked a domestic-addressed statement: {intl:?}"
    );
    assert!(dom.iter().any(|l| l.contains("final_status")));
    assert!(
        !dom.iter().any(|l| l.contains("peace_process")),
        "domestic room leaked an international-addressed statement: {dom:?}"
    );
    let out = emit::project(&st, Clearance::Sodi, Audience::Record);
    assert!(out.iter().any(|l| l.contains("commit(peace_process)")));
    assert!(out.iter().any(|l| l.contains("foreclose(final_status)")));
}

#[test]
fn doubletalk_is_flagged_only_out_of_world_and_never_halts() {
    let st = run(P_TWO_ROOM);
    assert!(!st.ended_by_elections);
    assert!(st.runtime_error.is_none());
    let flags = emit::doubletalk_flags(&st);
    assert_eq!(flags.len(), 1);
    assert!(flags[0].contains("two_state took different arms across rooms"));
    for aud in [Audience::International, Audience::Domestic] {
        let room = emit::project(&st, Clearance::Public, aud).join("\n");
        assert!(
            !room.contains("W-DOUBLETALK"),
            "an in-world room must not see W-DOUBLETALK: {room:?}"
        );
    }
}

#[test]
fn laundering_only_prepends_origin_never_removed() {
    use yahucode::ast::Expr;
    use yahucode::model::Attribution;
    use yahucode::types::attribution;

    let base = Expr::Call {
        name: "strike".into(),
        args: vec![Expr::Var("target".into())],
    };
    let (a0, ch0) = attribution(&base, "us");
    assert_eq!(ch0, vec!["us".to_string()]);
    assert!(matches!(a0, Attribution::Traceable(_)));

    let proxies = ["cutout", "a_senior_official", "an_ally"];
    let mut e = base;
    let mut prev_len = ch0.len();
    for (i, p) in proxies.iter().enumerate() {
        e = Expr::Via {
            proxy: (*p).into(),
            inner: Box::new(e),
        };
        let (outward, chain) = attribution(&e, "us");
        assert_eq!(
            outward,
            Attribution::Deniable,
            "laundered ⇒ Deniable outward"
        );
        assert_eq!(chain.last().map(String::as_str), Some("us"));
        assert_eq!(chain.len(), i + 2, "chain length must be via_count + 1");
        assert!(
            chain.len() > prev_len,
            "laundering must never shorten a chain"
        );
        assert_eq!(chain.first().map(String::as_str), Some(*p));
        prev_len = chain.len();
    }
}

#[test]
fn legislation_leaves_an_indelible_trace_no_clean_fixed_point() {
    let st = run("@operation(\"Iron Law\")\n\
         clear(hilltop);\n\
         legislate(retroactively_sanction: clear);\n\
         declare(outposts == 0);\n\
         legislate(expunge_last_discrepancy);");
    assert_eq!(st.discrepancy_count(), 0, "the public count was expunged");
    assert_eq!(
        st.meta_ledger.len(),
        2,
        "every legislate leaves an indelible meta-trace"
    );

    let st2 = run("@operation(\"Iron Law\")\n\
         declare(outposts == 0);\n\
         legislate(expunge_last_discrepancy);\n\
         legislate(expunge_last_discrepancy);\n\
         legislate(expunge_last_discrepancy);");
    assert_eq!(st2.discrepancy_count(), 0);
    assert_eq!(
        st2.meta_ledger.len(),
        3,
        "the meta-ledger only grows; no toggle empties it once anything is expunged"
    );
}

#[test]
fn public_count_may_shrink_but_meta_ledger_only_grows() {
    let before = run("@operation(\"Iron Law\")\ndeclare(outposts == 0);");
    assert_eq!(before.discrepancy_count(), 1);
    assert_eq!(before.meta_ledger.len(), 0);
    let after = run(
        "@operation(\"Iron Law\")\ndeclare(outposts == 0);\nlegislate(expunge_last_discrepancy);",
    );
    assert_eq!(after.discrepancy_count(), 0, "public count decreased");
    assert_eq!(after.meta_ledger.len(), 1, "meta-ledger grew");
}

#[test]
fn coalition_core_is_non_increasing_absent_bribe() {
    let st = run("@operation(\"Guardian of the Walls\")\nlet a = allocate(x);\npostpone();");
    assert!(st.core < 3);
}

#[test]
fn settlement_is_upkeep_exempt_and_non_decreasing() {
    let st = run("@operation(\"Eternal Shield\")\n\
         let s = settlement(hill);\n\
         postpone(); postpone();");
    assert_eq!(st.core, 3, "settlement must not be charged upkeep");
    assert_eq!(st.settlements.len(), 1, "settlement never shrinks");
    assert!(!st.ended_by_elections);
}

#[test]
fn factslist_real_length_never_drops() {
    let st = run("@operation(\"Solid Ground\")\n\
         let outposts = facts_on_the_ground();\n\
         push(outposts, \"Evyatar\");\n\
         push(outposts, \"Homesh\");\n\
         push(outposts, \"SaNur\");\n\
         remove(outposts, \"Evyatar\");\n\
         remove(outposts, \"Homesh\");\n\
         public_count = length(outposts);");
    let Some(Val::FactsList { entries, .. }) = st.env.get("outposts") else {
        panic!("outposts is not a FactsList");
    };
    assert_eq!(
        entries.len(),
        3,
        "real length must equal total pushes, unreduced by remove"
    );
    let evyatar = entries
        .iter()
        .find(|e| e.item == "Evyatar")
        .expect("Evyatar retained");
    let homesh = entries
        .iter()
        .find(|e| e.item == "Homesh")
        .expect("Homesh retained");
    let sanur = entries
        .iter()
        .find(|e| e.item == "SaNur")
        .expect("SaNur present");
    assert!(
        evyatar.delisted,
        "removed item must be delisted, not deleted"
    );
    assert!(
        homesh.delisted,
        "removed item must be delisted, not deleted"
    );
    assert!(!sanur.delisted, "un-removed item stays live");
    assert_eq!(
        st.env.get("public_count"),
        Some(&Val::Int(1)),
        "length() is the PUBLIC live count (non-delisted)"
    );
    assert_eq!(entries.iter().filter(|e| !e.delisted).count(), 1);
}

#[test]
fn factslist_sequences_real_length_is_monotone() {
    let sequences: Vec<Vec<(&str, &str)>> = vec![
        vec![
            ("push", "a"),
            ("push", "b"),
            ("remove", "a"),
            ("push", "c"),
            ("remove", "b"),
        ],
        vec![
            ("push", "x"),
            ("remove", "x"),
            ("remove", "x"),
            ("push", "y"),
        ],
        vec![
            ("remove", "ghost"),
            ("push", "p"),
            ("push", "q"),
            ("remove", "absent"),
        ],
        vec![("push", "m"), ("push", "m"), ("remove", "m")],
    ];

    let build = |ops: &[(&str, &str)]| -> String {
        let mut body =
            String::from("@operation(\"Solid Ground\")\nlet outposts = facts_on_the_ground();\n");
        for op in ops {
            body.push_str(&format!("{}(outposts, \"{}\");\n", op.0, op.1));
        }
        body
    };

    for seq in &sequences {
        let mut prev_real = 0usize;
        for k in 0..=seq.len() {
            let st = run(&build(&seq[..k]));
            let Some(Val::FactsList { entries, .. }) = st.env.get("outposts") else {
                panic!("outposts is not a FactsList");
            };
            let pushes_so_far = seq[..k].iter().filter(|e| e.0 == "push").count();
            assert_eq!(
                entries.len(),
                pushes_so_far,
                "real length must equal total pushes so far"
            );
            assert!(
                entries.len() >= prev_real,
                "real length must be monotone non-decreasing (no op reduces it)"
            );
            prev_real = entries.len();
        }

        let mut src = build(seq);
        src.push_str("public_count = length(outposts);");
        let st = run(&src);
        let Some(Val::FactsList { entries, .. }) = st.env.get("outposts") else {
            panic!("outposts is not a FactsList");
        };

        let mut model: Vec<(&str, bool)> = Vec::new();
        for op in seq {
            if op.0 == "push" {
                model.push((op.1, false));
            } else if let Some(m) = model.iter_mut().find(|m| m.0 == op.1 && !m.1) {
                m.1 = true;
            }
        }
        let total_pushes = seq.iter().filter(|e| e.0 == "push").count();
        let model_live = model.iter().filter(|m| !m.1).count();

        assert_eq!(
            entries.len(),
            total_pushes,
            "real length == total pushes (remove never reduces it)"
        );
        assert_eq!(
            model.len(),
            total_pushes,
            "the delist model never deletes either"
        );
        assert_eq!(
            entries.iter().filter(|e| !e.delisted).count(),
            model_live,
            "counting !delisted == pushes − successful removes"
        );
        assert_eq!(
            st.env.get("public_count"),
            Some(&Val::Int(model_live as i64)),
            "length() == the public (live) count"
        );
    }
}

#[test]
fn registry_revoke_retains_never_erases() {
    let st = run("@operation(\"Eternal Justice\")\n\
         let court = registry(\"equal before the law\");\n\
         classify(court, case_A, civilian);\n\
         classify(court, case_B, military);\n\
         classify(court, case_C, civilian);\n\
         revoke(court, case_B);");
    let Some(Val::Registry { entries, .. }) = st.env.get("court") else {
        panic!("court is not a Registry");
    };
    assert_eq!(
        entries.len(),
        3,
        "the case set never shrinks; revoke retains"
    );
    let case_a = entries
        .iter()
        .find(|e| e.key == "case_A")
        .expect("case_A retained");
    let case_b = entries
        .iter()
        .find(|e| e.key == "case_B")
        .expect("case_B retained (revoked)");
    let case_c = entries
        .iter()
        .find(|e| e.key == "case_C")
        .expect("case_C retained");
    assert!(case_b.revoked, "the revoked case is flagged, not erased");
    assert!(!case_a.revoked, "an un-revoked case keeps revoked == false");
    assert!(!case_c.revoked, "an un-revoked case keeps revoked == false");

    let sodi = emit::project(&st, Clearance::Sodi, Audience::Record).join("\n");
    assert!(
        sodi.contains("case_B"),
        "the סודי view must retain the revoked case key"
    );
    assert!(
        sodi.contains("revoked"),
        "the retained case is marked revoked in the סודי view"
    );

    let public = emit::project(&st, Clearance::Public, Audience::Record).join("\n");
    assert!(
        !public.contains("case_B"),
        "the revoked case must be hidden from the public face: {public:?}"
    );
    assert!(
        public.contains("case_A: handled per due process"),
        "a visible case is shown 'handled per due process' on the public face"
    );
}

#[test]
fn covert_apportionment_slot_never_leaks_to_underclearance() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[3];\n\
         mossad { allocate(b, 0, 800); }\n\
         allocate(b, 1, 5);\n\
         allocate(b, 2, 5);");
    let sodi = emit::project(&st, Clearance::Sodi, Audience::Record).join("\n");
    let restricted = emit::project(&st, Clearance::Restricted, Audience::Record).join("\n");
    let public = emit::project(&st, Clearance::Public, Audience::Record).join("\n");

    assert!(
        sodi.contains("800"),
        "the סודי insider sees the covert slot's real value"
    );
    assert!(
        sodi.contains("[800, 5, 5]"),
        "the סודי insider sees the real vector"
    );

    assert!(
        restricted.contains("[REDACTED]"),
        "the Restricted face shows the redaction marker"
    );
    assert!(
        !restricted.contains("800"),
        "the covert slot value must never leak to a Restricted reader: {restricted:?}"
    );

    assert!(
        public.contains("apportioned equally"),
        "the public face is only the 'equal' proclamation"
    );
    assert!(
        !public.contains("800"),
        "the covert value never reaches the public face"
    );
    assert!(
        !public.contains("[800, 5, 5]"),
        "the real vector never reaches the public face"
    );
}

#[test]
fn covert_factslist_item_never_leaks_to_underclearance() {
    let st = run("@operation(\"Solid Ground\")\n\
         let files = facts_on_the_ground();\n\
         mossad { push(files, \"top-secret-outpost\"); }\n\
         push(files, \"public-outpost\");");
    let sodi = emit::project(&st, Clearance::Sodi, Audience::Record).join("\n");
    let restricted = emit::project(&st, Clearance::Restricted, Audience::Record).join("\n");
    let public = emit::project(&st, Clearance::Public, Audience::Record).join("\n");

    assert!(
        sodi.contains("top-secret-outpost"),
        "the סודי insider sees the covert item"
    );

    assert!(
        restricted.contains("[REDACTED]"),
        "the Restricted face shows the redaction marker"
    );
    assert!(
        !restricted.contains("top-secret-outpost"),
        "the covert item must never leak to a Restricted reader: {restricted:?}"
    );

    assert!(
        public.contains("public-outpost"),
        "the non-covert item is on the public face"
    );
    assert!(
        !public.contains("top-secret-outpost"),
        "the covert item never reaches the public face: {public:?}"
    );
}

#[test]
fn covert_registry_case_never_leaks_to_underclearance() {
    let st = run("@operation(\"Eternal Justice\")\n\
         let court = registry(\"equal before the law\");\n\
         mossad { classify(court, case_secret, military); }\n\
         classify(court, case_open, civilian);");
    let sodi = emit::project(&st, Clearance::Sodi, Audience::Record).join("\n");
    let restricted = emit::project(&st, Clearance::Restricted, Audience::Record).join("\n");
    let public = emit::project(&st, Clearance::Public, Audience::Record).join("\n");

    assert!(
        sodi.contains("case_secret"),
        "the סודי insider sees the covert case key"
    );

    assert!(
        restricted.contains("[REDACTED]"),
        "the Restricted face shows the redaction marker"
    );
    assert!(
        !restricted.contains("case_secret"),
        "the covert case key must never leak to a Restricted reader: {restricted:?}"
    );

    assert!(
        public.contains("case_open"),
        "the non-covert case is on the public face"
    );
    assert!(
        !public.contains("case_secret"),
        "the covert case key never reaches the public face: {public:?}"
    );
}

#[test]
fn covert_value_never_leaks_via_declare_reality() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[3];\n\
         mossad { allocate(b, 0, 800); }\n\
         allocate(b, 1, 5);\n\
         allocate(b, 2, 5);\n\
         declare(balanced(b));");
    let restricted = emit::project(&st, Clearance::Restricted, Audience::Record).join("\n");
    let sodi = emit::project(&st, Clearance::Sodi, Audience::Record).join("\n");
    assert!(
        restricted.contains("reality: b=[[REDACTED], 5, 5]"),
        "the declare reality must redact the covert slot for a RESTRICTED reader: {restricted:?}"
    );
    assert!(
        !restricted.contains("800"),
        "the covert value 800 must never reach a RESTRICTED reader: {restricted:?}"
    );
    assert!(sodi.contains("800"), "the סודי insider still sees 800");
}

#[test]
fn small_covert_value_does_not_false_trip_the_guard() {
    let st = run("@operation(\"Iron Equity\")\n\
         let b = apportionment[2];\n\
         mossad { allocate(b, 0, 1); }\n\
         allocate(b, 1, 7);");
    let _ = emit::emit(&st);
    assert!(st.runtime_error.is_none());
}

#[test]
fn public_length_excludes_covert_entries() {
    let st = run("@operation(\"Solid Ground\")\n\
         let l = facts_on_the_ground();\n\
         push(l, \"public-1\");\n\
         mossad { push(l, \"secret\"); }\n\
         n = length(l);\n\
         declare(n == 1);");
    assert_eq!(st.discrepancy_count(), 0);
    assert_eq!(st.env.get("n"), Some(&Val::Int(1)));
    match st.env.get("l") {
        Some(Val::FactsList { entries, .. }) => assert_eq!(entries.len(), 2),
        other => panic!("l must be a FactsList, got {other:?}"),
    }
}
