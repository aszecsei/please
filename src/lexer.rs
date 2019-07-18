use crate::error;

use internship::IStr;
use serde::{Serialize, Deserialize};
use strum_macros::Display;

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Display, PartialEq, Clone, Deserialize, Serialize)]
pub enum TokenKind {
    #[strum(serialize = "@")]
    At,
    #[strum(serialize = ":")]
    Colon,
    #[strum(serialize = "=")]
    Assign,
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "+=")]
    AddAssign,

    #[strum(serialize = "(")]
    ParenL,
    #[strum(serialize = ")")]
    ParenR,
    #[strum(serialize = "{{")]
    InterpolationStart,
    #[strum(serialize = "}}")]
    InterpolationEnd,

    #[strum(serialize = "indent")]
    Indent,
    #[strum(serialize = "dedent")]
    Dedent,

    // Strings
    Identifier(IStr),
    Keyword(IStr),
    Command(IStr),
}

const KEYWORDS: &[&str] = &[
    "for",
    "in",
    "if",
    "else",
];

pub fn interned_keywords() -> Vec<IStr> {
    KEYWORDS.iter().map(|k| IStr::from(*k)).collect()
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Token<'a> {
    pub line: usize,
    pub col: usize,
    pub kind: TokenKind,
    pub filename: &'a str,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl<'a> Token<'a> {
    pub fn error(&self, kind: error::CompilationErrorKind) -> error::CompilationError {
        error::CompilationError {
            column: self.col,
            line: self.line,
            filename: String::from(self.filename),
            kind,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    Normal,
    Indented { indentation: usize },
    Text,
    Interpolation { interpolation_start_col: usize, interpolation_start_row: usize }
}

pub struct Lexer<'a> {
    /// Source filename
    filename: &'a str,
    /// Peekable char iterator
    input: Peekable<Chars<'a>>,
    /// Current token
    token: Option<Token<'a>>,
    /// Current char
    ch: Option<char>,
    /// Current line
    line: usize,
    /// Current column
    col: usize,
    /// Accrued errors
    errs: Vec<failure::Error>,
    /// State stack
    state: Vec<State>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = self.advance();
        while res.is_err() {
            self.errs.push(res.err().expect("unable to retrieve error"));
            res = self.advance();
        }
        self.token.clone()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // The lower bound is 0 - we might exclusively have whitespace, for example
        // However, no token is less than 0 characters, so our upper bound is equal to the number of characters
        (0, self.input.size_hint().1)
    }
}

// Because our lexer will always return None at the end of file, we can mark it as a fused iterator.
// Unsure if this impacts anything perf-wise, but it's good to know?
impl<'a> std::iter::FusedIterator for Lexer<'a> {}

// Public methods
impl<'a> Lexer<'a> {
    /// Lexes a file and returns an Option<Vec<Token>> - this has a value if there were no errors,
    /// and is None otherwise.
    pub fn lex(text: &'a str, filename: &'a str) -> error::Result<Vec<Token<'a>>> {
        let mut lex = Lexer::new(text, filename)?;
        let mut res = Vec::new();
        while let Some(t) = lex.next() {
            res.push(t);
        }
        if lex.errs.is_empty() {
            Ok(res)
        } else {
            Err(failure::Error::from(error::MultipleErrors { errs: lex.errs }))
        }
    }
}

// Private methods
impl<'a> Lexer<'a> {
    fn new(text: &'a str, filename: &'a str) -> error::Result<Self> {
        let mut res = Self {
            filename,
            input: text.chars().peekable(),
            token: None,
            ch: Some('\0'),
            line: 0,
            col: 0,
            errs: Vec::new(),
            state: vec![State::Normal],
        };
        res.bump()?;
        Ok(res)
    }

    fn advance(&mut self) -> error::Result<()> {
        if self.is_eof() {
            self.token = None;
            return Ok(())
        }

        match self.state()? {
            State::Normal => self.lex_normal()?,
            _ => return Err(self.internal_error("Unexpected state")),
        }

        Ok(())
    }

    fn lex_normal(&mut self) -> error::Result<()> {
        if self.ch_is('@') {
            self.lex_single(TokenKind::At)?;
        } else if self.ch_is(':') {
            self.lex_single(TokenKind::Colon)?;
        } else if self.ch_is('(') {
            self.lex_single(TokenKind::ParenL)?;
        } else if self.ch_is(')') {
            self.lex_single(TokenKind::ParenR)?;
        } else if is_ident_start(self.ch) {
            self.lex_identifier()?;
        } else {
            let err = self.error(error::CompilationErrorKind::UnexpectedChar {
                ch: self.ch.unwrap()
            });
            self.bump()?;
            return Err(err);
        }
        Ok(())
    }

