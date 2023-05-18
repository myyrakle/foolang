use super::{super::operator::unary::UnaryOperator, Expression};

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    pub operator: UnaryOperator,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}
