use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct SubInstruction {
    pub left: Operand,
    pub right: Operand,
}
