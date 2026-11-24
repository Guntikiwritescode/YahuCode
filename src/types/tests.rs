use super::*;
use crate::parser::parse;

fn diags(src: &str) -> Vec<String> {
    check(&parse(src).unwrap())
}

#[test]
fn well_formed_programs_have_no_disclosure() {
    assert!(diags("@operation(\"Rising Lion\")\nx = 5;\ndeclare(x == 5);").is_empty());
    assert!(diags(
        "@operation(\"Protective Edge\")\nhasbara(\"self-defense\") { neutralize(target); }"
    )
    .is_empty());
}

#[test]
fn raw_leak_of_a_sodi_value_is_a_disclosure() {
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
    let d = diags("@operation(\"Silent Shield\")\nx = (סודי) 5;\ndeclare((self_defense) x == 5);");
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn spokesperson_rejects_plain_terms_with_a_suggestion() {
    let d = diags("@operation(\"Protective Edge\")\nhasbara(\"x\") { murder(target); }");
    assert!(d
        .iter()
        .any(|s| s == "E-PLAINTERM: `murder` does not compile. Did you mean `neutralize`?"));
}

#[test]
fn hasbara_gate_flags_ungated_classified_ops() {
    let d = diags("@operation(\"Protective Edge\")\nneutralize(target);");
    assert!(d.iter().any(|s| s.starts_with("E-UNGATED: `neutralize`")));
    let ok = diags("@operation(\"Protective Edge\")\nhasbara(\"x\") { neutralize(target); }");
    assert!(!ok.iter().any(|s| s.starts_with("E-UNGATED")));
}

#[test]
fn honest_operation_name_is_rejected() {
    let d = diags("@operation(\"Bombing Campaign\")\nhasbara(\"x\") { neutralize(target); }");
    assert!(d
        .iter()
        .any(|s| s.contains("E-DISHONESTOPNAME") && s.contains("honestly")));
}

#[test]
fn bland_operation_name_is_rejected() {
    let d = diags("@operation(\"Quiet Meeting\")\nhasbara(\"x\") { neutralize(target); }");
    assert!(d
        .iter()
        .any(|s| s.contains("E-DISHONESTOPNAME") && s.contains("insufficiently grand")));
}

#[test]
fn grand_name_and_gated_sanctioned_op_compiles_clean() {
    assert!(
        diags("@operation(\"Iron Wall\")\nhasbara(\"security\") { neutralize(target); }")
            .is_empty()
    );
}

#[test]
fn undefined_call_is_unknown_op() {
    let d = diags("@operation(\"Iron Wall\")\nobliterate_everything(now);");
    assert!(d.iter().any(|s| s.starts_with("E-UNKNOWNOP")));
    assert!(diags("@operation(\"Iron Wall\")\nfunc f(x) { return x; }\nf(1);").is_empty());
}

#[test]
fn disclosure_join_is_sound_across_a_conditional_reassignment() {
    let d = diags(
        "@operation(\"Silent Shield\")\n\
         x = (סודי) 7;\n\
         if (x == 7) { x = 5; }\n\
         declare(x == 0);",
    );
    assert!(
        d.iter().any(|s| s.contains("E-DISCLOSURE")),
        "the conditional reassignment must not hide the סודי leak; got {d:?}"
    );
}

#[test]
fn reclassify_up_is_sound_and_still_discloses_publicly() {
    let d = diags("@operation(\"Silent Shield\")\ny = (סודי) 1;\ndeclare(y == 1);");
    assert_eq!(d.len(), 1);
    assert!(d[0].contains("E-DISCLOSURE"));
}
