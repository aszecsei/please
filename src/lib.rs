//! Please, the polite task runner

#![deny(missing_docs)]

mod error;
mod lexer;
mod parser;
mod run;

pub use run::run;