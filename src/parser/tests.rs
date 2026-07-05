use super::*;
use crate::ast::{DeclRhs, Stmt};

#[test]
fn parses_operation_and_body() {
    let p =
        parse("@operation(\"Rising Lion\")\ncasualties = 100;\ndeclare(casualties == 0);").unwrap();
    assert_eq!(p.op_name, "Rising Lion");
    assert_eq!(
        p.body,
        vec![
            Stmt::Assign {
                var: "casualties".into(),
                value: 100
            },
            Stmt::Declare {
                lhs: "casualties".into(),
                rhs: DeclRhs::Int(0)
            },
        ]
    );
}

#[test]
fn parses_hasbara_with_action() {
    let p =
        parse("@operation(\"Protective Edge\")\nhasbara(\"self-defense\") { neutralize(target); }")
            .unwrap();
    assert_eq!(
        p.body,
        vec![Stmt::Hasbara {
            talking_point: "self-defense".into(),
            body: vec![Stmt::Action {
                verb: "neutralize".into(),
                target: "target".into(),
                self_defense: false
            }]
        }]
    );
}

#[test]
fn assert_is_an_alias_of_declare() {
    let p = parse("@operation(\"Iron Wall\")\nx = 1;\nassert(x == 1);").unwrap();
    assert!(matches!(p.body[1], Stmt::Declare { .. }));
}

#[test]
fn missing_operation_is_an_error() {
    assert!(parse("x = 1;").is_err());
}

#[test]
fn unterminated_block_is_an_error() {
    assert!(parse("@operation(\"X\")\nhasbara(\"y\") { neutralize(target);").is_err());
}
