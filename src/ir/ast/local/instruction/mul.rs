use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct MulInstruction {
    pub left: Operand,
    pub right: Operand,
}
