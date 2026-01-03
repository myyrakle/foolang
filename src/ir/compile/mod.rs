use crate::{
    ir::{ast::CodeUnit, data::IRCompiledObject, error::IRError, IRCompiler},
    platforms::target::Target,
};

pub mod linux_amd64;

impl IRCompiler {
    pub fn compile(
        &self,
        target: &Target,
        code_unit: CodeUnit,
    ) -> Result<IRCompiledObject, IRError> {
        match target {
            Target::LinuxAmd64 => {
                let compiled_object = linux_amd64::compile(code_unit)?;
                let compiled_object = IRCompiledObject::ELF(compiled_object);

                Ok(compiled_object)
            }
            _ => {
                unimplemented!("Compilation for target {:?} is not implemented yet", target)
            }
        }
    }
}
