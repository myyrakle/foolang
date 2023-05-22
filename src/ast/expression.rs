use crate::lexer::primary::PrimaryToken;

use self::{
    binary::BinaryExpression, call::CallExpression, literal::LiteralExpression,
    unary::UnaryExpression, variable::VariableExpression,
};

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

impl From<BinaryExpression> for Expression {
    fn from(binary: BinaryExpression) -> Self {
        Expression::Binary(binary)
    }
}

impl From<UnaryExpression> for Expression {
    fn from(unary: UnaryExpression) -> Self {
        Expression::Unary(unary)
    }
}

impl Expression {
    pub fn is_unary(&self) -> bool {
        match self {
            Expression::Unary(_) => true,
            _ => false,
        }
    }
}
