use crate::model::SODI;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Operation,
    Str(String),
    Int(i64),
    Ident(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Eq,
    Comment(String),
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AmpAmp,
    PipePipe,
    Bang,
    Eof,
}

impl std::fmt::Display for Tok {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Tok::Operation => "`@operation`",
            Tok::Str(s) => return write!(f, "string \"{s}\""),
            Tok::Int(n) => return write!(f, "integer `{n}`"),
            Tok::Ident(name) => return write!(f, "identifier `{name}`"),
            Tok::LParen => "`(`",
            Tok::RParen => "`)`",
            Tok::LBrace => "`{`",
            Tok::RBrace => "`}`",
            Tok::LBracket => "`[`",
            Tok::RBracket => "`]`",
            Tok::Comma => "`,`",
            Tok::Semi => "`;`",
            Tok::Colon => "`:`",
            Tok::Eq => "`=`",
            Tok::Comment(_) => "a comment",
            Tok::EqEq => "`==`",
            Tok::Ne => "`!=`",
            Tok::Lt => "`<`",
            Tok::Le => "`<=`",
            Tok::Gt => "`>`",
            Tok::Ge => "`>=`",
            Tok::Plus => "`+`",
            Tok::Minus => "`-`",
            Tok::Star => "`*`",
            Tok::Slash => "`/`",
            Tok::Percent => "`%`",
            Tok::AmpAmp => "`&&`",
            Tok::PipePipe => "`||`",
            Tok::Bang => "`!`",
            Tok::Eof => "the end of the program",
        };
        f.write_str(text)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: u32,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lex error (line {}): {}", self.line, self.message)
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Tokens that are always one character long.
fn single_char_tok(c: char) -> Option<Tok> {
    Some(match c {
        '+' => Tok::Plus,
        '-' => Tok::Minus,
        '*' => Tok::Star,
        '/' => Tok::Slash,
        '%' => Tok::Percent,
        '(' => Tok::LParen,
        ')' => Tok::RParen,
        '{' => Tok::LBrace,
        '}' => Tok::RBrace,
        '[' => Tok::LBracket,
        ']' => Tok::RBracket,
        ',' => Tok::Comma,
        ';' => Tok::Semi,
        ':' => Tok::Colon,
        _ => return None,
    })
}

/// Tokens that gain a longer form when followed by `=`, such as `<` and `<=`.
fn eq_suffixed_tok(c: char) -> Option<(Tok, Tok)> {
    match c {
        '=' => Some((Tok::Eq, Tok::EqEq)),
        '!' => Some((Tok::Bang, Tok::Ne)),
        '<' => Some((Tok::Lt, Tok::Le)),
        '>' => Some((Tok::Gt, Tok::Ge)),
        _ => None,
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut line: u32 = 1;
    let mut out = Vec::new();

    while i < n {
        let c = chars[i];
        if let Some(tok) = single_char_tok(c) {
            out.push(Token { tok, line });
            i += 1;
            continue;
        }
        if let Some((short, long)) = eq_suffixed_tok(c) {
            if i + 1 < n && chars[i + 1] == '=' {
                out.push(Token { tok: long, line });
                i += 2;
            } else {
                out.push(Token { tok: short, line });
                i += 1;
            }
            continue;
        }
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            c if c.is_whitespace() => {
                i += 1;
            }
            // Collected, not discarded: the press build rewrites comments.
            '#' => {
                i += 1;
                let start = i;
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                out.push(Token {
                    tok: Tok::Comment(text.trim().to_string()),
                    line,
                });
            }
            '@' => {
                i += 1;
                let start = i;
                while i < n && is_ident_continue(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if word == "operation" {
                    out.push(Token {
                        tok: Tok::Operation,
                        line,
                    });
                } else {
                    return Err(LexError {
                        message: format!("unknown decorator `@{word}` (only `@operation` exists)"),
                        line,
                    });
                }
            }
            '"' => {
                i += 1;
                let start = i;
                while i < n && chars[i] != '"' {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
                if i >= n {
                    return Err(LexError {
                        message: "unterminated string literal".into(),
                        line,
                    });
                }
                let s: String = chars[start..i].iter().collect();
                i += 1;
                out.push(Token {
                    tok: Tok::Str(s),
                    line,
                });
            }
            '&' => {
                if i + 1 < n && chars[i + 1] == '&' {
                    i += 2;
                    out.push(Token {
                        tok: Tok::AmpAmp,
                        line,
                    });
                } else {
                    return Err(LexError {
                        message: "unexpected `&`. Did you mean `&&`?".into(),
                        line,
                    });
                }
            }
            '|' => {
                if i + 1 < n && chars[i + 1] == '|' {
                    i += 2;
                    out.push(Token {
                        tok: Tok::PipePipe,
                        line,
                    });
                } else {
                    return Err(LexError {
                        message: "unexpected `|`. Did you mean `||`?".into(),
                        line,
                    });
                }
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < n && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let v: i64 = text.parse().map_err(|_| LexError {
                    message: format!("integer literal out of range: {text}"),
                    line,
                })?;
                out.push(Token {
                    tok: Tok::Int(v),
                    line,
                });
            }
            c if is_ident_start(c) => {
                let start = i;
                while i < n && is_ident_continue(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                out.push(Token {
                    tok: Tok::Ident(word),
                    line,
                });
            }
            _ if src_matches_at(&chars, i, SODI) => {
                let m: Vec<char> = SODI.chars().collect();
                i += m.len();
                out.push(Token {
                    tok: Tok::Ident(SODI.to_string()),
                    line,
                });
            }
            other => {
                return Err(LexError {
                    message: format!("unexpected character `{other}`"),
                    line,
                });
            }
        }
    }
    out.push(Token {
        tok: Tok::Eof,
        line,
    });
    Ok(out)
}

fn src_matches_at(chars: &[char], i: usize, pat: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    i + p.len() <= chars.len() && chars[i..i + p.len()] == p[..]
}

#[cfg(test)]
mod tests;
