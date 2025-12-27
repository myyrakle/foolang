use crate::{
    ir::{
        ast::{global::GlobalStatement, CodeUnit},
        error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub mod constant;

pub fn compile(code_unit: CodeUnit) -> Result<ELFObject, IRError> {
    let mut compiled_object = ELFObject::new();

    for statement in code_unit.statements {
        match statement {
            GlobalStatement::Constant(constant) => {
                constant::compile_constant(&constant, &mut compiled_object)?;
            }
            GlobalStatement::DefineFunction(function) => {
                // Compile function definition
                // Placeholder for actual compilation logic
            }
        }
    }

    Ok(compiled_object)
}
