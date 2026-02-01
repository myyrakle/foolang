use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct DivInstruction {
    pub left: Operand,
    pub right: Operand,
}
