use self::{binary::BinaryOperator, unary::UnaryOperator};

mod binary;
mod unary;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}
