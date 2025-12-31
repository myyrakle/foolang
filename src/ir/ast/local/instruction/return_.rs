use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct ReturnInstruction {
    pub return_value: Option<Operand>,
}
