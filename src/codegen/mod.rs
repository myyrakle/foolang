use crate::ast::statement::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct CodeGenerator {
    statements: Vec<Statement>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self { statements: vec![] }
    }

    pub fn set_statements(&mut self, statements: Vec<Statement>) {
        self.statements = statements;
    }
}
