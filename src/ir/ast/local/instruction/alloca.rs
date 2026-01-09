use crate::ir::ast::{
    common::{Identifier, Operand},
    types::IRPrimitiveType,
};

/// Stack Allocation Instruction (memory allocation on the stack)
/// return pointer to the allocated memory
#[derive(Debug)]
pub struct AllocaInstruction {
    pub type_: IRPrimitiveType,
}

#[derive(Debug)]
pub struct LoadInstruction {
    pub ptr: Identifier,
}

#[derive(Debug)]
pub struct StoreInstruction {
    pub ptr: Identifier,
    pub value: Operand,
}
