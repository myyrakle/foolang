use crate::ir::ast::common::{Identifier, Operand};

#[derive(Debug)]
pub struct CallInstruction {
    pub function_name: Identifier,
    pub parameters: Vec<Operand>,
}
