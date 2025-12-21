use crate::ir::ast::CodeUnit;

pub mod ast;
pub mod error;

#[derive(Debug)]
pub struct IRCompiler {}

#[derive(Debug)]
pub struct IRCompileObject {}

#[derive(Debug)]
pub struct IRLinkObject {}

impl IRCompiler {
    pub fn new() -> Self {
        IRCompiler {}
    }

    pub fn compile(&self, _code_unit: CodeUnit) -> Result<IRCompileObject, error::IRError> {
        // Compilation logic goes here
        unimplemented!()
    }

    pub fn link(&self, _objects: Vec<IRCompileObject>) -> Result<IRLinkObject, error::IRError> {
        // Linking logic goes here
        unimplemented!()
    }
}
