use self::{call::CallExpression, variable::VariableExpression};

mod call;
mod variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Call(CallExpression),
}
