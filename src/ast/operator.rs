use self::{binary::BinaryOperator, unary::UnaryOperator};

pub mod binary;
pub mod unary;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}
