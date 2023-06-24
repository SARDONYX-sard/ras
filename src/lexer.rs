use crate::token::{Position, Token, TokenKind, TokenKindError};

/// Tokenizer
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lexer<'a> {
    /// Current character
    c: Option<char>,
    /// Source code of assembly
    text: &'a str,
    /// `self.text`'s current position index(0 based index)
    idx: usize,
    /// `self.text`'s current position line(1 based index)
    line: usize,
    /// `self.text`'s current position column.(0 based index)
    col: usize,
    /// The name of the target file of the lexer constructor (new method).
    file_name: &'a str,
}

impl Default for Lexer<'_> {
    fn default() -> Self {
        let Position { line, file_name } = Position::default();
        Self {
            c: Default::default(),
            text: Default::default(),
            idx: Default::default(),
            line,
            col: Default::default(),
            file_name,
        }
    }
}

impl<'a> Lexer<'a> {
    /// Tokenize assembly code.
    /// # Examples
    /// ```
    /// use crate::{Lexer, Position, Token, TokenKind};
    ///        let test_name = "test"
    ///         let asm_code = r#"
    ///     # This line is comment. Should be skipped.
    /// .text
    /// .global _start
    /// _start:
    ///     mov eax, eax
    ///     lea e, 1
    /// "#;
    ///
    ///         let mut lexer = Lexer::new(test_name, asm_code);
    ///         assert_eq!(
    ///             lexer.lex(),
    ///             Token {
    ///                 kind: TokenKind::Ident,
    ///                 pos: Position {
    ///                     file_name: test_name,
    ///                     line: 3
    ///                 },
    ///                 lit: ".text"
    ///             }
    ///         );
    /// ```
    pub fn new(file_name: &'a str, text: &'a str) -> Self {
        let c = match text.is_empty() {
            true => None,
            false => text.chars().nth(0),
        };
        Self {
            c,
            text,
            file_name,
            ..Default::default()
        }
    }

    /// Advance the current character position.  & increment `self.idx`
    fn advance(&mut self) {
        self.col += 1;
        self.idx += 1;

        if self.c == Some('\n') {
            self.col += 0;
            self.line += 1;
        };

        self.c = self.peek(0);
    }

    /// - If self.text is EOF, returns '\0'.
    /// - If not, return the `self.idx + n`th `char`.
    /// - If there is no `self.idx + n`th char, return `None`.
    fn peek(&self, n: usize) -> Option<char> {
        match self.text.len() == self.idx + n {
            true => None,
            false => self.text.chars().nth(self.idx + n),
        }
    }

    fn current_pos(&self) -> Position {
        Position {
            line: self.line,
            file_name: self.file_name,
        }
    }

    /// Advance position until the end of the line.
    fn skip_comment(&mut self) {
        while self.c != Some('\n') {
            self.advance();
        }
    }

    fn is_hex(&self) -> bool {
        let next_start_with_0x = || {
            (self.c == Some('0'))
                && (matches!(self.text.chars().nth(self.idx + 1), Some('x') | Some('X')))
        };
        let is_code_end = self.text.len() == self.idx + 1;

        match is_code_end {
            true => false, // Obvious: EOF is not a hex.
            false => next_start_with_0x(),
        }
    }

    /// Advance position until digit.
    fn read_number(&mut self) -> Token {
        // Copy the starting point here because the advance method increments self.idx.
        let start = self.idx;

        match self.is_hex() {
            true => {
                // consume "0x"
                self.advance(); // consume '0'
                self.advance(); // consume 'x'

                while Some(true) == self.c.map(|c| c.is_digit(16)) {
                    self.advance();
                }
            }
            false => {
                while Some(true) == self.c.map(|c| c.is_digit(10)) {
                    self.advance();
                }
            }
        };

        Token {
            lit: &self.text[start..self.idx],
            kind: TokenKind::Number,
            pos: self.current_pos(),
        }
    }

    /// Advance position until is_ident is true.
    /// # Note
    /// This method recognizes a decimal number as `TokenKind::Ident` even if it comes first.
    ///
    /// e.g. `01ident` -> `TokenKind::Ident`
    ///
    /// Therefore, users need to parse numbers with this in mind.
    fn read_ident(&mut self) -> Token {
        fn is_ident(c: char) -> bool {
            c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '$')
        }

