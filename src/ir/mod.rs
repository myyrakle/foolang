use crate::ir::data::object::{IRCompileObject, IRLinkObject};

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

    pub fn link(&self, _objects: Vec<IRCompileObject>) -> Result<IRLinkObject, error::IRError> {
        // Linking logic goes here
        unimplemented!()
    }
}
