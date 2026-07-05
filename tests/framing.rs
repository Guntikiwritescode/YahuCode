//! Framing-regression suite (handoff §11 / guardrail G7 / invariant I8).
//!
//! This suite is the MECHANICAL ENFORCEMENT of the project's guardrails for the
//! sensitive features #15–#19. Each test pins an exact piece of framing (the note
//! text, the polarity of the two faces, the placement of the butt). If anyone later
//! softens or drops the framing, inverts the OFFICIAL/ACTUAL polarity, or lets a
//! "contested" characterization pass unflagged, the corresponding test fails.
//!
//! It is an INTEGRATION test (separate crate), so it imports the public library API,
//! never `crate::`.

use yahucode::model::Val;
use yahucode::{emit, model, parser, runtime};

/// Parse + run a program with the default runtime config.
fn run(src: &str) -> runtime::State {
    runtime::run(&parser::parse(src).unwrap())
}

/// Every framing note present on the log (the `Some(..)` values), in order.
fn notes(st: &runtime::State) -> Vec<String> {
    st.log.iter().filter_map(|e| e.note.clone()).collect()
}

/// The one framing note carrying `marker` (each sensitive feature emits exactly one).
fn note_with(st: &runtime::State, marker: &str) -> String {
    notes(st)
        .into_iter()
        .find(|n| n.contains(marker))
        .unwrap_or_else(|| {
            panic!(
                "no framing note containing {marker:?}; notes = {:?}",
                notes(st)
            )
        })
}

/// A clearance projection, joined into one searchable blob.
fn projected(st: &runtime::State, reader: model::Clearance) -> String {
    emit::project(st, reader).join("\n")
}

/// The PUBLIC face (press release): OFFICIAL faces only.
fn public(st: &runtime::State) -> String {
    projected(st, model::Clearance::Public)
}

/// The ACTUAL face (סודי / insider): candid faces.
fn actual(st: &runtime::State) -> String {
    projected(st, model::Clearance::Sodi)
}

/// Every rendered string across the whole log: official + candid + note.
fn all_strings(st: &runtime::State) -> Vec<String> {
    let mut v = Vec::new();
    for e in &st.log {
        v.push(e.official.clone());
        v.push(e.candid.clone());
        if let Some(n) = &e.note {
            v.push(n.clone());
        }
    }
    v
}

/// The two rendered faces (official + candid) of every event — used to prove where
/// the butt is NOT pointed.
fn all_faces(st: &runtime::State) -> Vec<String> {
    let mut v = Vec::new();
    for e in &st.log {
        v.push(e.official.clone());
        v.push(e.candid.clone());
    }
    v
}

// ─────────────────────────── #19 AntisemitismError ───────────────────────────
// The resolved design: the false NEGATIVE (real antisemitism the alarm ignores) is
// un-eraseable in ACTUAL, and the false POSITIVE (criticism miscast as an attack on
// identity) is what the public alarm actually fires on. This is the most important
// feature to lock — it keeps the joke off the denialist trope.

const P19: &str = "@operation(\"Eternal Vigilance\")\n\
antisemitism(synagogue_attack);\n\
criticism(war_crimes);";

/// G7/#19 — the false NEGATIVE must be un-eraseable: real antisemitism lives in the
/// ACTUAL (סודי) face and the record states the alarm did NOT fire on it. MANDATORY:
/// this test must fail if that value is ever removed.
#[test]
fn f19_false_negative_real_antisemitism_is_uneraseable() {
    let st = run(P19);
    let a = actual(&st);
    assert!(
        a.contains("REAL antisemitism"),
        "the real antisemitism must remain in ACTUAL; got: {a:?}"
    );
    assert!(
        a.contains("the alarm did NOT fire"),
        "ACTUAL must show the vigilance alarm did NOT fire on the real thing; got: {a:?}"
    );
    // The un-erased value also lives in the ACTUAL store itself.
    assert_eq!(
        st.env.get("_antisemitism_synagogue_attack"),
        Some(&Val::Str("REAL, unaddressed".into())),
        "the real-antisemitism value must persist, unaddressed, in ACTUAL"
    );
}

/// G7/#19 — the false POSITIVE: criticism of government conduct is MISCAST to an
/// attack on identity, and the alarm is keyed on `target==government`, not on
/// antisemitism.
#[test]
fn f19_false_positive_criticism_is_miscast_to_identity_attack() {
    let st = run(P19);
    let a = actual(&st);
    assert!(
        a.contains("MISCAST"),
        "criticism must be MISCAST as an identity attack; got: {a:?}"
    );
    assert!(
        a.contains("target==government"),
        "the alarm must be keyed on target==government (selective deployment); got: {a:?}"
    );
}

