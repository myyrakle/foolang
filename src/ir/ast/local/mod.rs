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

impl From<AssignmentStatement> for LocalStatement {
    fn from(stmt: AssignmentStatement) -> Self {
        LocalStatement::Assignment(stmt)
    }
}

impl From<InstructionStatement> for LocalStatement {
    fn from(stmt: InstructionStatement) -> Self {
        LocalStatement::Instruction(stmt)
    }
}

impl From<LabelDefinition> for LocalStatement {
    fn from(label: LabelDefinition) -> Self {
        LocalStatement::Label(label)
    }
}
