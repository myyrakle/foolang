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

impl From<LiteralValue> for AssignmentStatementValue {
    fn from(lit: LiteralValue) -> Self {
        AssignmentStatementValue::Literal(lit)
    }
}

impl From<InstructionStatement> for AssignmentStatementValue {
    fn from(instr: InstructionStatement) -> Self {
        AssignmentStatementValue::Instruction(instr)
    }
}
