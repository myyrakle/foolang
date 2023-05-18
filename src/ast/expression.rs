use self::{
    call::CallExpression, literal::LiteralExpression, unary::UnaryExpression,
    variable::VariableExpression,
};

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
