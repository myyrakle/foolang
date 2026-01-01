use crate::{
    ir::error::IRError,
    platforms::{
        amd64::{instruction::Instruction, register::Register, rex::RexPrefix},
        linux::elf::{
            object::ELFObject,
            relocation::{Relocation, RelocationType},
            section::SectionType,
        },
    },
};

pub fn compile_return_instruction(
    instruction: &crate::ir::ast::local::instruction::return_::ReturnInstruction,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    Ok(())
}
