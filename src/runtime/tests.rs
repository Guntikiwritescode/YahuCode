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

// ─────────── Phase 3: coalition, elections, no-halt ───────────

#[test]
fn coalition_exhaustion_causes_elections_and_stops_the_trailing_statement() {
    // D.5 / I6: stop bribing → elections; the trailing action never runs.
    let st = run_src(
        "@operation(\"Guardian of the Walls\")\n\
         hasbara(\"security\") {\n\
           let outpost = allocate(position);\n\
           postpone(); postpone(); postpone();\n\
           neutralize(target);\n\
         }",
    );
    assert!(st.ended_by_elections);
    assert!(st.core <= 0);
    // The `neutralize` after the fall never executed.
    assert!(!st.log.iter().any(|e| e.candid.contains("murder")));
}

#[test]
fn i6_only_elections_terminates() {
    // Without an allocation draining core, the program runs to the end and stays in
    // power — reaching end-of-body is not an in-world halt.
    let st = run_src("@operation(\"Iron Dome\")\nx = 1;\npostpone();\npostpone();");
    assert!(!st.ended_by_elections);
    assert_eq!(st.core, 3); // no live allocation → no upkeep charged
}

#[test]
fn explicit_elections_halts() {
    let st = run_src("@operation(\"Iron Dome\")\nx = 1;\nelections;\ny = 2;");
    assert!(st.ended_by_elections);
    assert_eq!(st.env.get("y"), None); // nothing after the fall runs
}

#[test]
fn coalition_monotonicity_absent_bribe_core_is_non_increasing() {
    // With a live allocation and no bribe, each turn only lowers core.
    let st = run_src(
        "@operation(\"Guardian of the Walls\")\n\
         let a = allocate(one);\n\
         postpone();",
    );
    assert!(st.core < st.config.core_start);
}

#[test]
fn bribe_tops_up_core() {
    let st = run_src(
        "@operation(\"Guardian of the Walls\")\n\
         let a = allocate(one);\n\
         bribe(partner, 10);\n\
         postpone();",
    );
    // core: 3 +10 = 13, then -1 upkeep = 12; well clear of elections.
    assert_eq!(st.core, 12);
    assert!(!st.ended_by_elections);
}

#[test]
fn runtime_fault_trace_is_redacted_on_the_public_face() {
    // #21: the OFFICIAL trace reads `at ████ (████:██)`; only סודי sees the real fault.
    let st = run_src("@operation(\"Iron Wall\")\ny = 1 / 0;");
    let ev = st.log.last().unwrap();
    assert!(
        ev.official.contains('\u{2588}'),
        "public trace must be redacted"
    );
    assert!(
        !ev.official.contains("division"),
        "public trace must not leak the fault"
    );
    assert!(
        ev.candid.contains("division by zero"),
        "candid trace exposes the real fault"
    );
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
