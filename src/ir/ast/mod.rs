use crate::ir::ast::global::GlobalStatement;

pub mod common;
pub mod global;
pub mod local;
pub mod types;

#[derive(Debug)]
pub struct CodeUnit {
    pub filename: String,
    pub statements: Vec<GlobalStatement>,
}
