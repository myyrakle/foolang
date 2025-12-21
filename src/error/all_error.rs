#![allow(dead_code)]

use crate::ir::error::IRError;

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
    #[error("IR error: {0}")]
    IRError(String),
}

impl From<ParserError> for AllError {
    fn from(error: ParserError) -> Self {
        Self::ParserError(error)
    }
}

impl From<IRError> for AllError {
    fn from(error: IRError) -> Self {
        Self::IRError(error.message)
    }
}
