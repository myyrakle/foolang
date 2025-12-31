use crate::ir::ast::common::{Identifier, Label};

#[derive(Debug)]
pub struct BranchInstruction {
    pub condition: Identifier,
    pub true_label: Label,
    pub false_label: Label,
}

#[derive(Debug)]
pub struct JumpInstruction {
    pub label: Label,
}
