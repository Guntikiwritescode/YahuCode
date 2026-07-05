use super::*;
use crate::ast::{BinOp, Expr, Stmt};

fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
    Expr::BinOp {
        op,
        lhs: Box::new(l),
        rhs: Box::new(r),
    }
}

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
                value: Expr::Int(100),
            },
            Stmt::Declare(bin(BinOp::Eq, Expr::Var("casualties".into()), Expr::Int(0))),
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
                self_defense: false,
            }],
        }]
    );
}

#[test]
fn assert_is_an_alias_of_declare() {
    let p = parse("@operation(\"Iron Wall\")\nx = 1;\nassert(x == 1);").unwrap();
    assert!(matches!(p.body[1], Stmt::Declare(_)));
}

#[test]
fn parses_control_flow_and_functions() {
    let src = "@operation(\"Iron Dome\")\n\
        func sq(n) { return n * n; }\n\
        x = 0;\n\
        while (x < 3) { x = x + 1; }\n\
        if (x == 3) { y = sq(x); } else { y = 0; }";
    let p = parse(src).unwrap();
    assert!(matches!(p.body[0], Stmt::FuncDef { .. }));
    assert!(matches!(p.body[2], Stmt::While { .. }));
    assert!(matches!(p.body[3], Stmt::If { .. }));
}

#[test]
fn precedence_is_respected() {
    // 1 + 2 * 3  parses as  1 + (2 * 3)
    let p = parse("@operation(\"X\")\nr = 1 + 2 * 3;").unwrap();
    let Stmt::Assign { value, .. } = &p.body[0] else {
        panic!("expected assign");
    };
    assert_eq!(
        *value,
        bin(
            BinOp::Add,
            Expr::Int(1),
            bin(BinOp::Mul, Expr::Int(2), Expr::Int(3))
        )
    );
}

#[test]
fn user_call_is_not_an_action() {
    // `f(x);` where f is not an action verb → an expression statement (call).
    let p = parse("@operation(\"X\")\nf(x);").unwrap();
    assert!(matches!(p.body[0], Stmt::ExprStmt(Expr::Call { .. })));
}

#[test]
fn missing_operation_is_an_error() {
    assert!(parse("x = 1;").is_err());
}

#[test]
fn unterminated_block_is_an_error() {
    assert!(parse("@operation(\"X\")\nhasbara(\"y\") { neutralize(target);").is_err());
}
