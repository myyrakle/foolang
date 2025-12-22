use crate::ir::ast::{
    common::{literal::LiteralValue, Identifier},
    local::instruction::InstructionStatement,
};

#[derive(Debug)]
pub struct AssignmentStatement {
    pub name: Identifier,
    pub value: AssignmentStatementValue,
}

#[derive(Debug)]
pub enum AssignmentStatementValue {
    Literal(LiteralValue),
    Instruction(InstructionStatement),
}