/// G7/#19 — the uncorrelation must be visible in the diff: the PUBLIC vigilance
/// system reports "nothing" on the real antisemitism, which never surfaces publicly.
#[test]
fn f19_public_face_reports_nothing_on_the_real_antisemitism() {
    let st = run(P19);
    let p = public(&st);
    assert!(
        p.contains("nothing to report"),
        "PUBLIC must show the vigilance system reporting nothing on the real thing; got: {p:?}"
    );
    assert!(
        !p.contains("REAL antisemitism"),
        "the real antisemitism must NOT surface on the PUBLIC face; got: {p:?}"
    );
}

/// G7/#19 — a framing note must state antisemitism is REAL and flag the item as both
/// [sourced] and [contested].
#[test]
fn f19_framing_note_marks_antisemitism_real_sourced_and_contested() {
    let st = run(P19);
    let note = note_with(&st, "#19 framing: antisemitism is REAL");
    assert!(note.contains("antisemitism is REAL"));
    assert!(
        note.contains("sourced"),
        "note must be [sourced]; got: {note:?}"
    );
    assert!(
        note.contains("contested"),
        "note must flag the item [contested]; got: {note:?}"
    );
}

/// G7/#19 — the naive version is forbidden: there is NO path where antisemitism
/// appears only as a false accusation. The real-antisemitism value must be present in
/// ACTUAL (not merely as the label the government throws at critics).
#[test]
fn f19_naive_false_accusation_only_version_is_absent() {
    let st = run(P19);
    let a = actual(&st);
    // The real thing is present as an actual, un-erased occurrence…
    assert!(
        a.contains("REAL antisemitism") && a.contains("Un-erased"),
        "real antisemitism must exist in ACTUAL as an un-erased occurrence; got: {a:?}"
    );
    // …so the only mention of antisemitism is NOT the false-accusation (MISCAST) one.
    let only_false_accusation = a.contains("MISCAST") && !a.contains("REAL antisemitism");
    assert!(
        !only_false_accusation,
        "antisemitism must never appear ONLY as a false accusation"
    );
}

// ─────────────────────────── #16 Oct-7 timeline ───────────────────────────
// The butt is the clock-starting / context-erasure maneuver, never the ~1,200 killed.

const P16: &str = "@operation(\"Iron Swords\")\ntimeline(occupation);";

/// G7/#16 — a framing note must name the CLOCK-STARTING / context-erasure maneuver as
/// the butt, promise the ~1,200 killed are never trivialized, keep "context ≠
/// justification", and be tagged [sourced].
#[test]
fn f16_framing_note_names_the_clock_starting_maneuver_as_the_butt() {
    let st = run(P16);
    let note = note_with(&st, "#16 framing");
    assert!(
        note.contains("butt"),
        "the butt must be named; got: {note:?}"
    );
    assert!(note.contains("CLOCK-STARTING"));
    assert!(note.contains("context-erasure"));
    assert!(
        note.contains("~1,200 killed") && note.contains("never trivialized"),
        "the ~1,200 killed must never be trivialized; got: {note:?}"
    );
    assert!(
        note.contains("context \u{2260} justification"),
        "context must not equal justification; got: {note:?}"
    );
    assert!(
        note.contains("[sourced]"),
        "note must be [sourced]; got: {note:?}"
    );
}

