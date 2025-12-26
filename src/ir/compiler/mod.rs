use crate::ir::{
    ast::{global::GlobalStatement, CodeUnit},
    data::object::IRCompileObject,
    error::IRError,
    IRCompiler,
};

pub mod constant;

impl IRCompiler {
    pub fn compile(&self, code_unit: CodeUnit) -> Result<IRCompileObject, IRError> {
        let mut compiled_object = IRCompileObject::new();

        for statement in code_unit.statements {
            match statement {
                GlobalStatement::Constant(constant) => {
                    constant::compile_constant_definition(&constant, &mut compiled_object)?;
                }
                GlobalStatement::DefineFunction(function) => {
                    // Compile function definition
                    // Placeholder for actual compilation logic
                }
            }
        }

        Ok(compiled_object)
    }
}
