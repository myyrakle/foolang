#![allow(dead_code)]

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AllError {
    #[error("Lexer error: {0}")]
    LexerError(String),
    #[error("Parser error: {0}")]
    ParserError(String),
    #[error("Codegen error: {0}")]
    CodegenError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("FileNotFound error: {0}")]
    FileNotFound(String),
}
