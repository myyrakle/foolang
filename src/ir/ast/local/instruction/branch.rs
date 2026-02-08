use crate::ir::ast::common::{Label, Operand};

#[derive(Debug, Clone)]
pub struct BranchInstruction {
    pub condition: Operand,
    pub true_label: Label,
    pub false_label: Label,
}

#[derive(Debug, Clone)]
pub struct JumpInstruction {
    pub label: Label,
}
