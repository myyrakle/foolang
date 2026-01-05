use crate::{
    ir::{
        compile::linux_amd64::function::{generate_epilogue, FunctionContext},
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::{
            addressing::{modrm_rbp_disp32, sib_rbp_no_index},
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

pub fn compile_return_instruction(
    instruction: &crate::ir::ast::local::instruction::return_::ReturnInstruction,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // x86-64 System V ABI에 따라:
    // - 반환값은 RAX 레지스터에 저장
    // - ret 명령어로 함수 종료

    // 반환값이 있으면 RAX 레지스터에 로드
    if let Some(return_value) = &instruction.return_value {
        load_value_to_register(return_value, Register::RAX, context, object)?;
    }

    // Function epilogue 생성 (callee-saved 레지스터 복원, 스택 해제, ret)
    generate_epilogue(context, object);

    Ok(())
}

/// 값을 특정 레지스터로 로드하는 함수
fn load_value_to_register(
    operand: &crate::ir::ast::common::Operand,
    target_reg: Register,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::ast::common::{literal::LiteralValue, Operand};

    match operand {
        Operand::Literal(lit) => match lit {
            LiteralValue::Int64(value) => {
                // mov target_reg, immediate (64-bit)
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
                let string_const_name = format!("__str_const_{}", object.rodata_section.data.len());

                let string_offset = object.rodata_section.data.len();
                object.rodata_section.data.extend_from_slice(s.as_bytes());
                object.rodata_section.data.push(0); // null terminator

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
                let lea_offset = object.text_section.data.len();

                if target_reg.requires_rex() {
                    object.text_section.data.push(RexPrefix::REX_WR);
                } else {
                    object.text_section.data.push(RexPrefix::RexW as u8);
                }
                object.text_section.data.push(Instruction::Lea as u8);

                let modrm = ((target_reg.number() & Instruction::REG_NUMBER_MASK)
                    << Instruction::MODRM_REG_SHIFT)
                    | Instruction::MODRM_RIP_RELATIVE_RM;
                object.text_section.data.push(modrm);

                object
                    .text_section
                    .data
                    .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

                object.relocations.push(Relocation {
                    section: SectionType::Text,
                    offset: lea_offset + 3,
                    symbol: string_const_name,
                    reloc_type: RelocationType::PcRel32,
                    addend: Instruction::CALL_ADDEND,
                });
            }
            LiteralValue::Boolean(b) => {
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
                    "Float64 return values not yet implemented",
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

                let modrm = ((target_reg.number() & Instruction::REG_NUMBER_MASK)
                    << Instruction::MODRM_REG_SHIFT)
                    | Instruction::MODRM_RIP_RELATIVE_RM;
                object.text_section.data.push(modrm);

                object
                    .text_section
                    .data
                    .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

                object.relocations.push(Relocation {
                    section: SectionType::Text,
                    offset: lea_offset + 3,
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
