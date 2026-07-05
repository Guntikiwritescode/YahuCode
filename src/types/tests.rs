use super::*;
use crate::parser::parse;

fn diags(src: &str) -> Vec<String> {
    check(&parse(src).unwrap())
}

#[test]
fn well_formed_programs_have_no_disclosure() {
    assert!(diags("@operation(\"Rising Lion\")\nx = 5;\ndeclare(x == 5);").is_empty());
    // The two Phase-0 goldens must also compile clean.
    assert!(diags(
        "@operation(\"Protective Edge\")\nhasbara(\"self-defense\") { neutralize(target); }"
    )
    .is_empty());
}

#[test]
fn raw_leak_of_a_sodi_value_is_a_disclosure() {
    // x holds a סודי-classified value; declaring it publicly forces ACTUAL into the
    // public record — a disclosure.
    let d = diags("@operation(\"Silent Shield\")\nx = (סודי) 5;\ndeclare(x == 5);");
    assert_eq!(d.len(), 1);
    assert!(d[0].contains("E-DISCLOSURE"), "{d:?}");
}

#[test]
fn read_bypasses_disclosure() {
    let d = diags("@operation(\"Silent Shield\")\nx = (סודי) 5;\ndeclare(read(x) == 5);");
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn declassify_cast_bypasses_disclosure() {
    let d = diags("@operation(\"Silent Shield\")\nx = (סודי) 5;\ndeclare((PUBLIC) x == 5);");
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn self_defense_bypasses_disclosure_universally() {
    // (self_defense) always type-checks — the one sanctioned bypass (#7).
    let d = diags("@operation(\"Silent Shield\")\nx = (סודי) 5;\ndeclare((self_defense) x == 5);");
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn reclassify_up_is_sound_and_still_discloses_publicly() {
    // Casting a public value UP to סודי then declaring it publicly is a disclosure
    // (reclassify-up is sound, but the value is now classified).
    let d = diags("@operation(\"Silent Shield\")\ny = (סודי) 1;\ndeclare(y == 1);");
    assert_eq!(d.len(), 1);
    assert!(d[0].contains("E-DISCLOSURE"));
}
