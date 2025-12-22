use crate::ir::ast::local::instruction::{add::AddInstruction, call::CallInstruction};

pub mod add;
pub mod call;

#[derive(Debug)]
pub enum InstructionStatement {
    Call(CallInstruction),
    Add(AddInstruction),
}
