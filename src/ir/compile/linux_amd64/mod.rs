use crate::{
    ir::{
        ast::{global::GlobalStatement, CodeUnit},
        error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub mod add;
pub mod alloca;
pub mod branch;
pub mod call;
pub mod common;
pub mod compare;
pub mod constant;
pub mod div;
pub mod function;
pub mod mul;
pub mod rem;
pub mod return_;
pub mod statements;
pub mod sub;

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

    // 1단계: 모든 함수 이름을 미리 수집 (forward reference 해결용)
    // 함수 body 컴파일 전에 모든 함수 이름을 알아두면,
    // 함수 호출 시 로컬 함수인지 외부 함수인지 판단 가능
    use std::collections::HashSet;
    let mut defined_functions: HashSet<String> = HashSet::new();
    for statement in &code_unit.statements {
        if let GlobalStatement::DefineFunction(function) = statement {
            defined_functions.insert(function.function_name.clone());
        }
    }

    // 함수 이름 목록을 object에 저장 (call.rs에서 사용)
    compiled_object.defined_functions = defined_functions;

    // 2단계: 실제 컴파일
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
                        add::AddInstruction,
                        alloca::AllocaInstruction,
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
            IRCompiler,
        },
        platforms::{linux::elf::object::ELFOutputType, target::Target},
    };

    // 컴파일 후 링크해서 최종 실행
    // gcc output_with_libc.o -o output_linked.exe && ./output_linked.exe
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[test]
    fn test_object_compile_with_gcc() {
        use crate::ir::error::IRErrorKind;

        let compiler = IRCompiler::new();

        let object_filename = "object_compile_test.o";
        let executable_filename = "object_compile_test.exe";

        struct TestCase {
            name: &'static str,
            code_unit: CodeUnit,
            expected_output: &'static str,
            want_error: bool,
            expected_error: Option<IRErrorKind>,
        }

        let success_cases = vec![
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
                name: "alloca 명령어 테스트 (Int64 할당)",
                expected_output: "Alloca test\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // ptr = alloca i64 (스택에 8바이트 할당)
                                AssignmentStatement {
                                    name: "ptr".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int64,
                                        }),
                                    ),
                                }.into(),
                                // puts 호출
                                LocalStatement::Instruction(
                                    InstructionStatement::Call(CallInstruction {
                                        function_name: "puts".into(),
                                        parameters: vec![crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::String("Alloca test".into()),
                                        )],
                                    }),
                                ),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "alloca 명령어 테스트 (여러 타입 할당)",
                expected_output: "Multiple alloca test\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // ptr1 = alloca i64
                                AssignmentStatement {
                                    name: "ptr1".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int64,
                                        }),
                                    ),
                                }.into(),
                                // ptr2 = alloca i32
                                AssignmentStatement {
                                    name: "ptr2".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int32,
                                        }),
                                    ),
                                }.into(),
                                // ptr3 = alloca i8
                                AssignmentStatement {
                                    name: "ptr3".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int8,
                                        }),
                                    ),
                                }.into(),
                                // puts 호출
                                LocalStatement::Instruction(
                                    InstructionStatement::Call(CallInstruction {
                                        function_name: "puts".into(),
                                        parameters: vec![crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::String("Multiple alloca test".into()),
                                        )],
                                    }),
                                ),
                            ],
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

        let error_cases = vec![
            TestCase {
                name: "Label 중복 정의 오류",
                expected_output: "",
                want_error: true,
                expected_error: Some(IRErrorKind::LabelAlreadyDefined),
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Label(LabelDefinition {
                                    name: "start".into(),
                                }),
                                LocalStatement::Label(LabelDefinition {
                                    name: "start".into(),
                                }),
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "puts".into(),
                                        parameters: vec![crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::String("Hello, world!".into()),
                                        )],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "존재하지 않는 Label 에 대한 점프",
                expected_output: "",
                want_error: true,
                expected_error: Some(IRErrorKind::LabelNotFound),
                code_unit: CodeUnit {
                    filename: "example.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Instruction(
                                    JumpInstruction {
                                        label: "undefined_label".into(),
                                    }
                                    .into(),
                                ),
                                LocalStatement::Label(LabelDefinition {
                                    name: "start".into(),
                                }),
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "puts".into(),
                                        parameters: vec![crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::String("Hello, world!".into()),
                                        )],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "존재하지 않는 분기에 대한 branch",
                expected_output: "",
                want_error: true,
                expected_error: Some(IRErrorKind::VariableNotFound),
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
                                    LocalStatement::Instruction(
                                        BranchInstruction {
                                            condition: "UNDEFINED_VAR".into(),
                                            true_label: "true_point".into(),
                                            false_label: "false_point".into(),
                                        }
                                        .into(),
                                    ),
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
                                    LocalStatement::Label(LabelDefinition {
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
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction { return_value: None },
                                    )),
                                    LocalStatement::Label(LabelDefinition {
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
                name: "load/store 명령어 테스트 - 기본 Int64",
                expected_output: "42\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "load_store_test.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // ptr = alloca i64
                                AssignmentStatement {
                                    name: "ptr".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int64,
                                        }),
                                    ),
                                }.into(),
                                // store 42, ptr
                                LocalStatement::Instruction(InstructionStatement::Store(
                                    crate::ir::ast::local::instruction::alloca::StoreInstruction {
                                        ptr: "ptr".into(),
                                        value: crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(42),
                                        ),
                                    },
                                )),
                                // value = load ptr
                                AssignmentStatement {
                                    name: "value".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Load(
                                            crate::ir::ast::local::instruction::alloca::LoadInstruction {
                                                ptr: "ptr".into(),
                                            },
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", value)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("value".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "load/store 명령어 테스트 - 여러 번 store/load",
                expected_output: "84\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "multiple_store_load.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // ptr = alloca i64
                                AssignmentStatement {
                                    name: "ptr".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Alloca(AllocaInstruction {
                                            type_: IRPrimitiveType::Int64,
                                        }),
                                    ),
                                }.into(),
                                // store 84, ptr
                                LocalStatement::Instruction(InstructionStatement::Store(
                                    crate::ir::ast::local::instruction::alloca::StoreInstruction {
                                        ptr: "ptr".into(),
                                        value: crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(84),
                                        ),
                                    },
                                )),
                                // value = load ptr
                                AssignmentStatement {
                                    name: "value".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Load(
                                            crate::ir::ast::local::instruction::alloca::LoadInstruction {
                                                ptr: "ptr".into(),
                                            },
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", value)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("value".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "ADD 명령어 테스트 - Int64 리터럴 덧셈",
                expected_output: "30\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "add_int64_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = add 10, 20
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Add(AddInstruction {
                                            left: crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(10),
                                            ),
                                            right: crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(20),
                                            ),
                                        }),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "ADD 명령어 테스트 - 리터럴과 변수 덧셈",
                expected_output: "30\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "add_literal_identifier.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // x = 10
                                AssignmentStatement {
                                    name: "x".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Call(CallInstruction {
                                            function_name: "get_ten".into(),
                                            parameters: vec![],
                                        }),
                                    ),
                                }.into(),
                                // result = add 20, x
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Add(AddInstruction {
                                            left: crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(20),
                                            ),
                                            right: crate::ir::ast::common::Operand::Identifier("x".into()),
                                        }),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    }),
                    GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "get_ten".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Int64.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Instruction(InstructionStatement::Return(
                                    ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(10),
                                        )),
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "ADD 명령어 테스트 - 변수와 변수 덧셈",
                expected_output: "30\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "add_identifier_identifier.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // x = 10
                                AssignmentStatement {
                                    name: "x".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Call(CallInstruction {
                                            function_name: "get_ten".into(),
                                            parameters: vec![],
                                        }),
                                    ),
                                }.into(),
                                // y = 20
                                AssignmentStatement {
                                    name: "y".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Call(CallInstruction {
                                            function_name: "get_twenty".into(),
                                            parameters: vec![],
                                        }),
                                    ),
                                }.into(),
                                // result = add x, y
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Add(AddInstruction {
                                            left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                            right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                        }),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    }),
                    GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "get_ten".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Int64.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Instruction(InstructionStatement::Return(
                                    ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(10),
                                        )),
                                    },
                                )),
                            ],
                        },
                    }),
                    GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "get_twenty".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Int64.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Instruction(InstructionStatement::Return(
                                    ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(20),
                                        )),
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "ADD 명령어 테스트 - 체이닝 연산",
                expected_output: "40\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "add_chained.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // x = 5
                                AssignmentStatement {
                                    name: "x".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Call(CallInstruction {
                                            function_name: "get_five".into(),
                                            parameters: vec![],
                                        }),
                                    ),
                                }.into(),
                                // y = add x, 10  (y = 15)
                                AssignmentStatement {
                                    name: "y".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Add(AddInstruction {
                                            left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                            right: crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(10),
                                            ),
                                        }),
                                    ),
                                }.into(),
                                // z = add y, 25  (z = 40)
                                AssignmentStatement {
                                    name: "z".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Add(AddInstruction {
                                            left: crate::ir::ast::common::Operand::Identifier("y".into()),
                                            right: crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(25),
                                            ),
                                        }),
                                    ),
                                }.into(),
                                // printf("%lld\n", z)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("z".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    }),
                    GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "get_five".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Int64.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                LocalStatement::Instruction(InstructionStatement::Return(
                                    ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(
                                            LiteralValue::Int64(5),
                                        )),
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "SUB 명령어 테스트 - Int64 리터럴 뺄셈",
                expected_output: "10\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "sub_int64_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = sub 30, 20
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Sub(
                                            crate::ir::ast::local::instruction::sub::SubInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(30),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(20),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "SUB 명령어 테스트 - 변수 간 뺄셈",
                expected_output: "15\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "sub_identifier_identifier.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 25
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_twenty_five".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = 10
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_ten".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // result = sub x, y
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Sub(
                                                crate::ir::ast::local::instruction::sub::SubInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_twenty_five".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(25),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_ten".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(10),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "SUB 명령어 테스트 - 체이닝 연산",
                expected_output: "10\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "sub_chained.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 50
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_fifty".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = sub x, 20  (y = 30)
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Sub(
                                                crate::ir::ast::local::instruction::sub::SubInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(20),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // result = sub y, 20  (result = 10)
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Sub(
                                                crate::ir::ast::local::instruction::sub::SubInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(20),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_fifty".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(50),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "SUB 명령어 테스트 - Int32 리터럴 뺄셈",
                expected_output: "5\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "sub_int32_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = sub 15, 10
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Sub(
                                            crate::ir::ast::local::instruction::sub::SubInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(15),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(10),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "MUL 명령어 테스트 - Int64 리터럴 곱셈",
                expected_output: "200\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "mul_int64_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = mul 10, 20
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Mul(
                                            crate::ir::ast::local::instruction::mul::MulInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(10),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(20),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "MUL 명령어 테스트 - 변수 간 곱셈",
                expected_output: "60\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "mul_identifier_identifier.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 6
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_six".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = 10
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_ten".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // result = mul x, y
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Mul(
                                                crate::ir::ast::local::instruction::mul::MulInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_six".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(6),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_ten".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(10),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "MUL 명령어 테스트 - 체이닝 곱셈",
                expected_output: "200\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "mul_chained.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 2
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_two".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = mul x, 5  // y = 10
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Mul(
                                                crate::ir::ast::local::instruction::mul::MulInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(5),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // result = mul y, 20  // result = 200
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Mul(
                                                crate::ir::ast::local::instruction::mul::MulInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(20),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_two".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(2),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "MUL 명령어 테스트 - Int32 리터럴 곱셈",
                expected_output: "50\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "mul_int32_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = mul 5, 10
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Mul(
                                            crate::ir::ast::local::instruction::mul::MulInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(5),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(10),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "DIV 명령어 테스트 - Int64 리터럴 나눗셈",
                expected_output: "5\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "div_int64_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = div 100, 20
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Div(
                                            crate::ir::ast::local::instruction::div::DivInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(100),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(20),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "DIV 명령어 테스트 - 변수 간 나눗셈",
                expected_output: "6\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "div_identifier_identifier.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 60
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_sixty".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = 10
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_ten".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // result = div x, y
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Div(
                                                crate::ir::ast::local::instruction::div::DivInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_sixty".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(60),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_ten".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(10),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "DIV 명령어 테스트 - 체이닝 나눗셈",
                expected_output: "5\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "div_chained.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 200
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_two_hundred".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = div x, 10  // y = 20
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Div(
                                                crate::ir::ast::local::instruction::div::DivInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(10),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // result = div y, 4  // result = 5
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Div(
                                                crate::ir::ast::local::instruction::div::DivInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(4),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_two_hundred".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        ReturnInstruction {
                                            return_value: Some(crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::Int64(200),
                                            )),
                                        },
                                    )),
                                ],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "DIV 명령어 테스트 - Int32 리터럴 나눗셈",
                expected_output: "3\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "div_int32_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = div 15, 5
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Div(
                                            crate::ir::ast::local::instruction::div::DivInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(15),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(5),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "REM 명령어 테스트 - Int64 리터럴 나머지",
                expected_output: "10\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "rem_int64_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = rem 100, 30
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Rem(
                                            crate::ir::ast::local::instruction::rem::RemInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(100),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(30),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "REM 명령어 테스트 - 변수 간 나머지",
                expected_output: "4\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "rem_identifier_identifier.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 60
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_sixty".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = 7
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_seven".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // result = rem x, y
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Rem(
                                                crate::ir::ast::local::instruction::rem::RemInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_sixty".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![LocalStatement::Instruction(
                                    InstructionStatement::Return(crate::ir::ast::local::instruction::return_::ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(LiteralValue::Int64(60))),
                                    }),
                                )],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_seven".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![LocalStatement::Instruction(
                                    InstructionStatement::Return(crate::ir::ast::local::instruction::return_::ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(LiteralValue::Int64(7))),
                                    }),
                                )],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "REM 명령어 테스트 - 체이닝 나머지",
                expected_output: "6\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "rem_chaining.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // temp = rem 200, 60
                                    AssignmentStatement {
                                        name: "temp".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Rem(
                                                crate::ir::ast::local::instruction::rem::RemInstruction {
                                                    left: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(200),
                                                    ),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(60),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // result = rem temp, 7
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Rem(
                                                crate::ir::ast::local::instruction::rem::RemInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("temp".into()),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(7),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
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
                name: "REM 명령어 테스트 - Int32 리터럴 나머지",
                expected_output: "2\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "rem_int32_literal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = rem 17, 5
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Rem(
                                            crate::ir::ast::local::instruction::rem::RemInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(17),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(5),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "COMPARE 명령어 테스트 - 같은 값 비교",
                expected_output: "1\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_equal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = compare 10, 10
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Compare(
                                            crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(10),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(10),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "COMPARE 명령어 테스트 - 다른 값 비교",
                expected_output: "0\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_not_equal.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = compare 10, 20
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Compare(
                                            crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(10),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int64(20),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "COMPARE 명령어 테스트 - 변수 간 비교",
                expected_output: "1\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_identifier_identifier.foolang".into(),
                    statements: vec![
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // x = 60
                                    AssignmentStatement {
                                        name: "x".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_sixty".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // y = 60
                                    AssignmentStatement {
                                        name: "y".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Call(CallInstruction {
                                                function_name: "get_sixty".into(),
                                                parameters: vec![],
                                            }),
                                        ),
                                    }.into(),
                                    // result = compare x, y
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Compare(
                                                crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                    left: crate::ir::ast::common::Operand::Identifier("x".into()),
                                                    right: crate::ir::ast::common::Operand::Identifier("y".into()),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // printf("%lld\n", result)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "printf".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::String("%lld\n".into()),
                                                ),
                                                crate::ir::ast::common::Operand::Identifier("result".into()),
                                            ],
                                        },
                                    )),
                                ],
                            },
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "get_sixty".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Int64.into(),
                            function_body: LocalStatements {
                                statements: vec![LocalStatement::Instruction(
                                    InstructionStatement::Return(crate::ir::ast::local::instruction::return_::ReturnInstruction {
                                        return_value: Some(crate::ir::ast::common::Operand::Literal(LiteralValue::Int64(60))),
                                    }),
                                )],
                            },
                        }),
                    ],
                },
            },
            TestCase {
                name: "COMPARE 명령어 테스트 - Int32 비교",
                expected_output: "0\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_int32.foolang".into(),
                    statements: vec![GlobalStatement::DefineFunction(FunctionDefinition {
                        function_name: "main".into(),
                        arguments: vec![],
                        return_type: IRPrimitiveType::Void.into(),
                        function_body: LocalStatements {
                            statements: vec![
                                // result = compare 5, 7
                                AssignmentStatement {
                                    name: "result".into(),
                                    value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                        InstructionStatement::Compare(
                                            crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                left: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(5),
                                                ),
                                                right: crate::ir::ast::common::Operand::Literal(
                                                    LiteralValue::Int32(7),
                                                ),
                                            }
                                        ),
                                    ),
                                }.into(),
                                // printf("%lld\n", result)
                                LocalStatement::Instruction(InstructionStatement::Call(
                                    CallInstruction {
                                        function_name: "printf".into(),
                                        parameters: vec![
                                            crate::ir::ast::common::Operand::Literal(
                                                LiteralValue::String("%lld\n".into()),
                                            ),
                                            crate::ir::ast::common::Operand::Identifier("result".into()),
                                        ],
                                    },
                                )),
                            ],
                        },
                    })],
                },
            },
            TestCase {
                name: "COMPARE + BRANCH 연계 테스트 - 같은 값",
                expected_output: "EQUAL\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_branch_equal.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "EQUAL_TEXT".into(),
                            value: LiteralValue::String("EQUAL".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "NOT_EQUAL_TEXT".into(),
                            value: LiteralValue::String("NOT EQUAL".into()),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // result = compare 42, 42
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Compare(
                                                crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                    left: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(42),
                                                    ),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(42),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // branch result, equal_label, not_equal_label
                                    LocalStatement::Instruction(InstructionStatement::Branch(
                                        crate::ir::ast::local::instruction::branch::BranchInstruction {
                                            condition: "result".into(),
                                            true_label: "equal_label".into(),
                                            false_label: "not_equal_label".into(),
                                        },
                                    )),
                                    // equal_label:
                                    crate::ir::ast::local::LocalStatement::Label(
                                        crate::ir::ast::local::label::LabelDefinition {
                                            name: "equal_label".into(),
                                        },
                                    ),
                                    // puts(EQUAL_TEXT)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier("EQUAL_TEXT".into()),
                                            ],
                                        },
                                    )),
                                    // return
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        crate::ir::ast::local::instruction::return_::ReturnInstruction {
                                            return_value: None,
                                        },
                                    )),
                                    // not_equal_label:
                                    crate::ir::ast::local::LocalStatement::Label(
                                        crate::ir::ast::local::label::LabelDefinition {
                                            name: "not_equal_label".into(),
                                        },
                                    ),
                                    // puts(NOT_EQUAL_TEXT)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier("NOT_EQUAL_TEXT".into()),
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
                name: "COMPARE + BRANCH 연계 테스트 - 다른 값",
                expected_output: "NOT EQUAL\n",
                want_error: false,
                expected_error: None,
                code_unit: CodeUnit {
                    filename: "compare_branch_not_equal.foolang".into(),
                    statements: vec![
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "EQUAL_TEXT".into(),
                            value: LiteralValue::String("EQUAL".into()),
                        }),
                        GlobalStatement::Constant(ConstantDefinition {
                            constant_name: "NOT_EQUAL_TEXT".into(),
                            value: LiteralValue::String("NOT EQUAL".into()),
                        }),
                        GlobalStatement::DefineFunction(FunctionDefinition {
                            function_name: "main".into(),
                            arguments: vec![],
                            return_type: IRPrimitiveType::Void.into(),
                            function_body: LocalStatements {
                                statements: vec![
                                    // result = compare 10, 20
                                    AssignmentStatement {
                                        name: "result".into(),
                                        value: crate::ir::ast::local::assignment::AssignmentStatementValue::Instruction(
                                            InstructionStatement::Compare(
                                                crate::ir::ast::local::instruction::compare::CompareInstruction {
                                                    left: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(10),
                                                    ),
                                                    right: crate::ir::ast::common::Operand::Literal(
                                                        LiteralValue::Int64(20),
                                                    ),
                                                }
                                            ),
                                        ),
                                    }.into(),
                                    // branch result, equal_label, not_equal_label
                                    LocalStatement::Instruction(InstructionStatement::Branch(
                                        crate::ir::ast::local::instruction::branch::BranchInstruction {
                                            condition: "result".into(),
                                            true_label: "equal_label".into(),
                                            false_label: "not_equal_label".into(),
                                        },
                                    )),
                                    // equal_label:
                                    crate::ir::ast::local::LocalStatement::Label(
                                        crate::ir::ast::local::label::LabelDefinition {
                                            name: "equal_label".into(),
                                        },
                                    ),
                                    // puts(EQUAL_TEXT)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier("EQUAL_TEXT".into()),
                                            ],
                                        },
                                    )),
                                    // return
                                    LocalStatement::Instruction(InstructionStatement::Return(
                                        crate::ir::ast::local::instruction::return_::ReturnInstruction {
                                            return_value: None,
                                        },
                                    )),
                                    // not_equal_label:
                                    crate::ir::ast::local::LocalStatement::Label(
                                        crate::ir::ast::local::label::LabelDefinition {
                                            name: "not_equal_label".into(),
                                        },
                                    ),
                                    // puts(NOT_EQUAL_TEXT)
                                    LocalStatement::Instruction(InstructionStatement::Call(
                                        CallInstruction {
                                            function_name: "puts".into(),
                                            parameters: vec![
                                                crate::ir::ast::common::Operand::Identifier("NOT_EQUAL_TEXT".into()),
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

        let test_cases = success_cases.into_iter().chain(error_cases.into_iter());

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
                        object.err().unwrap().kind,
                        expected_err,
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
                .args(&[object_filename, "-o", executable_filename, "-no-pie"])
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
