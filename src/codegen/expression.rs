use crate::{ast::expression::Expression, error::all_error::AllError};

use super::CodeGenerator;

impl CodeGenerator {
    pub fn generate_expression(&mut self, expression: Expression) -> Result<Vec<String>, AllError> {
        let _codes = vec![];

        match expression {
            Expression::Call(_call_expression) => {
                unimplemented!();
            }
            Expression::Literal(_literal_expression) => {
                unimplemented!();
            }
            Expression::Variable(_variable_expression) => {
                unimplemented!();
            }
            Expression::Binary(_binary_expression) => {
                unimplemented!();
            }
            Expression::Unary(_unary_expression) => {
                unimplemented!();
            }
            _ => {
                unimplemented!();
            }
        }

        Ok(_codes)
    }
}
