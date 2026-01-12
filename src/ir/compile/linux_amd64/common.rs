/// 산술 연산 인스트럭션에서 사용하는 공통 함수들

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
fn is_integer_operand(operand: &Operand, _context: &FunctionContext) -> Result<bool, IRError> {
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
    let needs_rex_r = reg_field.map_or(false, |r| r.requires_rex());
    let needs_rex_b = rm_field.map_or(false, |r| r.requires_rex());

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
    let var_loc = context.get_variable(var_name).ok_or_else(|| {
        IRError::new(
            IRErrorKind::VariableNotFound,
            &format!("Variable '{}' not found", var_name),
        )
    })?;

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

    Ok(())
}

/// Operand를 지정된 레지스터에 로드
pub fn load_operand_to_register(
    operand: &Operand,
    target_reg: Register,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match operand {
        Operand::Literal(lit) => {
            load_literal_to_register(lit, target_reg, object)?;
        }
        Operand::Identifier(id) => {
            load_identifier_to_register(&id.name, target_reg, context, object)?;
        }
    }
    Ok(())
}
