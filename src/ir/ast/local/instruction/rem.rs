use crate::ir::ast::common::Operand;

/// REM 인스트럭션
/// 나머지셈 연산을 수행합니다.
#[derive(Debug)]
pub struct RemInstruction {
    pub left: Operand,
    pub right: Operand,
}
