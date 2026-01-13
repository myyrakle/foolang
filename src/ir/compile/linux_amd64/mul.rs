use crate::{
    ir::{
        ast::local::instruction::mul::MulInstruction,
        compile::linux_amd64::{
            common::{load_operand_to_register, validate_operand_types},
            function::FunctionContext,
        },
        error::IRError,
    },
    platforms::{
        amd64::{
            instruction::Instruction,
            register::{modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::object::ELFObject,
    },
};

/// MUL 인스트럭션 컴파일
///
/// 전략:
/// 1. 두 operand의 타입 검증 (정수끼리만 가능)
/// 2. left operand를 RAX에 로드
/// 3. right operand를 RCX에 로드
/// 4. IMUL RAX, RCX 명령 생성 (결과는 RAX에 저장됨)
pub fn compile_mul_instruction(
    mul_instruction: &MulInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // Step 1: 타입 검증
    validate_operand_types(
        &mul_instruction.left,
        &mul_instruction.right,
        context,
        "MUL",
    )?;

    // Step 2: left operand를 RAX에 로드
    load_operand_to_register(&mul_instruction.left, Register::RAX, context, object)?;

    // Step 3: right operand를 RCX에 로드
    load_operand_to_register(&mul_instruction.right, Register::RCX, context, object)?;

    // Step 4: IMUL 명령 생성 (IMUL RAX, RCX)
    // REX.W + 0F AF /r (IMUL r64, r/m64)
    // RAX = RAX * RCX (signed multiplication)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object
        .text_section
        .data
        .extend_from_slice(&Instruction::IMul.as_bytes()); // IMUL opcode (0x0F 0xAF)
    // modrm_reg_reg(reg, rm) => reg 필드에 RAX, r/m 필드에 RCX
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RAX, Register::RCX));

    Ok(())
}