        let start = self.idx;

        while Some(true) == self.c.map(|c| is_ident(c)) {
            self.advance();
        }

        Token {
            lit: &self.text[start..self.idx],
            kind: TokenKind::Ident,
            pos: self.current_pos(),
        }
    }

    /// Advance position until next `"`.
    fn read_string(&mut self) -> Token {
        let start = self.idx;

        while Some(false) == self.c.map(|c| matches!(c, '"')) {
            self.advance();
        }

        Token {
            lit: &self.text[start..self.idx],
            kind: TokenKind::String,
            pos: self.current_pos(),
        }
    }

    /// Determine the token type from the given characters
    /// and create and return a Token structure along with the current position.
    fn single_letter_token(&mut self, c: char) -> Result<Token, TokenKindError> {
        self.advance();
        let kind: TokenKind = c.try_into()?;

        Ok(Token {
            lit: kind.clone().try_into()?,
            kind,
            pos: self.current_pos(),
        })
    }

    pub fn lex(&mut self) -> Result<Token, LexerError> {
        while let Some(c) = self.c {
            // non return matches
            match c {
                c if c.is_whitespace() => {
                    self.advance();
                    continue;
                }
                '#' => {
                    self.skip_comment();
                    continue;
                }
                _ => {}
            };

            return Ok(match c {
                    '0'..='9' => self.read_number(),
                    'A'..='Z' | 'a'..='z' | '_' | '.' => self.read_ident(),
                    '"' => self.read_string(),
                    ',' | ':' | '(' | ')' | '+' | '-' | '*' | '/' | '$' | '%' => {
                        self.single_letter_token(c)
                        .expect("Non error. Reason: We are filtering out failed cast characters before calling this method.")
                    }
                    _ => return Err(LexerError::UnexpectedToken(c, self.current_pos())),
                });
        }

        Ok(Token {
            lit: "\0",
            kind: TokenKind::Eof,
            pos: self.current_pos(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
pub enum LexerError<'a> {
    #[error("Unexpected token {0}. {1}")]
    UnexpectedToken(char, Position<'a>),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        lexer::Lexer,
        token::{Position, Token, TokenKind, TokenKindError},
    };

    #[test]
    fn should_tokenize_str() {
        let asm_code = r#"
    # This line is comment. Should be skipped.
.text
.global _start
_start:
    mov eax, eax
    lea e, 1
"#;

        let mut lexer = Lexer::new(Default::default(), asm_code);
        assert_eq!(
            lexer.lex(),
            Ok(Token {
                lit: ".text",
                kind: TokenKind::Ident,
                pos: Position {
                    line: 3,
                    ..Default::default()
                },
            })
        );
    }

    #[test]
    fn should_tokenize_number() {
        let asm_code = "0x16";
        let mut lexer = Lexer::new(Default::default(), asm_code);
        assert_eq!(
            lexer.lex(),
            Ok(Token {
                lit: "0x16",
                kind: TokenKind::Number,
                pos: Position {
                    line: 1,
                    ..Default::default()
                },
            })
        );

        let asm_code = "16 x016";
        let mut lexer = Lexer::new(Default::default(), asm_code);
        assert_eq!(
            lexer.lex(),
            Ok(Token {
                lit: "16",
                kind: TokenKind::Number,
                pos: Position {
                    line: 1,
                    ..Default::default()
                },
            })
        );
    }

    #[test]
    fn should_cast_with_known_char() {
        let mut lexer = Lexer::default();
        assert_eq!(
            lexer.single_letter_token('%'),
            Ok(Token {
                lit: "%",
                kind: TokenKind::Percent,
                pos: Position::default(),
            })
        );
    }

    #[test]
    fn should_err_cast_with_unknown_char() {
        let mut lexer = Lexer::default();
        assert_eq!(
            lexer.single_letter_token('c'),
            Err(TokenKindError::UnexpectedChar('c'))
        );
    }
}
