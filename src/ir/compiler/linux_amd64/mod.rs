use crate::{
    ir::{
        ast::{global::GlobalStatement, CodeUnit},
        error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub mod constant;
pub mod function;
pub mod instruction;

pub fn compile(code_unit: CodeUnit) -> Result<ELFObject, IRError> {
    let mut compiled_object = ELFObject::new();

    for statement in code_unit.statements {
        match statement {
            GlobalStatement::Constant(constant) => {
                constant::compile_constant(&constant, &mut compiled_object)?;
            }
            GlobalStatement::DefineFunction(function) => {
                function::compile_function(&function, &mut compiled_object)?;
            }
        }
    }

    Ok(compiled_object)
}

#[cfg(test)]
mod tests {
    use crate::{
        ir::{
            ast::{
                common::literal::LiteralValue,
                global::{
                    constant::ConstantDefinition, function::FunctionDefinition, GlobalStatement,
                },
                local::{
                    instruction::{call::CallInstruction, InstructionStatement},
                    LocalStatement, LocalStatements,
                },
                CodeUnit,
            },
            IRCompiler,
        },
        platforms::{linux::elf::object::ELFOutputType, target::Target},
    };

    #[test]
    fn test_compile() {
        let compiler = IRCompiler::new();

        let code_unit = CodeUnit {
            filename: "example.foolang".into(),
            statements: vec![
                GlobalStatement::Constant(ConstantDefinition {
                    constant_name: "HELLOWORLD_TEXT".into(),
                    value: LiteralValue::String("Hello, world!".into()),
                }),
                GlobalStatement::DefineFunction(FunctionDefinition {
                    function_name: "main".into(),
                    arguments: vec![],
                    return_type: None,
                    function_body: LocalStatements {
                        statements: vec![LocalStatement::Instruction(InstructionStatement::Call(
                            CallInstruction {
                                function_name: "printf".into(),
                                parameters: vec![],
                            },
                        ))],
                    },
                }),
            ],
        };

        let target = Target::LinuxAmd64;

        let object = compiler.compile(&target, code_unit);

        std::fs::write(
            "output.exe",
            match object {
                Ok(obj) => match obj {
                    crate::ir::data::IRCompiledObject::ELF(elf_obj) => {
                        elf_obj.encode(ELFOutputType::Executable)
                    }
                },
                Err(_) => vec![],
            },
        )
        .expect("Failed to write output.exe");

        println!("This is a placeholder main function.");
    }
}
