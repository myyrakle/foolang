use crate::{
    ir::{
        ast::{
            common::{literal::LiteralValue, Operand},
            local::instruction::add::AddInstruction,
        },
        compile::linux_amd64::function::{FunctionContext, VariableLocation},
        error::{IRError, IRErrorKind},
    },
    platforms::{
        amd64::{
            addressing::{modrm_rbp_disp32, modrm_rip_relative, sib_rbp_no_index},
            instruction::Instruction,
            register::{modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::{
            object::ELFObject,
            relocation::{Relocation, RelocationType},
            section::SectionType,
            symbol::{Symbol, SymbolBinding, SymbolType},
        },
    },
};

/// Operand 크기 및 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSize {
    // 정수 타입
    Int8,
    Int16,
    Int32,
    Int64,
    // 부동소수점 타입
    Float32,
    Float64,
}

/// ADD 인스트럭션 컴파일
///
/// 전략:
/// 1. 두 operand의 크기 결정
/// 2. left operand를 RAX에 로드
/// 3. right operand를 RCX에 로드
/// 4. ADD RAX, RCX 명령 생성 (결과는 RAX에 저장됨)
pub fn compile_add_instruction(
    add_instruction: &AddInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // Step 1: Operand 크기 결정
    let operand_size =
        determine_operand_size(&add_instruction.left, &add_instruction.right, context)?;

    // Step 2: left operand를 RAX에 로드
    load_operand_to_register(
        &add_instruction.left,
        Register::RAX,
        operand_size,
        context,
        object,
    )?;

    // Step 3: right operand를 RCX에 로드
    load_operand_to_register(
        &add_instruction.right,
        Register::RCX,
        operand_size,
        context,
        object,
    )?;

    // Step 4: ADD 명령 생성 (ADD RAX, RCX)
    emit_add_instruction(operand_size, object)?;

    Ok(())
}

/// Operand를 지정된 레지스터에 로드
fn load_operand_to_register(
    operand: &Operand,
    target_reg: Register,
    size: OperandSize,
    context: &FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match operand {
        Operand::Literal(lit) => {
            load_literal_to_register(lit, target_reg, size, object)?;
        }
        Operand::Identifier(id) => {
            load_identifier_to_register(&id.name, target_reg, context, object)?;
        }
    }
    Ok(())
}

/// 정수 immediate 값을 레지스터에 로드하는 헬퍼 함수
/// MOV reg, imm64 형태의 명령어 생성
fn emit_mov_imm64(object: &mut ELFObject, target_reg: Register, value: i64) {
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

/// 리터럴 값을 레지스터에 로드
fn load_literal_to_register(
    lit: &LiteralValue,
    target_reg: Register,
    size: OperandSize,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    match (lit, size) {
        (LiteralValue::Int8(value), OperandSize::Int8) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        (LiteralValue::Int16(value), OperandSize::Int16) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        (LiteralValue::Int32(value), OperandSize::Int32) => {
            // MOV reg, imm64 (sign-extended)
            emit_mov_imm64(object, target_reg, *value as i64);
        }
        (LiteralValue::Int64(value), OperandSize::Int64) => {
            // MOV reg, imm64
            emit_mov_imm64(object, target_reg, *value);
        }
        (LiteralValue::Float64(value), OperandSize::Float64) => {
            // 부동소수점 리터럴은 메모리에 저장하고 로드해야 함
            // 1. 리터럴을 .rodata에 저장
            let const_name = format!("__float64_const_{}", object.rodata_section.data.len());
            let const_offset = object.rodata_section.data.len();
            object
                .rodata_section
                .data
                .extend_from_slice(&value.to_le_bytes());

            // 2. 심볼 테이블에 추가
            object.symbol_table.add_symbol(Symbol {
                name: const_name.clone(),
                section: SectionType::RoData,
                offset: const_offset,
                size: 8,
                symbol_type: SymbolType::Object,
                binding: SymbolBinding::Local,
            });

            // 3. LEA target_reg, [rip + offset] 방식으로 주소 로드
            // XMM 레지스터는 일반 레지스터 번호를 사용하지만 다른 인코딩 필요
            // 우선은 범용 레지스터에 로드 (나중에 MOVSD로 XMM으로 옮김)
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::Lea as u8);
            object
                .text_section
                .data
                .push(modrm_rip_relative(target_reg.number()));

            // Displacement는 링커가 relocation을 통해 채움
            let reloc_offset = object.text_section.data.len();
            object
                .text_section
                .data
                .extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

            // Relocation 추가
            object.relocations.push(Relocation {
                section: SectionType::Text,
                offset: reloc_offset,
                symbol: const_name,
                reloc_type: RelocationType::PcRel32,
                addend: -4,
            });
        }
        _ => {
            return Err(IRError::new(
                IRErrorKind::TypeError,
                &format!(
                    "Type mismatch: literal {:?} does not match operand size {:?}",
                    lit, size
                ),
            ));
        }
    }
    Ok(())
}

/// REX prefix를 생성하여 추가 (64-bit 연산용)
///
/// # Parameters
/// - `reg_field`: Reg 필드에 사용되는 레지스터 (R8-R15이면 REX.R 필요)
/// - `rm_field`: R/M 필드에 사용되는 레지스터 (R8-R15이면 REX.B 필요)
fn emit_rex_prefix(
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

/// 식별자(변수)를 레지스터에 로드
fn load_identifier_to_register(
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

/// Operand의 크기 결정
fn determine_operand_size(
    left: &Operand,
    right: &Operand,
    context: &FunctionContext,
) -> Result<OperandSize, IRError> {
    let left_size = get_operand_size(left, context)?;
    let right_size = get_operand_size(right, context)?;

    match (left_size, right_size) {
        (Some(l), Some(r)) if l != r => Err(IRError::new(
            IRErrorKind::TypeError,
            &format!(
                "Operand type mismatch in ADD instruction: {:?} vs {:?}",
                l, r
            ),
        )),
        (Some(size), _) | (_, Some(size)) => Ok(size),
        (None, None) => Ok(OperandSize::Int64), // 기본값: 64-bit integer
    }
}

/// 개별 Operand의 크기 가져오기
fn get_operand_size(
    operand: &Operand,
    _context: &FunctionContext,
) -> Result<Option<OperandSize>, IRError> {
    use crate::ir::ast::types::IRPrimitiveType;

    match operand {
        Operand::Literal(lit) => match lit {
            LiteralValue::Int8(_) => Ok(Some(OperandSize::Int8)),
            LiteralValue::Int16(_) => Ok(Some(OperandSize::Int16)),
            LiteralValue::Int32(_) => Ok(Some(OperandSize::Int32)),
            LiteralValue::Int64(_) => Ok(Some(OperandSize::Int64)),
            LiteralValue::Float64(_) => Ok(Some(OperandSize::Float64)),
            LiteralValue::Boolean(_) | LiteralValue::String(_) => Err(IRError::new(
                IRErrorKind::TypeError,
                "Boolean and String types are not supported in ADD instruction",
            )),
        },
        Operand::Identifier(id) => {
            // 식별자의 타입 정보를 사용하여 크기 결정
            use crate::ir::ast::types::IRType;

            match &id.type_ {
                IRType::Primitive(prim_type) => match prim_type {
                    IRPrimitiveType::Int8 => Ok(Some(OperandSize::Int8)),
                    IRPrimitiveType::Int16 => Ok(Some(OperandSize::Int16)),
                    IRPrimitiveType::Int32 => Ok(Some(OperandSize::Int32)),
                    IRPrimitiveType::Int64 => Ok(Some(OperandSize::Int64)),
                    IRPrimitiveType::UInt8 => Ok(Some(OperandSize::Int8)),
                    IRPrimitiveType::UInt16 => Ok(Some(OperandSize::Int16)),
                    IRPrimitiveType::UInt32 => Ok(Some(OperandSize::Int32)),
                    IRPrimitiveType::UInt64 => Ok(Some(OperandSize::Int64)),
                    IRPrimitiveType::Float32 => Ok(Some(OperandSize::Float32)),
                    IRPrimitiveType::Float64 => Ok(Some(OperandSize::Float64)),
                    _ => Err(IRError::new(
                        IRErrorKind::TypeError,
                        &format!("Unsupported type for ADD instruction: {:?}", prim_type),
                    )),
                },
                IRType::None => {
                    // 타입 정보가 없는 경우 (구 코드 호환성을 위해)
                    // None을 반환하여 다른 operand의 타입 사용 또는 기본값 사용
                    Ok(None)
                }
                IRType::Custom(_) => Err(IRError::new(
                    IRErrorKind::TypeError,
                    "ADD instruction requires primitive numeric type, not custom type",
                )),
            }
        }
    }
}

/// ADD 명령 생성
///
/// 정수: ADD r/m, reg 는 r/m += reg 를 의미합니다.
/// ModR/M byte: reg 필드에는 source (RCX), r/m 필드에는 destination (RAX)
///
/// 부동소수점: ADDSD xmm0, xmm1 (xmm0 += xmm1) 형태 사용
/// 참고: 현재는 부동소수점을 범용 레지스터에 주소로 로드한 상태이므로,
///      실제로는 메모리에서 XMM 레지스터로 로드 후 연산 필요
fn emit_add_instruction(size: OperandSize, object: &mut ELFObject) -> Result<(), IRError> {
    match size {
        OperandSize::Int8 | OperandSize::Int16 | OperandSize::Int32 | OperandSize::Int64 => {
            // 정수 ADD: 모두 64-bit로 단순화
            // REX.W + ADD rax, rcx (RAX += RCX)
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::Add as u8);
            // modrm_reg_reg(reg, rm) => reg 필드에 RCX, r/m 필드에 RAX
            object
                .text_section
                .data
                .push(modrm_reg_reg(Register::RCX, Register::RAX));
        }
        OperandSize::Float32 => {
            // ADDSS xmm0, xmm1 (Scalar Single-Precision Float ADD)
            // 인코딩: F3 0F 58 /r
            // 현재 구현은 복잡하므로 NotImplemented 반환
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Float32 ADD not yet fully implemented (requires XMM register handling)",
            ));
        }
        OperandSize::Float64 => {
            // ADDSD xmm0, xmm1 (Scalar Double-Precision Float ADD)
            // 인코딩: F2 0F 58 /r
            //
            // 복잡도:
            // 1. RAX와 RCX는 현재 Float64 상수의 주소를 가리킴
            // 2. MOVSD로 메모리에서 XMM0, XMM1로 로드 필요
            // 3. ADDSD XMM0, XMM1 실행
            // 4. MOVSD로 XMM0를 메모리(또는 RAX를 통해)로 저장
            //
            // 현재 단계에서는 NotImplemented로 표시
            return Err(IRError::new(
                IRErrorKind::NotImplemented,
                "Float64 ADD not yet fully implemented (requires XMM register support and memory operations)",
            ));
        }
    }
    Ok(())
}
