use crate::lexer::primary::PrimaryToken;

use self::{
    binary::BinaryExpression, call::CallExpression, literal::LiteralExpression,
    unary::UnaryExpression, variable::VariableExpression,
};

use super::statement::Statement;

pub(crate) mod binary;
pub(crate) mod call;
pub(crate) mod literal;
pub(crate) mod unary;
pub(crate) mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Call(CallExpression),
    Comment(String),
}

impl From<LiteralExpression> for Expression {
    fn from(literal: LiteralExpression) -> Self {
        Expression::Literal(literal)
    }
}

impl From<PrimaryToken> for Expression {
    fn from(token: PrimaryToken) -> Self {
        Expression::Literal(token.into())
    }
}
