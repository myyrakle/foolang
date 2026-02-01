use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct SubInstruction {
    pub left: Operand,
    pub right: Operand,
}
