use crate::{
    ir::{
        ast::{global::GlobalStatement, CodeUnit},
        error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub mod branch;
pub mod call;
pub mod constant;
pub mod function;
pub mod return_;
pub mod statements;

pub fn compile(code_unit: CodeUnit) -> Result<ELFObject, IRError> {
    let mut compiled_object = ELFObject::new();

    // 함수 이름 목록을 수집하여 main 함수가 있는지 확인
    let mut has_main_function = false;

    for statement in &code_unit.statements {
        if let GlobalStatement::DefineFunction(function) = statement {
            if function.function_name == "main" {
                has_main_function = true;
                break;
            }
        }
    }

    // main 함수가 있으면 엔트리포인트로 설정
    if has_main_function {
        compiled_object.set_entry_point("main");
    }

    // 실제 컴파일
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
                    assignment::AssignmentStatement,
                    instruction::{
                        branch::{BranchInstruction, JumpInstruction},
                        call::CallInstruction,
                        return_::ReturnInstruction,
                        InstructionStatement,
                    },
                    label::LabelDefinition,
                    LocalStatement, LocalStatements,
                },
                types::IRPrimitiveType,
                CodeUnit,
            },
            error::IRError,
            IRCompiler,
        },
        platforms::{linux::elf::object::ELFOutputType, target::Target},
    };

    // 컴파일 후 링크해서 최종 실행
    // gcc output_with_libc.o -o output_linked.exe && ./output_linked.exe
    #[test]
    fn test_object_compile_with_gcc() {
        let compiler = IRCompiler::new();

        let object_filename = "object_compile_test.o";
        let executable_filename = "object_compile_test.exe";

        struct TestCase {
            name: &'static str,
            code_unit: CodeUnit,
            expected_output: &'static str,
            want_error: bool,
            expected_error: Option<IRError>,
        }

        let test_cases = vec![
            TestCase {
                name: "간단한 Hello World 출력",
                expected_output: "Hello, world!\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![LocalStatement::Instruction(
                                InstructionStatement::Call(CallInstruction {
                                    function_name: "puts".into(),
                                    parameters: vec![crate::ir::ast::common::Operand::Literal(
                                        LiteralValue::String("Hello, world!".into()),
                                    )],
                                }),
                            )],
                        },
                    })],
                },
            },
            TestCase {
                name: "간단한 Hello World 출력 (함수 호출 포함)",
                expected_output: "Hello, world!\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "HELLOWORLD_TEXT".into(),
                            value: LiteralValue::String("Hello, world!".into()),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_text".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![InstructionStatement::Return(ReturnInstruction {
                                    return_value: Some(
                                        crate::ir::ast::common::Operand::Identifier(
                                            "HELLOWORLD_TEXT".into(),
                                        ),
                                    ),
                                })
                                .into()],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    AssignmentStatement {
                                        name: "text".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_text".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "text".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "무조건 분기 테스트",
                expected_output: "SUCCEEDED!\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "FAILED_TEXT".into(),
                            value: LiteralValue::String("FAILED!".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "SUCCEEDED_TEXT".into(),
                            value: LiteralValue::String("SUCCEEDED!".into()),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(JumpInstruction{
                                        label: "jump_point".into(),
                                    }.into()),
                                     LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "FAILED_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                   LocalStatement::Label(LabelDefinition{
                                        name: "jump_point".into(),
                                   }),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "SUCCEEDED_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "TRUE 분기 테스트",
                expected_output: "TRUE!\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "FALSE_TEXT".into(),
                            value: LiteralValue::String("FALSE!".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "TRUE_TEXT".into(),
                            value: LiteralValue::String("TRUE!".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "FLAG".into(),
                            value: LiteralValue::Int64(1),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(BranchInstruction{
                                        condition: "FLAG".into(),
                                        true_label: "true_point".into(),
                                        false_label: "false_point".into(),
                                    }.into()),
                                     LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "FALSE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                    LocalStatement::Label(LabelDefinition{
                                            name: "true_point".into(),
                                    }),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "TRUE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                    LocalStatement::Instruction(InstructionStatement::Return(ReturnInstruction {
                                        return_value: None,
                                    })),
                                    LocalStatement::Label(LabelDefinition{
                                        name: "false_point".into(),
                                    }),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "FALSE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "FALSE 분기 테스트",
                expected_output: "FALSE!\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "FALSE_TEXT".into(),
                            value: LiteralValue::String("FALSE!".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "TRUE_TEXT".into(),
                            value: LiteralValue::String("TRUE!".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "FLAG".into(),
                            value: LiteralValue::Int64(0),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(BranchInstruction{
                                        condition: "FLAG".into(),
                                        true_label: "true_point".into(),
                                        false_label: "false_point".into(),
                                    }.into()),
                                     LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "TRUE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                    LocalStatement::Label(LabelDefinition{
                                            name: "true_point".into(),
                                    }),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "TRUE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                    LocalStatement::Instruction(InstructionStatement::Return(ReturnInstruction {
                                        return_value: None,
                                    })),
                                    LocalStatement::Label(LabelDefinition{
                                        name: "false_point".into(),
                                    }),
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier(
                                                    "FALSE_TEXT".into(),
                                                ),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
        ];

        let target = Target::LinuxAmd64;

        for test_case in test_cases {
            let object = compiler.compile(&target, test_case.code_unit);

            if test_case.want_error {
                assert!(
                    object.is_err(),
                    "Test case '{}' expected an error but got success",
                    test_case.name
                );
                if let Some(expected_err) = test_case.expected_error {
                    assert_eq!(
                        object.err().unwrap().to_string(),
                        expected_err.to_string(),
                        "Test case '{}' error mismatch",
                        test_case.name
                    );
                }

                continue;
            }

            let object = object.expect(&format!(
                "Test case '{}' compilation failed unexpectedly",
                test_case.name
            ));

            let encoded_object = match object {
                crate::ir::data::IRCompiledObject::ELF(elf_obj) => {
                    elf_obj.encode(ELFOutputType::Relocatable)
                }
            };

            std::fs::write(object_filename, encoded_object).expect("Failed to write object file");

            // gcc로 링크
            std::process::Command::new("gcc")
                .args(&[object_filename, "-o", executable_filename])
                .status()
                .expect("Failed to link with gcc");

            let output = std::process::Command::new(format!("./{}", executable_filename))
                .output()
                .expect("Failed to execute");

            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Program output: {}", stdout);

            assert_eq!(
                stdout, test_case.expected_output,
                "Test case '{}' output mismatch",
                test_case.name
            );
        }
    }
}
