use super::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
}
