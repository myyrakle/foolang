use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct CompareInstruction {
    pub left: Operand,
    pub right: Operand,
}
