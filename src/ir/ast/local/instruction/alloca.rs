use crate::ir::ast::{
    common::{Identifier, Operand},
    types::IRPrimitiveType,
};

#[derive(Debug)]
pub struct AllocaInstruction {
    pub _type: IRPrimitiveType,
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
