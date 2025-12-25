use crate::ir::ast::global::{constant::ConstantDefinition, function::FunctionDefinition};

pub mod constant;
pub mod function;

#[derive(Debug)]
pub struct GlobalStatements {
    pub statements: Vec<GlobalStatement>,
}

#[derive(Debug)]
pub enum GlobalStatement {
    DefineFunction(FunctionDefinition),
    Constant(ConstantDefinition),
}
