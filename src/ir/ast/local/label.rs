use crate::ir::ast::common::Identifier;

#[derive(Debug, Clone)]
pub struct LabelDefinition {
    pub name: Identifier,
}
