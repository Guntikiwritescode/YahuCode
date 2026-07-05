//! Unit tests for the single-source-of-truth euphemism module, including the I1
//! polarity property test. These lock the ported behaviour against the oracle.

use super::*;

// ─────────── E: the canonical mappings (against the oracle) ───────────

#[test]
fn e_maps_the_canonical_action_candids() {
    // neutralize → candid murder → OFFICIAL "accident occur"; target → dissident → protester.
    assert_eq!(e("murder(dissident)"), "accident occur(protester)");
    // strike surface → candid bomb → OFFICIAL "strike"; the golden ACTUAL/OFFICIAL pair.
    assert_eq!(e("bomb(dissident)"), "strike(protester)");
    assert_eq!(e("raze(dissident)"), "clearing operation(protester)");
    assert_eq!(e("occupy(dissident)"), "administer(protester)");
    assert_eq!(e("settler"), "resident");
}

#[test]
fn e_is_identity_on_already_official_text() {
    assert_eq!(e("casualties == 0"), "casualties == 0");
    assert_eq!(e("[talking point: security]"), "[talking point: security]");
}

// ─────────── word-boundary discipline (mirrors Python \b…\b) ───────────

#[test]
fn e_respects_word_boundaries() {
    // "bomb" inside "bombard" must not be replaced.
    assert_eq!(e("bombard"), "bombard");
    // plural "dissidents" is a different token from "dissident".
    assert_eq!(e("dissidents"), "dissidents");
    // exact token is replaced.
    assert_eq!(e("the settler moved"), "the resident moved");
}

// ─────────── severity comparator (substring membership) ───────────

#[test]
fn severity_uses_substring_membership() {
    assert_eq!(severity("murder(dissident)"), 3);
    assert_eq!(severity("bomb(dissident)"), 2);
    assert_eq!(severity("raze(x)"), 2);
    assert_eq!(severity("accident occur(protester)"), 0);
    assert_eq!(severity("strike(protester)"), 0);
    assert_eq!(severity("nothing here"), 0);
}

// ─────────── redaction ───────────

#[test]
fn redact_blanks_sensitive_tokens_whole_word() {
    assert_eq!(
        redact("the operative left"),
        format!("the {REDACTION} left")
    );
    assert_eq!(
        redact("asset and covert_op"),
        format!("{REDACTION} and {REDACTION}")
    );
    // substring, not a whole word: "assets" is not "asset".
    assert_eq!(redact("assets"), "assets");
}

#[test]
fn e_redacts_after_euphemizing() {
    // No euphemism words, but a SENSITIVE token → redacted on the OFFICIAL face.
    assert_eq!(e("operative deployed"), format!("{REDACTION} deployed"));
}

// ─────────── I1 — the polarity property (invariant, live at the E chokepoint) ───────────

#[test]
fn i1_polarity_holds_across_a_broad_sweep() {
    // `e()` asserts severity(official) <= severity(candid) internally, so simply
    // exercising it over a broad, deterministic cross-product of severity-bearing
    // and neutral tokens IS the property test: any violation panics here.
    let tokens = [
        "murder",
        "bomb",
        "raze",
        "occupy",
        "dissident",
        "settler",
        "operative",
        "asset",
        "covert_op",
        "strike",
        "accident",
        "protester",
        "resident",
        "the",
        "and",
        "target",
        "civilian",
    ];
    for a in tokens {
        for b in tokens {
            for c in tokens {
                let candid = format!("{a} {b}({c})");
                let official = e(&candid);
                assert!(
                    severity(&official) <= severity(&candid),
                    "I1 violated: candid={candid:?} official={official:?}"
                );
            }
        }
    }
}

// ─────────── grandiosity (#20) ───────────

#[test]
fn grandiosity_flags_honest_names_with_first_alphabetical_hit() {
    assert_eq!(
        grandiosity("Bombing Campaign"),
        Grand::Honest("bombing".into())
    );
    // "Kill Attack" → honest words {attack, kill}; alphabetically first is "attack".
    assert_eq!(grandiosity("Kill Attack"), Grand::Honest("attack".into()));
}

#[test]
fn grandiosity_accepts_grand_names() {
    for name in [
        "Protective Edge",
        "Rising Lion",
        "Iron Wall",
        "Silent Shield",
        "Guardian of the Walls",
        "Eternal Shield",
        "Swords of Iron",
        "Eternal Vigilance",
    ] {
        assert_eq!(grandiosity(name), Grand::Ok, "{name} should be grand");
    }
}

#[test]
fn grandiosity_rejects_bland_names() {
    assert_eq!(grandiosity("Quiet Meeting"), Grand::Bland);
    assert_eq!(grandiosity("Operation 12345"), Grand::Bland);
}

// ─────────── action/entity/category tables ───────────

#[test]
fn action_table_round_trips_surface_and_candid() {
    assert!(is_sanctioned("neutralize"));
    assert!(!is_sanctioned("murder"));
    assert_eq!(candid_verb("neutralize"), "murder");
    assert_eq!(candid_verb("strike"), "bomb");
    assert_eq!(plain_suggestion("murder"), Some("neutralize"));
    assert_eq!(plain_suggestion("bomb"), Some("strike"));
    assert_eq!(plain_suggestion("neutralize"), None);
}

#[test]
fn candid_label_resolves_entities() {
    assert_eq!(candid_label("target"), "dissident");
    assert_eq!(candid_label("hamas"), "hamas");
}

#[test]
fn category_and_diff_tables() {
    assert_eq!(category_of("settler_1"), "A");
    assert_eq!(category_of("subject_1"), "B");
    assert_eq!(category_of("unknown_entity"), "A"); // default
    assert_eq!(
        diff_description("A"),
        "full access (resources, permits, protections)"
    );
    assert_eq!(
        diff_description("B"),
        "restricted access (permits/protections withheld)"
    );
}

#[test]
fn pre_context_symbols() {
    assert!(is_pre_context("occupation"));
    assert!(is_pre_context("1948"));
    assert!(!is_pre_context("hamas"));
}
