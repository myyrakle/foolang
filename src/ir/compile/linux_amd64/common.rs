//! 산술 연산 인스트럭션에서 사용하는 공통 함수들

use crate::{
    ir::{
        ast::{
            common::{literal::LiteralValue, Operand},
            types::IRType,
        },
        compile::linux_amd64::function::{FunctionContext, VariableLocation},
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::{
            addressing::{modrm_rbp_disp32, sib_rbp_no_index},
            instruction::Instruction,
            register::{modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::object::ELFObject,
    },
};

/// 두 operand가 호환 가능한 타입인지 검증
/// 정수끼리, 또는 부동소수점끼리만 연산 가능
pub fn validate_operand_types(
    left: &Operand,
    right: &Operand,
    context: &FunctionContext,
    instruction_name: &str,
) -> Result<(), IRError> {
    let left_is_int = is_integer_operand(left, context)?;
    let right_is_int = is_integer_operand(right, context)?;

    if left_is_int != right_is_int {
        return Err(IRError::new(
            IRErrorKind::TypeError,
            &format!(
                "Type mismatch in {} instruction: cannot mix integer and floating-point operands",
                instruction_name
            ),
        ));
    }

    Ok(())
}

/// Operand가 정수 타입인지 확인
fn is_integer_operand(operand: &Operand, context: &FunctionContext) -> Result<bool, IRError> {
    match operand {
        Operand::Literal(lit) => match lit {
            LiteralValue::Int8(_)
            | LiteralValue::Int16(_)
            | LiteralValue::Int32(_)
            | LiteralValue::Int64(_) => Ok(true),
            LiteralValue::Float64(_) => Ok(false),
            LiteralValue::Boolean(_) | LiteralValue::String(_) => Err(IRError::new(
                IRErrorKind::TypeError,
                "Boolean and String types are not supported in arithmetic instructions",
            )),
        },
        Operand::Identifier(id) => match &id.type_ {
            IRType::Primitive(prim_type) => {
                if prim_type.is_integer() {
                    Ok(true)
                } else if prim_type.is_float() {
                    Ok(false)
                } else {
                    Err(IRError::new(
                        IRErrorKind::TypeError,
                        &format!(
                            "Unsupported type for arithmetic instruction: {:?}",
                            prim_type
                        ),
                    ))
                }
            }
            IRType::None => Ok(true), // 기본값: 정수로 간주
            IRType::Custom(_) => Err(IRError::new(
                IRErrorKind::TypeError,
                "Arithmetic instructions require primitive numeric type, not custom type",
            )),
        },
        Operand::SSAValue(ssa_id) => {
            // Phase 12: SSA 값 타입 확인
            if let Some(ssa_value) = context.ssa_values.get(ssa_id) {
                match &ssa_value.type_ {
                    IRType::Primitive(prim_type) => {
                        if prim_type.is_integer() {
                            Ok(true)
                        } else if prim_type.is_float() {
                            Ok(false)
                        } else {
                            Err(IRError::new(
                                IRErrorKind::TypeError,
                                &format!(
                                    "Unsupported type for arithmetic instruction: {:?}",
                                    prim_type
                                ),
                            ))
                        }
                    }
                    IRType::None => Ok(true), // 기본값: 정수로 간주
                    IRType::Custom(_) => Err(IRError::new(
                        IRErrorKind::TypeError,
                        "Arithmetic instructions require primitive numeric type, not custom type",
                    )),
                }
            } else {
                Err(IRError::new(
                    IRErrorKind::VariableNotFound,
                    &format!("SSA value {:?} not found", ssa_id),
                ))
            }
        }
    }
}

/// 정수 immediate 값을 레지스터에 로드하는 헬퍼 함수
/// MOV reg, imm64 형태의 명령어 생성
pub fn emit_mov_imm64(object: &mut ELFObject, target_reg: Register, value: i64) {
    // REX prefix: R8-R15는 REX.B 필요, 그 외는 REX.W만 필요
    emit_rex_prefix(object, None, Some(target_reg));

    // Opcode: MOV_IMM64_BASE + 레지스터 번호 (하위 3비트만 사용)
    object
        .text_section
        .data
        .push(Instruction::MOV_IMM64_BASE + (target_reg.number() & Instruction::REG_NUMBER_MASK));

    // Immediate 값 (8바이트, little-endian)
    object
        .text_section
        .data
        .extend_from_slice(&value.to_le_bytes());
}

/// REX prefix를 생성하여 추가 (64-bit 연산용)
///
/// # Parameters
/// - `reg_field`: Reg 필드에 사용되는 레지스터 (R8-R15이면 REX.R 필요)
/// - `rm_field`: R/M 필드에 사용되는 레지스터 (R8-R15이면 REX.B 필요)
pub fn emit_rex_prefix(
    object: &mut ELFObject,
    reg_field: Option<Register>,
    rm_field: Option<Register>,
) {
    let needs_rex_r = reg_field.is_some_and(|r| r.requires_rex());
    let needs_rex_b = rm_field.is_some_and(|r| r.requires_rex());

    let rex = match (needs_rex_r, needs_rex_b) {
        (true, true) => RexPrefix::REX_WRB,
        (true, false) => RexPrefix::REX_WR,
        (false, true) => RexPrefix::REX_WB,
        (false, false) => RexPrefix::RexW as u8,
    };

    object.text_section.data.push(rex);
}

/// 리터럴 값을 레지스터에 로드
pub fn load_literal_to_register(
    lit: &LiteralValue,
    target_reg: Register,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match lit {
        LiteralValue::Int8(value) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        LiteralValue::Int16(value) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        LiteralValue::Int32(value) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        LiteralValue::Int64(value) => {
            // MOV reg, imm64
            emit_mov_imm64(object, target_reg, *value);
        }
        _ => {
            return Err(IRError::new(
                IRErrorKind::TypeError,
                &format!("Unsupported literal type for register load: {:?}", lit),
            ));
        }
    }
    Ok(())
}

/// 식별자(변수)를 레지스터에 로드
pub fn load_identifier_to_register(
    var_name: &str,
    target_reg: Register,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // 먼저 로컬 변수 확인
    if let Some(var_loc) = context.get_variable(var_name) {
        match var_loc {
            VariableLocation::Register(src_reg) => {
                if *src_reg != target_reg {
                    // MOV target_reg, src_reg
                    emit_rex_prefix(object, Some(target_reg), Some(*src_reg));
                    object.text_section.data.push(Instruction::MovLoad as u8);
                    object
                        .text_section
                        .data
                        .push(modrm_reg_reg(target_reg, *src_reg));
                }
                // 같은 레지스터면 아무것도 안 함
            }
            VariableLocation::Stack(offset) => {
                // MOV target_reg, [rbp + offset]
                emit_rex_prefix(object, Some(target_reg), None);
                object.text_section.data.push(Instruction::MovLoad as u8);
                object
                    .text_section
                    .data
                    .push(modrm_rbp_disp32(target_reg.number()));
                object.text_section.data.push(sib_rbp_no_index());
                object
                    .text_section
                    .data
                    .extend_from_slice(&offset.to_le_bytes());
            }
        }
    } else if let Some(symbol) = object.symbol_table.find_symbol(var_name) {
        // 전역 상수/변수: RIP-relative addressing으로 로드
        use crate::platforms::amd64::addressing::modrm_rip_relative;
        use crate::platforms::linux::elf::relocation::{Relocation, RelocationType};
        use crate::platforms::linux::elf::section::SectionType;

        // Borrow checker를 위해 symbol name을 먼저 clone
        let symbol_name = symbol.name.clone();
        let load_offset = object.text_section.data.len();

        emit_rex_prefix(object, Some(target_reg), None);
        object.text_section.data.push(Instruction::MovLoad as u8);
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
        const REX_MOV_TO_DISP_OFFSET: usize = 3;
        object.relocations.push(Relocation {
            section: SectionType::Text,
            offset: load_offset + REX_MOV_TO_DISP_OFFSET,
            symbol: symbol_name,
            reloc_type: RelocationType::PcRel32,
            addend: Instruction::CALL_ADDEND,
        });
    } else {
        return Err(IRError::new(
            IRErrorKind::VariableNotFound,
            &format!("Variable '{}' not found (neither local nor global)", var_name),
        ));
    }

    Ok(())
}

/// Operand를 지정된 레지스터에 로드
pub fn load_operand_to_register(
    operand: &Operand,
    target_reg: Register,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match operand {
        Operand::Literal(lit) => {
            load_literal_to_register(lit, target_reg, object)?;
        }
        Operand::Identifier(id) => {
            // Phase 12: SSA 기반 또는 변수명 기반 로딩
            if context.liveness.is_some() {
                // SSA 기반: 변수명 -> SSA 값 -> 위치
                // 로컬 변수가 아니면 (전역 상수 등) 기존 방식으로 폴백
                if let Some(ssa_id) = context.get_current_version(&id.name) {
                    load_ssa_value_to_register(ssa_id, target_reg, context, object)?;

                    // Phase 13: 마지막 사용 후 레지스터 해제
                    context.free_ssa_value_if_last_use(ssa_id);
                } else {
                    // SSA context에 없으면 전역 상수/변수일 수 있음
                    load_identifier_to_register(&id.name, target_reg, context, object)?;
                }
            } else {
                // 기존 방식: 변수명 -> 위치
                load_identifier_to_register(&id.name, target_reg, context, object)?;
            }
        }
        Operand::SSAValue(ssa_id) => {
            // Phase 12: SSA 값 직접 로딩
            load_ssa_value_to_register(*ssa_id, target_reg, context, object)?;

            // Phase 13: 마지막 사용 후 레지스터 해제
            context.free_ssa_value_if_last_use(*ssa_id);
        }
    }
    Ok(())
}

/// SSA 값을 레지스터에 로드 (Phase 12)
fn load_ssa_value_to_register(
    ssa_id: crate::ir::ssa::SSAValueId,
    target_reg: Register,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::ssa::register_allocator::ValueLocation as SSAValueLocation;

    let ssa_loc = context.get_ssa_value_location(ssa_id).ok_or_else(|| {
        IRError::new(
            IRErrorKind::VariableNotFound,
            &format!("SSA value {:?} not found", ssa_id),
        )
    })?;

    match ssa_loc {
        SSAValueLocation::Register(src_reg) => {
            if *src_reg != target_reg {
                // MOV target_reg, src_reg
                emit_rex_prefix(object, Some(target_reg), Some(*src_reg));
                object.text_section.data.push(Instruction::MovLoad as u8);
                object
                    .text_section
                    .data
                    .push(modrm_reg_reg(target_reg, *src_reg));
            }
            // 같은 레지스터면 아무것도 안 함
        }
        SSAValueLocation::Spilled(offset) => {
            // MOV target_reg, [rbp + offset]
            emit_rex_prefix(object, Some(target_reg), None);
            object.text_section.data.push(Instruction::MovLoad as u8);
            object
                .text_section
                .data
                .push(modrm_rbp_disp32(target_reg.number()));
            object.text_section.data.push(sib_rbp_no_index());
            object
                .text_section
                .data
                .extend_from_slice(&offset.to_le_bytes());
        }
        SSAValueLocation::Unassigned => {
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                &format!("SSA value {:?} is unassigned", ssa_id),
            ));
        }
    }

    Ok(())
}

