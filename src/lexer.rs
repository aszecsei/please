use crate::error::{CompilationError, CompilationErrorKind};

use internship::IStr;
use strum_macros::Display;

use std::fmt;
use std::iter::Peekable;
use std::str::{Chars, FromStr};

#[derive(Debug, Display, PartialEq, Clone)]
pub enum TokenKind {
    #[strum(serialize = "@")]
    At,
    Identifier,
    RawString,
    CookedString,
    Command,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub offset: usize,
    pub length: usize,
    pub line: usize,
    pub col: usize,
    pub text: IStr,
    pub kind: TokenKind,
    pub filename: &'a str,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl<'a> Token<'a> {
    pub fn lexeme(&self) -> String {
        format!("{}", self.text)
    }

    pub fn error(&self, kind: CompilationErrorKind) -> CompilationError {
        CompilationError {
            column: self.col,
            line: self.line,
            filename: String::from(self.filename),
            kind,
        }
    }
}

pub struct Lexer<'a> {
    /// Source text
    text: &'a str,
    /// Source filename
    filename: &'a str,
    /// Peekable char iterator
    input: Peekable<Chars<'a>>,
    /// Current token
    token: Option<Token<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        self.token.clone()
    }
}

// Public methods
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, filename: &'a str) -> Self {
        Self {
            text,
            filename,
            input: text.chars().peekable(),
            token: None,
        }
    }

    pub fn lex(text: &'a str, filename: &'a str) -> Vec<Token<'a>> {
        Lexer::new(text, filename).collect()
    }
}

// Private methods
impl<'a> Lexer<'a> {
    fn advance(&mut self) {

    }
}