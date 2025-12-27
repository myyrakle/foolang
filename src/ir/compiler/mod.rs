use crate::ir::{ast::CodeUnit, data::IRCompiledObject, error::IRError, IRCompiler};

pub mod amd64_linux;

impl IRCompiler {
    pub fn compile(&self, code_unit: CodeUnit) -> Result<IRCompiledObject, IRError> {
        let compiled_object = amd64_linux::compile(code_unit)?;

        // TODO: 플랫폼별 분기 처리
        let compiled_object = IRCompiledObject::ELF(compiled_object);

        Ok(compiled_object)
    }
}
