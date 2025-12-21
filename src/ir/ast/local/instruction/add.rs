use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct AddInstruction {
    pub left: Operand,
    pub right: Operand,
}
