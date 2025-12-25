use crate::ir::{
    ast::{
        common::literal::LiteralValue,
        global::{constant::ConstantDefinition, function::FunctionDefinition, GlobalStatement},
        local::{
            instruction::{call::CallInstruction, InstructionStatement},
            LocalStatement, LocalStatements,
        },
        CodeUnit,
    },
    IRCompiler,
};

pub mod ir;

pub fn main() {
    let compiler = IRCompiler::new();

    let code_unit = CodeUnit {
        filename: "example.foolang".into(),
        statements: vec![
            GlobalStatement::Constant(ConstantDefinition {
                constant_name: "HELLWORLD_TEXT".into(),
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

    let object = compiler.compile(code_unit);

    println!("This is a placeholder main function.");
}
