use crate::{
    ir::data::{IRCompiledObject, IRLinkedObject},
    platforms::target::Target,
};

pub mod ast;
pub mod compile;
pub mod data;
pub mod error;
pub mod ssa;

#[derive(Debug)]
pub struct IRCompiler {}

impl IRCompiler {
    pub fn new() -> Self {
        IRCompiler {}
    }

    pub fn link(
        &self,
        _target: &Target,
        _objects: Vec<IRCompiledObject>,
    ) -> Result<IRLinkedObject, error::IRError> {
        // Linking logic goes here
        unimplemented!()
    }
}
