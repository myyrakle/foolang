use crate::ir::{
    ast::{global::GlobalStatement, CodeUnit},
    data::object::{IRCompileObject, IRLinkObject},
};

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

    pub fn compile(&self, code_unit: CodeUnit) -> Result<IRCompileObject, error::IRError> {
        let mut compiled_object = IRCompileObject::new();

        for statement in code_unit.statements {
            match statement {
                GlobalStatement::Constant(constant) => {
                    compiler::constant::compile_constant_definition(
                        &constant,
                        &mut compiled_object,
                    )?;
                }
                GlobalStatement::DefineFunction(function) => {
                    // Compile function definition
                    // Placeholder for actual compilation logic
                }
            }
        }

        Ok(compiled_object)
    }

    pub fn link(&self, _objects: Vec<IRCompileObject>) -> Result<IRLinkObject, error::IRError> {
        // Linking logic goes here
        unimplemented!()
    }
}
