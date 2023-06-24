use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position<'a> {
    /// The name of the target file of the lexer constructor (new method).
    pub file_name: &'a str,
    /// `self.text`'s current position line(1 based index)
    pub line: usize,
}

impl Default for Position<'_> {
    fn default() -> Self {
        Self {
            file_name: Default::default(),
            line: 1,
        }
    }
}

impl fmt::Display for Position<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file_name, self.line)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    Ident,
    Number,
    String,
    Comma,
    Colon,
    Lpar,
    Rpar,
    Plus,
    Minus,
    Mul,
    Div,
    Percent,
    Dolor,
    /// enf of line
    Eol,
    /// end of file
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
pub enum TokenKindError {
    #[error("Couldn't cast to TokenKind. Unexpected: {0}")]
    UnexpectedChar(char),
    #[error("Couldn't cast to &str. Unexpected: {0:?}")]
    UnexpectedKind(TokenKind),
}

impl TryFrom<char> for TokenKind {
    type Error = TokenKindError;

    fn try_from(s: char) -> Result<Self, Self::Error> {
        Ok(match s {
            ',' => TokenKind::Comma,
            ':' => TokenKind::Comma,
            '(' => TokenKind::Lpar,
            ')' => TokenKind::Rpar,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '%' => TokenKind::Percent,
            '$' => TokenKind::Dolor,
            '\n' => TokenKind::Eol,
            '\0' => TokenKind::Eof,
            _ => return Err(TokenKindError::UnexpectedChar(s)),
        })
    }
}

impl<'a> TryFrom<TokenKind> for &'a str {
    type Error = TokenKindError;

    fn try_from(s: TokenKind) -> Result<Self, Self::Error> {
        Ok(match s {
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::Lpar => "(",
            TokenKind::Rpar => ")",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",
            TokenKind::Percent => "%",
            TokenKind::Dolor => "$",
            TokenKind::Eol => "\n",
            TokenKind::Eof => "\0",
            _ => return Err(TokenKindError::UnexpectedKind(s)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub pos: Position<'a>,
    pub lit: &'a str,
}
