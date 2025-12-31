use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct DivInstruction {
    pub left: Operand,
    pub right: Operand,
}
