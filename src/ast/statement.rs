use self::{
    define_function::FunctionDefinitionStatement, define_variable::VariableDefinitionStatement,
};

use super::expression::Expression;
pub mod define_function;
pub mod define_variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    DefineVariable(VariableDefinitionStatement),
    DefineFunction(FunctionDefinitionStatement),
    Return(Expression),
}

impl From<Expression> for Statement {
    fn from(expression: Expression) -> Self {
        Statement::Expression(expression)
    }
}
