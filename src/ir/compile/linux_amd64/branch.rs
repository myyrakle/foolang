use crate::{
    ir::{
        ast::local::{instruction::branch::JumpInstruction, label::LabelDefinition},
        compile::linux_amd64::function::FunctionContext,
    },
    platforms::linux::elf::object::ELFObject,
};

pub fn compile_label_definition(
    label_definition: &LabelDefinition,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), crate::ir::error::IRError> {
    unimplemented!()
}

pub fn compile_jump_instruction(
    instruction: &JumpInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), crate::ir::error::IRError> {
    unimplemented!()
}
