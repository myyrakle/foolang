use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub function_name: String,
    pub arguments: Vec<Expression>,
}
