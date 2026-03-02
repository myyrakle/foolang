use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct AddInstruction {
    pub left: Operand,
    pub right: Operand,
}
