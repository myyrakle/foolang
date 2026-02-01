use crate::ir::ast::{
    common::{Identifier, Operand},
    types::IRPrimitiveType,
};

/// Stack Allocation Instruction (memory allocation on the stack)
/// return pointer to the allocated memory
#[derive(Debug, Clone)]
pub struct AllocaInstruction {
    pub type_: IRPrimitiveType,
}

#[derive(Debug, Clone)]
pub struct LoadInstruction {
    pub ptr: Identifier,
}

#[derive(Debug, Clone)]
pub struct StoreInstruction {
    pub ptr: Identifier,
    pub value: Operand,
}
