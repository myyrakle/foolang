use crate::ir::ast::common::Operand;

#[derive(Debug)]
pub struct CallInstruction {
    pub function_name: String,
    pub parameters: Vec<Operand>,
}