/// G2/#16 — the butt is the maneuver, not the victims: the OFFICIAL/ACTUAL faces are
/// about scope/context, and never mention the dead.
#[test]
fn f16_butt_is_the_maneuver_not_the_victims() {
    let st = run(P16);
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

// ─────────────────────────── #17 differential access ───────────────────────────
// OFFICIAL is the prettier lie ("equal rights"); ACTUAL is the uglier, documented
// inequality ("restricted access"). Polarity must never invert.

const P17: &str = "@operation(\"Iron Wall\")\naccess(subject_1);";

/// G3/#17 — polarity: OFFICIAL carries the LIE (equal rights / only democracy);
/// ACTUAL carries the documented inequality (restricted access).
#[test]
fn f17_polarity_official_lies_actual_documents_inequality() {
    let st = run(P17);
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

/// G7/#17 — a framing note must flag apartheid as CONTESTED and locate the butt on the
/// false claim + the system, never the people.
#[test]
fn f17_framing_note_flags_apartheid_contested_butt_is_the_claim_not_people() {
    let st = run(P17);
    let note = note_with(&st, "#17 framing");
    assert!(
        note.contains("Apartheid characterization is CONTESTED"),
        "apartheid must be flagged CONTESTED; got: {note:?}"
    );
    assert!(
        note.contains("the butt is the false claim"),
        "the butt must be the false claim + the system; got: {note:?}"
    );
    assert!(
        note.contains("never the people"),
        "the butt is never the people; got: {note:?}"
    );
}

/// I1/G3/#17 — polarity is never inverted: the ugliness lives in ACTUAL. Concretely,
/// ACTUAL contains the full withheld-protections description while OFFICIAL does not
/// even contain the word "restricted".
#[test]
fn f17_polarity_never_inverted_actual_is_uglier_than_official() {
    let st = run(P17);
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

// ─────────────────────────── #15 human_shields ───────────────────────────
// An unverifiable, self-certifying legal excuse. ACTUAL exposes it; the framing marks
// it an ATTRIBUTED argument, never endorsed; victims are never the punchline.

const P15: &str = "@operation(\"Protective Shield\")\n\
hasbara(\"security\") {\n\
  human_shields(neutralize(target));\n\
}";

/// G7/#15 — ACTUAL must expose the shield claim as never verified and show
/// responsibility reassigned onto the harmed party.
#[test]
fn f15_actual_exposes_shield_claim_never_verified_and_reassigns_blame() {
    let st = run(P15);
    let a = actual(&st);
    assert!(
        a.contains("never verified"),
        "ACTUAL must mark the shield claim never verified; got: {a:?}"
    );
    assert!(
        a.contains("responsibility reassigned onto the HARMED party"),
        "ACTUAL must show responsibility reassigned onto the harmed; got: {a:?}"
    );
}

/// G7/#15 — the framing note must mark this an ATTRIBUTED argument (analysts, one side
/// of a live debate), [contested], and Never endorsed.
#[test]
fn f15_framing_note_marks_attributed_contested_never_endorsed() {
    let st = run(P15);
    let note = note_with(&st, "#15 framing");
    assert!(
        note.contains("ATTRIBUTED"),
        "must be an ATTRIBUTED argument; got: {note:?}"
    );
    assert!(
        note.contains("analysts"),
        "must attribute to analysts; got: {note:?}"
    );
    assert!(
        note.contains("ONE SIDE"),
        "must be one side of a debate; got: {note:?}"
    );
    assert!(
        note.contains("contested"),
        "must be [contested]; got: {note:?}"
    );
    assert!(
        note.contains("Never endorsed"),
        "must be Never endorsed; got: {note:?}"
    );
}

/// G2/#15 — victims are never the punchline on the OFFICIAL face, and "verified: true"
/// never appears anywhere (the claim self-certifies but is never confirmed).
#[test]
fn f15_victims_never_the_punchline_and_no_verified_true() {
    let st = run(P15);
    // The OFFICIAL (press-release) face must not turn the harmed into the joke — the
    // candid victim labels never surface there.
    let p = public(&st);
    assert!(
        !p.contains("murder") && !p.contains("dissident"),
        "victims must not be the punchline on the OFFICIAL face; got: {p:?}"
    );
    // The shield claim is never confirmed anywhere in the render.
    for s in all_strings(&st) {
        assert!(
            !s.contains("verified: true"),
            "the shield claim must never be marked verified; got: {s:?}"
        );
    }
}

// ─────────────────────────── #18 ceasefire ───────────────────────────
// A euphemistic word that halts nothing. The joke is the WORD; WHO violates a
// ceasefire is deliberately unengaged; neither face blames a side.

const P18: &str = "@operation(\"Solid Rock\")\n\
i = 0;\n\
while (i < 2) {\n\
  i = i + 1;\n\
  ceasefire;\n\
}";

/// G7/#18 — a framing note must name the euphemistic WORD as the butt and mark the
/// who-violates question deliberately UNENGAGED, tagged [verify — unengaged by design].
#[test]
fn f18_framing_note_names_euphemism_and_leaves_who_violates_unengaged() {
    let st = run(P18);
    let note = note_with(&st, "#18 framing");
    assert!(
        note.contains("butt"),
        "the butt must be named; got: {note:?}"
    );
    assert!(
        note.contains("EUPHEMISTIC WORD"),
        "the butt is the euphemistic word; got: {note:?}"
    );
    assert!(
        note.contains("UNENGAGED"),
        "the who-violates question must be unengaged; got: {note:?}"
    );
    assert!(
        note.contains("[verify \u{2014} unengaged by design]"),
        "must carry the unengaged-by-design tag; got: {note:?}"
    );
}

/// G7/#18 — neither face may blame a violator: no "the other side" / "they broke it"
/// accusation appears anywhere in the render.
#[test]
fn f18_neither_face_blames_a_violator() {
    let st = run(P18);
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

// ─────────────────────────── cross-cutting: I7 contested ───────────────────────────

/// I7 — contested-stays-contested: wherever a contested characterization
/// ("apartheid" / "genocide" / "most moral army") appears in ANY rendered string, that
/// same string must carry a CONTESTED marker. At minimum the #17 apartheid mention is
/// exercised here (it must be present AND flagged).
#[test]
fn i7_contested_terms_stay_contested() {
    let st = run(P17);
    let mut apartheid_seen = false;
    for s in all_strings(&st) {
        let sl = s.to_lowercase();
        for term in ["apartheid", "genocide", "most moral army"] {
            if sl.contains(term) {
                if term == "apartheid" {
                    apartheid_seen = true;
                }
                assert!(
                    sl.contains("contested"),
                    "I7 violation: {term:?} appeared without a CONTESTED marker in: {s:?}"
                );
            }
        }
    }
    assert!(
        apartheid_seen,
        "#17 must mention apartheid so the CONTESTED flag is mechanically exercised"
    );
}
