#![allow(dead_code)]

use crate::ir::error::IRError;

use self::parser_error::ParserError;

pub mod parser_error;

#[derive(Debug)]
pub enum Errors {
    LexerError(String),
    ParserError(ParserError),
    CodegenError(String),
    IOError(String),
    FileNotFound(String),
    IRError(IRError),
}

impl From<ParserError> for Errors {
    fn from(error: ParserError) -> Self {
        Self::ParserError(error)
    }
}

impl From<IRError> for Errors {
    fn from(error: IRError) -> Self {
        Self::IRError(error)
    }
}
