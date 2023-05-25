use crate::lexer::primary::PrimaryToken;

use self::{
    binary::BinaryExpression, call::CallExpression, literal::LiteralExpression,
    parentheses::ParenthesesExpression, unary::UnaryExpression, variable::VariableExpression,
};

pub(crate) mod binary;
pub(crate) mod call;
pub(crate) mod literal;
pub(crate) mod parentheses;
pub(crate) mod unary;
pub(crate) mod variable;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Call(CallExpression),
    Parentheses(ParenthesesExpression),
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

impl From<ParenthesesExpression> for Expression {
    fn from(parentheses: ParenthesesExpression) -> Self {
        Expression::Parentheses(parentheses)
    }
}

impl From<VariableExpression> for Expression {
    fn from(variable: VariableExpression) -> Self {
        Expression::Variable(variable)
    }
}

impl From<CallExpression> for Expression {
    fn from(call: CallExpression) -> Self {
        Expression::Call(call)
    }
}

#[allow(dead_code)]
impl Expression {
    pub fn is_unary(&self) -> bool {
        match self {
            Expression::Unary(_) => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Expression::Binary(_) => true,
            _ => false,
        }
    }

    pub fn is_parentheses(&self) -> bool {
        match self {
            Expression::Parentheses(_) => true,
            _ => false,
        }
    }
}
