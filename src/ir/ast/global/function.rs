use crate::ir::ast::{local::LocalStatements, types::IRType};

#[derive(Debug)]
pub struct FunctionDefinition {
    pub function_name: String,
    pub return_type: IRType,
    pub arguments: Vec<FunctionArgument>,
    pub function_body: LocalStatements,
}

#[derive(Debug)]
pub struct FunctionArgument {
    pub argument_name: String,
    pub argument_type: String,
}
