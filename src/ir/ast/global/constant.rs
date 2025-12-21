use crate::ir::ast::common::{literal::LiteralValue, Identifier};

#[derive(Debug)]
pub struct ConstantDefinition {
    pub constant_name: Identifier,
    pub value: LiteralValue,
}
