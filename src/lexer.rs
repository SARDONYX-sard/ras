//! Simple hand-written assembler lexer
//!
//! Copyright 2023 rust-analyzer
//! MIT license: https://opensource.org/license/mit/
//! https://github.com/rust-analyzer/ungrammar/blob/20bc271547bb130f282c704f736e4989743ce332/Cargo.toml#L5

use std::str::Chars;

use crate::error::{bail, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    /// An identifier
    Ident(String),
    /// str literal e.g.: 'hello', "World"
    Token(String),
    Number(String),
    Plus,
    Mul,
    Minus,
    Div,
    Dolor,
    Percent,
    Colon,
    Comma,
    LParen,
    RParen,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) loc: Location,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct Location {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Location {
    fn advance(&mut self, text: &str) {
        match text.rfind('\n') {
            Some(idx) => {
                self.line += text.chars().filter(|&it| it == '\n').count();
                self.column = text[idx + 1..].chars().count();
            }
            None => self.column += text.chars().count(),
        }
    }
}

pub(crate) fn tokenize(mut input: &str) -> Result<Vec<Token>> {
    let mut res = Vec::new();
    let mut loc = Location::default();
    while !input.is_empty() {
        let old_input = input;
        skip_ws(&mut input);
        skip_comment(&mut input);
        if old_input.len() == input.len() {
            match advance(&mut input) {
                Ok(kind) => {
                    res.push(Token { kind, loc });
                }
                Err(err) => return Err(err.with_location(loc)),
            }
        }
        let consumed = old_input.len() - input.len();
        loc.advance(&old_input[..consumed]);
    }

    Ok(res)
}

fn skip_ws(input: &mut &str) {
    *input = input.trim_start_matches(is_whitespace)
}
fn skip_comment(input: &mut &str) {
    if input.starts_with('#') {
        let idx = input.find('\n').map_or(input.len(), |it| it + 1);
        *input = &input[idx..]
    }
}

fn advance(input: &mut &str) -> Result<TokenKind> {
    let mut chars = input.chars();
    let c = chars.next().unwrap();
    let res = match c {
        ',' => TokenKind::Comma,
        '+' => TokenKind::Plus,
        '-' => TokenKind::Minus,
        '*' => TokenKind::Mul,
        '/' => TokenKind::Div,
        '%' => TokenKind::Percent,
        '$' => TokenKind::Dolor,
        ':' => TokenKind::Colon,
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '\'' => take_until('\'', &mut chars)?,
        '\"' => take_until('\"', &mut chars)?,
        c if c.is_ascii_digit() => {
            let mut buf = String::new();
            buf.push(c);
            loop {
                match chars.clone().next() {
                    Some(c) if is_number_char(c) => {
                        chars.next();
                        buf.push(c);
                    }
                    _ => break,
                }
            }
            TokenKind::Number(buf)
        }
        c if is_ident_char(c) => {
            let mut buf = String::new();
            buf.push(c);
            loop {
                match chars.clone().next() {
                    Some(c) if is_ident_char(c) => {
                        chars.next();
                        buf.push(c);
                    }
                    _ => break,
                }
            }
            TokenKind::Ident(buf)
        }
        '\r' => bail!("unexpected `\\r`, only Unix-style line endings allowed"),
        c => bail!("unexpected character: `{}`", c),
    };

    *input = chars.as_str();
    Ok(res)
}

/// Create TokenKind::Token
fn take_until(ch: char, chars: &mut Chars<'_>) -> Result<TokenKind> {
    let mut buf = String::new();
    loop {
        match chars.next() {
            None => bail!("unclosed token literal"),
            Some(c) if ch == c => break,
            Some('\\') => match chars.next() {
                Some(c) if is_escapable(c) => buf.push(to_escape_char(c)?),
                c => bail!("unsupported escape literal. Got {c:?}"),
            },
            Some(c) => buf.push(c),
        }
    }
    Ok(TokenKind::Token(buf))
}

fn to_escape_char(c: char) -> Result<char> {
    Ok(match c {
        '\'' => '\'',
        '"' => '\"',
        '\\' => '\\',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        c => bail!("invalid escape character. Got {c}"),
    })
}
fn is_escapable(c: char) -> bool {
    matches!(c, '\'' | '"' | '\\' | 'n' | 'r' | 't' | '0')
}
fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n')
}
fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '.')
}
fn is_number_char(c: char) -> bool {
    c.is_ascii_hexdigit()
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::lexer::{tokenize, Location, Token, TokenKind};
    use pretty_assertions::assert_eq;

    #[test]
    fn debug_tokenize() -> Result<()> {
        let asm_code = r#"# This line is comment. Should be skipped.
.text
.global _start
_start:
    mov eax, eax
    lea e, 0x10
"#;

        let actual = tokenize(asm_code)?;
        assert_eq!(
            vec![
                Token {
                    kind: TokenKind::Ident(".text".to_owned()),
                    loc: Location { line: 1, column: 0 },
                },
                Token {
                    kind: TokenKind::Ident(".global".to_owned()),
                    loc: Location { line: 2, column: 0 },
                },
                Token {
                    kind: TokenKind::Ident("_start".to_owned()),
                    loc: Location { line: 2, column: 8 },
                },
                Token {
                    kind: TokenKind::Ident("_start".to_owned()),
                    loc: Location { line: 3, column: 0 },
                },
                Token {
                    kind: TokenKind::Colon,
                    loc: Location { line: 3, column: 6 },
                },
                Token {
                    kind: TokenKind::Ident("mov".to_owned()),
                    loc: Location { line: 4, column: 4 },
                },
                Token {
                    kind: TokenKind::Ident("eax".to_owned()),
                    loc: Location { line: 4, column: 8 },
                },
                Token {
                    kind: TokenKind::Comma,
                    loc: Location {
                        line: 4,
                        column: 11,
                    },
                },
                Token {
                    kind: TokenKind::Ident("eax".to_owned()),
                    loc: Location {
                        line: 4,
                        column: 13,
                    },
                },
                Token {
                    kind: TokenKind::Ident("lea".to_owned()),
                    loc: Location { line: 5, column: 4 },
                },
                Token {
                    kind: TokenKind::Ident("e".to_owned()),
                    loc: Location { line: 5, column: 8 },
                },
                Token {
                    kind: TokenKind::Comma,
                    loc: Location { line: 5, column: 9 },
                },
                Token {
                    kind: TokenKind::Number("0".to_owned()),
                    loc: Location {
                        line: 5,
                        column: 11,
                    },
                },
                Token {
                    kind: TokenKind::Ident("x".to_owned()),
                    loc: Location {
                        line: 5,
                        column: 12,
                    },
                },
                Token {
                    kind: TokenKind::Number("10".to_owned()),
                    loc: Location {
                        line: 5,
                        column: 13,
                    },
                },
            ],
            actual
        );
        Ok(())
    }
}
