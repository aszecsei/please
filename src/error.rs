use failure::Fail;
use std::fmt;

#[derive(Debug, Fail)]
pub enum CompilationErrorKind {
    #[fail(display = "expected '{}' but got '{}'", expected, got)]
    ExpectedButGot {
        got: char,
        expected: char,
    },
    #[fail(display = "unexpected character '{}'", ch)]
    UnexpectedChar {
        ch: char,
    },
    #[fail(display = "internal error: '{}'", message)]
    Internal {
        message: String,
    },
}

#[derive(Debug, Fail)]
#[fail(display = "{} at {}:{}:{}", kind, filename, line, column)]
pub struct CompilationError {
    pub line: usize,
    pub column: usize,
    pub filename: String,
    pub kind: CompilationErrorKind,
}

impl CompilationError {
    pub fn new(kind: CompilationErrorKind, filename: String, line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            filename,
            kind,
        }
    }
}

#[derive(Debug, Fail)]
pub struct MultipleErrors {
    pub errs: Vec<failure::Error>,
}

impl fmt::Display for MultipleErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.errs.len() > 1 {
            writeln!(f, "multiple errors:")?;
        }
        if self.errs.len() > 5 {
            for err_idx in 0..5 {
                writeln!(f, "\t{}", self.errs[err_idx])?;
            }
            writeln!(f, "\t{} other errors omitted.", self.errs.len() - 5)?;
        } else {
            for err in self.errs.iter() {
                writeln!(f, "\t{}", err)?;
            }
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, failure::Error>;