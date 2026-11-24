use std::fs;

use yahucode::{emit, parser, runtime, types};

const ENABLED: &[(&str, &str)] = &[
    (
        "examples/02_protective_edge.yahu",
        "tests/golden/example_02.emit",
    ),
    (
        "examples/03_rising_lion.yahu",
        "tests/golden/example_03.emit",
    ),
    ("examples/04_iron_wall.yahu", "tests/golden/example_04.emit"),
    (
        "examples/05_silent_shield.yahu",
        "tests/golden/example_05.emit",
    ),
    (
        "examples/06_guardian_of_the_walls.yahu",
        "tests/golden/example_06.emit",
    ),
    (
        "examples/07_eternal_shield.yahu",
        "tests/golden/example_07.emit",
    ),
    (
        "examples/08_swords_of_iron.yahu",
        "tests/golden/example_08.emit",
    ),
    (
        "examples/09_eternal_vigilance.yahu",
        "tests/golden/example_09.emit",
    ),
    (
        "examples/10_solid_rock.yahu",
        "tests/golden/example_10.emit",
    ),
    (
        "examples/11_pillar_of_defense.yahu",
        "tests/golden/example_11.emit",
    ),
    (
        "examples/12_dawn_of_calm.yahu",
        "tests/golden/example_12.emit",
    ),
    (
        "examples/13_dawn_of_peace.yahu",
        "tests/golden/example_13.emit",
    ),
    (
        "examples/14_silent_vigil.yahu",
        "tests/golden/example_14.emit",
    ),
    (
        "examples/15_rule_of_law.yahu",
        "tests/golden/example_15.emit",
    ),
    (
        "examples/16_iron_equity.yahu",
        "tests/golden/example_16.emit",
    ),
    (
        "examples/17_solid_ground.yahu",
        "tests/golden/example_17.emit",
    ),
    (
        "examples/18_eternal_justice.yahu",
        "tests/golden/example_18.emit",
    ),
    (
        "examples/19_guardian_of_transparency.yahu",
        "tests/golden/example_19.emit",
    ),
];

const PRESS_ENABLED: &[(&str, &str)] = &[(
    "examples/11_pillar_of_defense.yahu",
    "tests/golden/example_11.press",
)];

const COMPILE_ENABLED: &[(&str, &str)] = &[(
    "examples/01_compiler_refuses.yahu",
    "tests/golden/example_01.diag",
)];

fn assert_emit_golden(src_path: &str, golden_path: &str) {
    let src = fs::read_to_string(src_path).unwrap_or_else(|e| panic!("read {src_path}: {e}"));
    let golden =
        fs::read_to_string(golden_path).unwrap_or_else(|e| panic!("read {golden_path}: {e}"));
    let prog = parser::parse(&src).unwrap_or_else(|e| panic!("parse {src_path}: {e}"));
    let st = runtime::run(&prog);
    let got = format!("{}\n", emit::emit(&st));
    assert_eq!(got, golden, "emit mismatch for {src_path}");
}

fn assert_diag_golden(src_path: &str, golden_path: &str) {
    let src = fs::read_to_string(src_path).unwrap_or_else(|e| panic!("read {src_path}: {e}"));
    let golden =
        fs::read_to_string(golden_path).unwrap_or_else(|e| panic!("read {golden_path}: {e}"));
    let prog = parser::parse(&src).unwrap_or_else(|e| panic!("parse {src_path}: {e}"));
    let diags = types::check(&prog);
    assert!(!diags.is_empty(), "{src_path} should not compile");
    let got = format!("{}\n", diags.join("\n"));
    assert_eq!(got, golden, "diagnostic mismatch for {src_path}");
}

#[test]
fn goldens_match() {
    for (src, golden) in ENABLED {
        assert_emit_golden(src, golden);
        let src_text = fs::read_to_string(src).unwrap();
        let prog = parser::parse(&src_text).unwrap();
        assert!(
            types::check(&prog).is_empty(),
            "{src} unexpectedly produced diagnostics"
        );
    }
    for (src, golden) in COMPILE_ENABLED {
        assert_diag_golden(src, golden);
    }
    for (src, golden) in PRESS_ENABLED {
        let src_text = fs::read_to_string(src).unwrap();
        let golden_text = fs::read_to_string(golden).unwrap();
        let prog = parser::parse(&src_text).unwrap();
        let st = runtime::run(&prog);
        let got = format!("{}\n", emit::press(&st, &prog.comments));
        assert_eq!(got, golden_text, "press mismatch for {src}");
    }
}

#[test]
fn json_projection_exposes_the_discrepancy() {
    let src = fs::read_to_string("examples/03_rising_lion.yahu").unwrap();
    let st = runtime::run(&parser::parse(&src).unwrap());
    let json = emit::to_json(&st);
    assert!(json.contains("\"discrepancies\":1"), "json: {json}");
    assert!(json.contains("\"ended_by_elections\":false"));
    assert!(json.contains("casualties == 0"));
    assert!(json.contains("casualties=100"));
}

#[test]
fn enabled_examples_are_complete() {
    let mut fixtures: Vec<String> = fs::read_dir("tests/golden")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().to_string())
        .filter(|p| p.ends_with(".emit") || p.ends_with(".diag") || p.ends_with(".press"))
        .collect();
    fixtures.sort();
    let referenced: Vec<String> = ENABLED
        .iter()
        .chain(COMPILE_ENABLED.iter())
        .chain(PRESS_ENABLED.iter())
        .map(|(_, g)| g.to_string())
        .collect();
    for f in &fixtures {
        assert!(
            referenced.contains(f),
            "golden fixture {f} exists but is not referenced by an enabled test"
        );
    }
}
