#![allow(dead_code)]

use self::parser_error::ParserError;

pub mod parser_error;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AllError {
    #[error("Lexer error: {0}")]
    LexerError(String),
    #[error("Parser error: {0}")]
    ParserError(ParserError),
    #[error("Codegen error: {0}")]
    CodegenError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("FileNotFound error: {0}")]
    FileNotFound(String),
}

impl From<ParserError> for AllError {
    fn from(error: ParserError) -> Self {
        Self::ParserError(error)
    }
}
