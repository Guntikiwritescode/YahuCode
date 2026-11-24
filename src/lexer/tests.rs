use super::*;

fn toks(src: &str) -> Vec<Tok> {
    lex(src).unwrap().into_iter().map(|t| t.tok).collect()
}

#[test]
fn lexes_operation_header() {
    assert_eq!(
        toks(r#"@operation("Protective Edge")"#),
        vec![
            Tok::Operation,
            Tok::LParen,
            Tok::Str("Protective Edge".into()),
            Tok::RParen,
            Tok::Eof,
        ]
    );
}

#[test]
fn lexes_assignment_and_declare() {
    assert_eq!(
        toks("casualties = 100;\ndeclare(casualties == 0);"),
        vec![
            Tok::Ident("casualties".into()),
            Tok::Eq,
            Tok::Int(100),
            Tok::Semi,
            Tok::Ident("declare".into()),
            Tok::LParen,
            Tok::Ident("casualties".into()),
            Tok::EqEq,
            Tok::Int(0),
            Tok::RParen,
            Tok::Semi,
            Tok::Eof,
        ]
    );
}

#[test]
fn lexes_braces_and_action() {
    assert_eq!(
        toks("hasbara(\"x\") { neutralize(target); }"),
        vec![
            Tok::Ident("hasbara".into()),
            Tok::LParen,
            Tok::Str("x".into()),
            Tok::RParen,
            Tok::LBrace,
            Tok::Ident("neutralize".into()),
            Tok::LParen,
            Tok::Ident("target".into()),
            Tok::RParen,
            Tok::Semi,
            Tok::RBrace,
            Tok::Eof,
        ]
    );
}

#[test]
fn tracks_line_numbers() {
    let ts = lex("a = 1;\nb = 2;").unwrap();
    let semis: Vec<u32> = ts
        .iter()
        .filter(|t| t.tok == Tok::Semi)
        .map(|t| t.line)
        .collect();
    assert_eq!(semis, vec![1, 2]);
}

#[test]
fn rejects_unknown_decorator() {
    assert!(lex("@bogus(\"x\")").is_err());
}

#[test]
fn rejects_unterminated_string() {
    assert!(lex("@operation(\"oops").is_err());
}
