use crate::ir::data::{IRCompiledObject, IRLinkedObject};

pub mod ast;
pub mod compiler;
pub mod data;
pub mod error;

#[derive(Debug)]
pub struct IRCompiler {}

impl IRCompiler {
    pub fn new() -> Self {
        IRCompiler {}
    }

    pub fn link(&self, _objects: Vec<IRCompiledObject>) -> Result<IRLinkedObject, error::IRError> {
        // Linking logic goes here
        unimplemented!()
    }
}
