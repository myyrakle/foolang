use crate::{
    ir::{
        compile::linux_amd64::function::FunctionContext,
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::{
            addressing::{modrm_rbp_disp32, modrm_rip_relative, sib_rbp_no_index},
            instruction::Instruction,
            register::Register,
            rex::RexPrefix,
        },
        linux::elf::{
            object::ELFObject,
            relocation::{Relocation, RelocationType},
            section::SectionType,
        },
    },
};

pub fn compile_call_instruction(
    instruction: &crate::ir::ast::local::instruction::call::CallInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // x86-64 System V ABI 호출 규약:
    // 인자 순서: RDI, RSI, RDX, RCX, R8, R9, 그 이상은 스택
    // 반환값: RAX
    // 호출자 보존: RBP, RBX, R12-R15
    // 피호출자 자유: RAX, RCX, RDX, RSI, RDI, R8-R11

    let function_name = &instruction.function_name.name;
    let parameters = &instruction.parameters;

    // x86-64 System V ABI parameter registers
    let param_registers = [
        Register::RDI,
        Register::RSI,
        Register::RDX,
        Register::RCX,
        Register::R8,
        Register::R9,
    ];

    // 최대 6개 이상의 매개변수는 스택 사용 (TODO: 추후 구현)
    if parameters.len() > 6 {
        return Err(IRError::new(
            IRErrorKind::NotImplemented,
            "More than 6 parameters not yet supported (stack parameters not implemented)",
        ));
    }

    // 매개변수를 레지스터에 로드
    for (i, param) in parameters.iter().enumerate() {
        let target_reg = param_registers[i];
        compile_parameter_to_register(param, target_reg, context, object)?;
    }

    // call 명령어 생성
    let call_offset = object.text_section.data.len();
    object.text_section.data.push(Instruction::CALL_NEAR);
    object
        .text_section
        .data
        .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

    // 함수 호출에 대한 재배치 정보 추가
    object.relocations.push(Relocation {
        section: SectionType::Text,
        offset: call_offset + 1, // call 명령어의 offset 필드 위치
        symbol: function_name.clone(),
        reloc_type: RelocationType::PltPcRel32,
        addend: Instruction::CALL_ADDEND,
    });

    // 외부 함수 심볼을 UNDEFINED로 추가
    if !object
        .symbol_table
        .symbols
        .iter()
        .any(|s| s.name == *function_name)
    {
        use crate::platforms::linux::elf::symbol::{Symbol, SymbolBinding, SymbolType};

        object.symbol_table.add_symbol(Symbol {
            name: function_name.clone(),
            section: SectionType::Undefined,
            offset: 0,
            size: 0,
            symbol_type: SymbolType::Function,
            binding: SymbolBinding::Global,
        });
    }

    Ok(())
}

