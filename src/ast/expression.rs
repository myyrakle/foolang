use self::{
    binary::BinaryExpression, call::CallExpression, literal::LiteralExpression,
    unary::UnaryExpression, variable::VariableExpression,
};

use super::statement::Statement;

mod binary;
mod call;
mod literal;
mod unary;
mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Call(CallExpression),
}

impl From<Expression> for Statement {
    fn from(expression: Expression) -> Self {
        Statement::Expression(expression)
    }
}
