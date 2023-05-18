use crate::ast::expression::Expression;

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinitionStatement {
    pub name: String,
    pub parameters: Vec<String>, // TODO: add type
    // pub return_type: Type,
    pub body: Vec<Statement>,
}