    fn state(&self) -> error::Result<State> {
        if self.state.is_empty() {
            Err(self.internal_error("attempted to access empty lexer state stack"))
        } else {
            Ok(self.state[self.state.len() - 1])
        }
    }

    fn pop_state(&mut self) -> error::Result<()> {
        if self.state.pop().is_none() {
            return Err(self.internal_error("attempted to pop empty lexer state stack"));
        } else {
            Ok(())
        }
    }

    /// Lex a single character token
    fn lex_single(&mut self, kind: TokenKind) -> error::Result<()> {
        let token = self.make_token(kind);
        self.bump()?;
        self.token = Some(token);
        Ok(())
    }

    /// Lex a double character token
    fn lex_double(&mut self, kind: TokenKind) -> error::Result<()> {
        let token = self.make_token(kind);
        self.bump()?;
        self.bump()?;
        self.token = Some(token);
        Ok(())
    }

    /// Lex identifier: [a-zA-Z_][a-zA-Z0-9_]*
    fn lex_identifier(&mut self) -> error::Result<()> {
        let (line, col) = (self.line, self.col);
        let mut ident = String::new();
        while is_ident_continue(self.ch) {
            ident.push(self.ch.unwrap());
            self.bump()?;
        }
        let interned = IStr::from(ident);
        let kind = if interned_keywords().contains(&interned) {
            TokenKind::Keyword(interned)
        } else {
            TokenKind::Identifier(interned)
        };
        self.token = Some(Token {
            line,
            col,
            kind,
            filename: self.filename
        });
        Ok(())
    }

    fn make_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            line: self.line,
            col: self.col,
            kind,
            filename: self.filename,
        }
    }

    #[inline]
    fn bump(&mut self) -> error::Result<()> {
        if let Some(c) = self.ch {
            match c {
                '\n' => {
                    self.col = 0;
                    self.line += 1;
                },
                '\0' => {},
                _ => {
                    self.col += c.len_utf8();
                }
            }
            self.ch = self.input.next();
            Ok(())
        } else {
            Err(self.internal_error("Lexer advanced past end of text"))
        }
    }

    #[inline]
    fn nextch(&mut self) -> Option<char> {
        self.input.peek().cloned()
    }

    #[inline]
    fn ch_is(&self, ch: char) -> bool {
        self.ch == Some(ch)
    }

    #[inline]
    fn nextch_is(&mut self, c: char) -> bool {
        self.nextch() == Some(c)
    }

    #[inline]
    fn expect(&mut self, c: char) -> error::Result<()> {
        if self.ch != Some(c) {
            let e = Err(self.error(error::CompilationErrorKind::ExpectedButGot {
                expected: c,
                got: self.ch.unwrap_or('\0')
            }));
            self.bump()?;
            e
        } else {
            self.bump()?;
            Ok(())
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.ch.is_none()
    }

    #[inline]
    fn error(&self, kind: error::CompilationErrorKind) -> failure::Error {
        failure::Error::from(error::CompilationError::new(kind, self.filename.to_owned(), self.line + 1, self.col + 1)) // Add 1 to line and col to offset 0-indexing
    }

    #[inline]
    fn internal_error<T: Into<String>>(&self, message: T) -> failure::Error {
        self.error(error::CompilationErrorKind::Internal {
            message: message.into()
        })
    }
}

#[inline]
fn is_in_range(c: Option<char>, low: char, high: char) -> bool {
    c.map_or(false, |c| low <= c && c <= high)
}

#[inline]
fn is_decimal_digit(c: Option<char>) -> bool {
    is_in_range(c, '0', '9')
}

#[inline]
fn is_ident_start(c: Option<char>) -> bool {
    is_in_range(c, 'a', 'z') || is_in_range(c, 'A', 'Z') || c == Some('_')
}

#[inline]
fn is_ident_continue(c: Option<char>) -> bool {
    is_ident_start(c) || is_decimal_digit(c)
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot_matches;

    #[test]
    fn test_lexer_rustplease() -> std::io::Result<()> {
        let filename = "./examples/rust.please";
        let buf = std::fs::read_to_string(filename)?;
        let tokens = super::Lexer::lex(&buf, filename).unwrap();
        assert_yaml_snapshot_matches!("tokens", tokens);
        Ok(())
    }
}