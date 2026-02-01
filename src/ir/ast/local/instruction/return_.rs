use crate::ir::ast::common::Operand;

#[derive(Debug, Clone)]
pub struct ReturnInstruction {
    pub return_value: Option<Operand>,
}
