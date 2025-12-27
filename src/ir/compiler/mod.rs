use crate::{
    ir::{ast::CodeUnit, data::IRCompiledObject, error::IRError, IRCompiler},
    platforms::target::Target,
};

pub mod amd64_linux;

impl IRCompiler {
    pub fn compile(
        &self,
        target: &Target,
        code_unit: CodeUnit,
    ) -> Result<IRCompiledObject, IRError> {
        match target {
            Target::Amd64Linux => {
                let compiled_object = amd64_linux::compile(code_unit)?;
                let compiled_object = IRCompiledObject::ELF(compiled_object);

                Ok(compiled_object)
            }
            _ => {
                unimplemented!("Compilation for target {:?} is not implemented yet", target)
            }
        }
    }
}
