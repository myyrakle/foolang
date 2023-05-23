#![allow(dead_code)]

use self::{binary::BinaryOperator, unary::UnaryOperator};

pub mod binary;
pub mod unary;

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}

impl Operator {
    pub fn is_unary(&self) -> bool {
        match self {
            Operator::Unary(_) => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Operator::Binary(_) => true,
            _ => false,
        }
    }
}