/// 매개변수를 특정 레지스터로 로드하는 함수
fn compile_parameter_to_register(
    param: &crate::ir::ast::common::Operand,
    target_reg: Register,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::ast::common::{literal::LiteralValue, Operand};

    match param {
        Operand::Literal(lit) => match lit {
            LiteralValue::Int64(value) => {
                // mov target_reg, immediate (64-bit)
                // REX.W + 0xB8+rd id (64-bit immediate)
                if target_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WB);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                object.text_section.data.push(
                    Instruction::MOV_IMM64_BASE
                        + (target_reg.number() & Instruction::REG_NUMBER_MASK),
                );
                object
                    .text_section
                    .data
                    .extend_from_slice(&value.to_le_bytes());
            }
            LiteralValue::String(s) => {
                // 문자열은 .rodata 섹션에 저장하고 주소를 레지스터에 로드
                // 문자열 상수 이름 생성
                let string_const_name = format!("__str_const_{}", object.rodata_section.data.len());

                // .rodata에 문자열 추가 (null terminator 포함)
                let string_offset = object.rodata_section.data.len();
                object.rodata_section.data.extend_from_slice(s.as_bytes());
                object.rodata_section.data.push(0); // null terminator

                // symbol table에 문자열 상수 추가
                use crate::platforms::linux::elf::symbol::{Symbol, SymbolBinding, SymbolType};
                object.symbol_table.add_symbol(Symbol {
                    name: string_const_name.clone(),
                    section: SectionType::RoData,
                    offset: string_offset,
                    size: s.len() + 1,
                    symbol_type: SymbolType::Object,
                    binding: SymbolBinding::Local,
                });

                // lea target_reg, [rip + offset]
                // REX.W + 0x8D /r + disp32
                let lea_offset = object.text_section.data.len();

                if target_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WR);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                object.text_section.data.push(Instruction::Lea as u8);

                // ModR/M byte: mod=00 (RIP-relative), reg=target_reg, r/m=101 (RIP+disp32)
                object
                    .text_section
                    .data
                    .push(modrm_rip_relative(target_reg.number()));

                // placeholder for displacement
                object
                    .text_section
                    .data
                    .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

                // relocation 추가
                object.relocations.push(Relocation {
                    section: SectionType::Text,
                    offset: lea_offset + 3, // LEA 명령어의 disp32 위치
                    symbol: string_const_name,
                    reloc_type: RelocationType::PcRel32,
                    addend: Instruction::CALL_ADDEND,
                });
            }
            LiteralValue::Boolean(b) => {
                // boolean을 0 또는 1로 변환
                let value = if *b { 1i64 } else { 0i64 };
                if target_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WB);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                object.text_section.data.push(
                    Instruction::MOV_IMM64_BASE
                        + (target_reg.number() & Instruction::REG_NUMBER_MASK),
                );
                object
                    .text_section
                    .data
                    .extend_from_slice(&value.to_le_bytes());
            }
            LiteralValue::Float64(_) => {
                return Err(IRError::new(
                    IRErrorKind::NotImplemented,
                    "Float64 parameters not yet implemented",
                ));
            }
        },
        Operand::Identifier(id) => {
            use crate::ir::compile::linux_amd64::function::VariableLocation;
            use crate::platforms::amd64::register::modrm_reg_reg;

            // 먼저 로컬 변수 확인
            if let Some(var_loc) = context.get_variable(&id.name) {
                match var_loc {
                    VariableLocation::Register(src_reg) => {
                        // 레지스터에 저장된 로컬 변수
                        if *src_reg != target_reg {
                            // mov target_reg, src_reg
                            // REX prefix: target_reg가 R8-R15면 REX.R, src_reg가 R8-R15면 REX.B 필요
                            let needs_rex_r = target_reg.requires_rex();
                            let needs_rex_b = src_reg.requires_rex();

                            if needs_rex_r && needs_rex_b {
                                object.text_section.data.push(RexPrefix::REX_WRB);
                            } else if needs_rex_r {
                                object.text_section.data.push(RexPrefix::REX_WR);
                            } else if needs_rex_b {
                                object.text_section.data.push(RexPrefix::REX_WB);
                            } else {
                                object.text_section.data.push(RexPrefix::RexW as u8);
                            }
                            // MOV r, r/m (reg 필드가 destination, r/m 필드가 source)
                            object.text_section.data.push(Instruction::MovLoad as u8);
                            object
                                .text_section
                                .data
                                .push(modrm_reg_reg(target_reg, *src_reg));
                        }
                        // 같은 레지스터면 아무것도 안함
                    }
                    VariableLocation::Stack(offset) => {
                        // 스택에 저장된 로컬 변수
                        // mov target_reg, [rbp + offset]
                        if target_reg.requires_rex() {
                            object.text_section.data.push(RexPrefix::REX_WR);
                        } else {
                            object.text_section.data.push(RexPrefix::RexW as u8);
                        }
                        // MOV r64, r/m64
                        object.text_section.data.push(Instruction::MovLoad as u8);

                        // ModR/M byte: [RBP + disp32] addressing
                        object
                            .text_section
                            .data
                            .push(modrm_rbp_disp32(target_reg.number()));

                        // SIB byte: scale=1, index=none, base=RBP
                        object.text_section.data.push(sib_rbp_no_index());

                        // displacement (오프셋)
                        object
                            .text_section
                            .data
                            .extend_from_slice(&offset.to_le_bytes());
                    }
                }
            } else if let Some(symbol) = object.symbol_table.find_symbol(&id.name) {
                // 전역 상수/변수: 주소를 레지스터에 로드
                // lea target_reg, [rip + offset]
                let lea_offset = object.text_section.data.len();

                if target_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WR);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                object.text_section.data.push(Instruction::Lea as u8);

                // ModR/M byte: mod=00 (RIP-relative), reg=target_reg, r/m=101 (RIP+disp32)
                object
                    .text_section
                    .data
                    .push(modrm_rip_relative(target_reg.number()));

                // placeholder for displacement
                object
                    .text_section
                    .data
                    .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

                // relocation 추가
                object.relocations.push(Relocation {
                    section: SectionType::Text,
                    offset: lea_offset + 3, // LEA 명령어의 disp32 위치
                    symbol: symbol.name.clone(),
                    reloc_type: RelocationType::PcRel32,
                    addend: Instruction::CALL_ADDEND,
                });
            } else {
                return Err(IRError::new(
                    IRErrorKind::VariableNotFound,
                    &format!(
                        "Variable '{}' not found (neither local nor global)",
                        id.name
                    ),
                ));
            }
        }
    }

    Ok(())
}
