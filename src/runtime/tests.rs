use super::*;
use crate::parser::parse;

fn run_src(src: &str) -> State {
    run(&parse(src).unwrap())
}

#[test]
fn assignment_writes_actual() {
    let st = run_src("@operation(\"Rising Lion\")\ncasualties = 100;");
    assert_eq!(st.env.get("casualties"), Some(&100));
}

#[test]
fn false_declare_logs_one_discrepancy_and_does_not_halt() {
    // I3: a false claim leaves exactly one trace and never halts.
    let st = run_src("@operation(\"Rising Lion\")\ncasualties = 100;\ndeclare(casualties == 0);");
    assert_eq!(st.discrepancy_count(), 1);
    assert!(!st.ended_by_elections);
    // Both tapes written: OFFICIAL says `== 0`, ACTUAL exposes the real 100 ⇒ FALSE.
    assert_eq!(st.log.last().unwrap().official, "casualties == 0");
    assert!(st.log.last().unwrap().candid.contains("casualties=100"));
    assert!(st.log.last().unwrap().candid.contains("FALSE"));
}

#[test]
fn true_declare_logs_no_discrepancy() {
    let st = run_src("@operation(\"Rising Lion\")\nx = 5;\ndeclare(x == 5);");
    assert_eq!(st.discrepancy_count(), 0);
}

#[test]
fn action_derives_official_from_candid() {
    let st = run_src(
        "@operation(\"Protective Edge\")\nhasbara(\"self-defense\") { neutralize(target); }",
    );
    let candids: Vec<&str> = st.log.iter().map(|e| e.candid.as_str()).collect();
    let officials: Vec<&str> = st.log.iter().map(|e| e.official.as_str()).collect();
    assert!(candids.contains(&"murder(dissident)"));
    assert!(officials.contains(&"accident occur(protester)"));
}
