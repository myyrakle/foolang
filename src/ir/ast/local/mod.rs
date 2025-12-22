use crate::ir::ast::local::{
    assignment::AssignmentStatement, instruction::InstructionStatement, label::LabelDefinition,
};

pub mod assignment;
pub mod instruction;
pub mod label;

#[derive(Debug)]
pub struct LocalStatements {
    pub statements: Vec<LocalStatement>,
}

#[derive(Debug)]
pub enum LocalStatement {
    Assignment(AssignmentStatement),
    Instruction(InstructionStatement),
    Label(LabelDefinition),
}
