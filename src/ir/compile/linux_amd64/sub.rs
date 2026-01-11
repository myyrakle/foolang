use crate::{
    ir::{
        ast::local::instruction::sub::SubInstruction,
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

/// SUB 인스트럭션 컴파일
///
/// 전략:
/// 1. 두 operand의 타입 검증 (정수끼리만 가능)
/// 2. left operand를 RAX에 로드
/// 3. right operand를 RCX에 로드
/// 4. SUB RAX, RCX 명령 생성 (결과는 RAX에 저장됨)
pub fn compile_sub_instruction(
    sub_instruction: &SubInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // Step 1: 타입 검증
    validate_operand_types(
        &sub_instruction.left,
        &sub_instruction.right,
        context,
        "SUB",
    )?;

    // Step 2: left operand를 RAX에 로드
    load_operand_to_register(&sub_instruction.left, Register::RAX, context, object)?;

    // Step 3: right operand를 RCX에 로드
    load_operand_to_register(&sub_instruction.right, Register::RCX, context, object)?;

    // Step 4: SUB 명령 생성 (SUB RAX, RCX)
    // REX.W + SUB r/m64, r64 (RAX -= RCX)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object.text_section.data.push(Instruction::Sub as u8);
    // modrm_reg_reg(reg, rm) => reg 필드에 RCX, r/m 필드에 RAX
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RCX, Register::RAX));

    Ok(())
}
