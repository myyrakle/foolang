pub(crate) mod expression;

use crate::{ast::statement::Statement, error::all_error::AllError};

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
    pub fn generate(&mut self) -> Result<Vec<String>, AllError> {
        let mut codes = vec![];

        for statement in self.statements.clone().into_iter() {
            match statement {
                Statement::Expression(expression) => {
                    let mut result = self.generate_expression(expression.to_owned())?;
                    codes.append(&mut result);
                }
                Statement::DefineVariable(_variable_declaration) => {
                    unimplemented!();
                }
                Statement::DefineFunction(_function_declaration) => {
                    unimplemented!();
                }
                Statement::Return(_return_statement) => {
                    unimplemented!();
                }
                _ => {
                    unimplemented!();
                }
            }
        }

        Ok(codes)
    }
}
