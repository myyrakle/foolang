pub(crate) mod expression;

use crate::{ast::statement::Statement, error::Errors, ir::ast::CodeUnit};

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

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    pub fn generate(&mut self) -> Result<Vec<CodeUnit>, Errors> {
        unimplemented!("Code generation not yet implemented")
    }
}
