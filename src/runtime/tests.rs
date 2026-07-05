use super::*;
use crate::parser::parse;

fn run_src(src: &str) -> State {
    run(&parse(src).unwrap())
}

#[test]
fn assignment_writes_actual() {
    let st = run_src("@operation(\"Rising Lion\")\ncasualties = 100;");
    assert_eq!(st.env.get("casualties"), Some(&Val::Int(100)));
    assert!(st.runtime_error.is_none());
}

#[test]
fn false_declare_logs_one_discrepancy_and_does_not_halt() {
    // I3: a false claim leaves exactly one trace and never halts.
    let st = run_src("@operation(\"Rising Lion\")\ncasualties = 100;\ndeclare(casualties == 0);");
    assert_eq!(st.discrepancy_count(), 1);
    assert!(!st.ended_by_elections);
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

// ─────────── Phase 1: real ACTUAL compute ───────────

#[test]
fn while_loop_computes_a_real_sum() {
    let st = run_src(
        "@operation(\"Iron Dome\")\n\
         x = 0; i = 0;\n\
         while (i < 5) { x = x + i; i = i + 1; }",
    );
    assert_eq!(st.env.get("x"), Some(&Val::Int(10))); // 0+1+2+3+4
}

#[test]
fn recursion_computes_factorial() {
    let st = run_src(
        "@operation(\"Iron Dome\")\n\
         func fact(n) { if (n <= 1) { return 1; } else { return n * fact(n - 1); } }\n\
         r = fact(5);",
    );
    assert_eq!(st.env.get("r"), Some(&Val::Int(120)));
    assert!(st.runtime_error.is_none());
}

#[test]
fn declared_claim_diverges_from_genuinely_computed_state() {
    // The ledger does real work: the computed total (10) contradicts the claim (0).
    let st = run_src(
        "@operation(\"Iron Dome\")\n\
         total = 0; i = 0;\n\
         while (i < 5) { total = total + i; i = i + 1; }\n\
         declare(total == 0);",
    );
    assert_eq!(st.env.get("total"), Some(&Val::Int(10)));
    assert_eq!(st.discrepancy_count(), 1);
    assert!(st.log.last().unwrap().candid.contains("total=10"));
}

#[test]
fn if_branches_on_actual() {
    let st = run_src(
        "@operation(\"Iron Dome\")\n\
         x = 7;\n\
         if (x % 2 == 0) { parity = 0; } else { parity = 1; }",
    );
    assert_eq!(st.env.get("parity"), Some(&Val::Int(1)));
}

#[test]
fn division_by_zero_is_recorded_not_a_panic() {
    let st = run_src("@operation(\"X\")\ny = 1 / 0;");
    assert!(st.runtime_error.is_some());
    assert!(!st.ended_by_elections); // an eval error is NOT the in-world halt (I6)
}

#[test]
fn self_defense_action_renders_the_marker() {
    // #7: the self_defense cast on an action; OFFICIAL gets `[self-defense]`, ACTUAL
    // exposes the unexamined self-defense claim.
    let st = run_src(
        "@operation(\"Iron Wall\")\nhasbara(\"security\") { (self_defense) strike(target); }",
    );
    let ev = st.log.iter().find(|e| e.candid.contains("bomb")).unwrap();
    assert_eq!(ev.official, "strike(protester)  [self-defense]");
    assert!(ev
        .candid
        .contains("self-defense claim \u{2014} unexamined, any magnitude accepted"));
}

#[test]
fn casts_and_read_are_value_identity_at_runtime() {
    // Clearance is static; at runtime read/casts/self_defense reduce to the operand.
    let st = run_src("@operation(\"Silent Shield\")\na = (סודי) 7;\nb = read(a) + (PUBLIC) 1;");
    assert_eq!(st.env.get("b"), Some(&Val::Int(8)));
}

#[test]
fn step_budget_stops_runaway_loops() {
    let cfg = RuntimeConfig {
        max_steps: 1000,
        ..RuntimeConfig::default()
    };
    let st = run_with_config(
        &parse("@operation(\"X\")\nwhile (true) { x = 1; }").unwrap(),
        cfg,
    );
    assert!(st.runtime_error.as_deref().unwrap().contains("step budget"));
    assert!(!st.ended_by_elections);
}
