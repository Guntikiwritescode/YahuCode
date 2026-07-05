//! The lexer: source → tokens. A proper hand-written scanner (replacing the spike's
//! ad-hoc token regex), with line tracking for diagnostics.
//!
//! The token set is closed and grows with the grammar. This is the Phase-0 subset
//! plus the punctuation the full grammar needs.

use crate::model::SODI;

/// A lexical token.
#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// `@operation`
    Operation,
    /// a `"…"` string literal (contents, no surrounding quotes)
    Str(String),
    /// `==`
    EqEq,
    /// an integer literal
    Int(i64),
    /// an identifier / keyword
    Ident(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Eq,
    Eof,
}

/// A token with its source line (1-based).
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
}

/// A lexing failure — an *implementation/user input* error, surfaced loudly (§12),
/// never swallowed.
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

/// Tokenize `src`. The `סודי` cast keyword is recognized as an identifier token so the
/// parser can treat it uniformly with `PUBLIC`/`RESTRICTED` cast targets.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut line: u32 = 1;
    let mut out = Vec::new();

    while i < n {
        let c = chars[i];
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            c if c.is_whitespace() => {
                i += 1;
            }
            '@' => {
                // Only `@operation` exists in the grammar.
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
                i += 1; // closing quote
                out.push(Token {
                    tok: Tok::Str(s),
                    line,
                });
            }
            '=' => {
                if i + 1 < n && chars[i + 1] == '=' {
                    i += 2;
                    out.push(Token {
                        tok: Tok::EqEq,
                        line,
                    });
                } else {
                    i += 1;
                    out.push(Token { tok: Tok::Eq, line });
                }
            }
            '(' => {
                i += 1;
                out.push(Token {
                    tok: Tok::LParen,
                    line,
                });
            }
            ')' => {
                i += 1;
                out.push(Token {
                    tok: Tok::RParen,
                    line,
                });
            }
            '{' => {
                i += 1;
                out.push(Token {
                    tok: Tok::LBrace,
                    line,
                });
            }
            '}' => {
                i += 1;
                out.push(Token {
                    tok: Tok::RBrace,
                    line,
                });
            }
            ',' => {
                i += 1;
                out.push(Token {
                    tok: Tok::Comma,
                    line,
                });
            }
            ';' => {
                i += 1;
                out.push(Token {
                    tok: Tok::Semi,
                    line,
                });
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
            // The `סודי` clearance keyword (Hebrew) — a valid cast target / identifier.
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
                    message: format!("unexpected character {other:?}"),
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
