use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct MulInstruction {
    pub left: Operand,
    pub right: Operand,
}
