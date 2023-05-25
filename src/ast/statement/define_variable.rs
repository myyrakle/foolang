use crate::ast::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinitionStatement {
    pub mutable: bool,
    pub name: String,
    pub value: Option<Expression>,
    // pub type: Type,
}
