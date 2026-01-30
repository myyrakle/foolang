use crate::ir::ast::types::IRType;

pub mod literal;

#[derive(Debug)]
pub struct Identifier {
    pub type_: IRType,
    pub name: String,
}

impl Identifier {
    /// 변수명을 FunctionContext에서 현재 SSA 값으로 해석
    ///
    /// Phase 7: SSA 값 조회를 위한 헬퍼 메서드
    pub fn resolve_to_ssa(
        &self,
        context: &crate::ir::compile::linux_amd64::function::FunctionContext,
    ) -> Result<crate::ir::ssa::SSAValueId, crate::ir::error::IRError> {
        context
            .get_current_version(&self.name)
            .ok_or_else(|| crate::ir::error::IRError {
                kind: crate::ir::error::IRErrorKind::VariableNotFound,
                message: format!("Variable '{}' not found in SSA context", self.name),
            })
    }
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_: IRType::None,
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Identifier(Identifier),
    Literal(literal::LiteralValue),
    /// SSA 값 직접 참조 (Phase 7에서 추가)
    SSAValue(crate::ir::ssa::SSAValueId),
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
}

impl From<&str> for Label {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
