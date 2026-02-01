use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct CompareInstruction {
    pub left: Operand,
    pub right: Operand,
}
