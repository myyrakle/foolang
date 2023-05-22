use super::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct ParenthesesExpression {
    pub expression: Box<Expression>,
}
