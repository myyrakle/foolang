use crate::ir::ast::common::{Identifier, Operand};

#[derive(Debug, Clone)]
pub struct CallInstruction {
    pub function_name: Identifier,
    pub parameters: Vec<Operand>,
}
