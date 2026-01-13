use crate::{
    ir::{
        ast::local::instruction::rem::RemInstruction,
        compile::linux_amd64::{
            common::{emit_rex_prefix, load_operand_to_register, validate_operand_types},
            function::FunctionContext,
        },
        error::IRError,
    },
    platforms::{
        amd64::{
            instruction::Instruction,
            register::{modrm_digit_reg, modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::object::ELFObject,
    },
};

/// REM 인스트럭션 컴파일
///
/// 전략:
/// 1. 두 operand의 타입 검증 (정수끼리만 가능)
/// 2. left operand(피제수)를 RAX에 로드
/// 3. CQO 명령으로 RAX를 RDX:RAX로 sign-extend
/// 4. right operand(제수)를 RCX에 로드
/// 5. IDIV RCX 명령 생성 (몫은 RAX에, 나머지는 RDX에 저장됨)
/// 6. RDX(나머지)를 RAX로 이동
pub fn compile_rem_instruction(
    rem_instruction: &RemInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // Step 1: 타입 검증
    validate_operand_types(
        &rem_instruction.left,
        &rem_instruction.right,
        context,
        "REM",
    )?;

    // Step 2: left operand(피제수)를 RAX에 로드
    load_operand_to_register(&rem_instruction.left, Register::RAX, context, object)?;

    // Step 3: CQO - RAX를 RDX:RAX로 sign-extend
    // REX.W + 99 (CQO: sign-extend RAX into RDX:RAX)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object
        .text_section
        .data
        .extend_from_slice(&Instruction::Cqo.as_bytes()); // CQO (0x99)

    // Step 4: right operand(제수)를 RCX에 로드
    load_operand_to_register(&rem_instruction.right, Register::RCX, context, object)?;

    // Step 5: IDIV RCX 명령 생성
    // REX.W + F7 /7 (IDIV r/m64)
    // 몫은 RAX에, 나머지는 RDX에 저장
    object.text_section.data.push(RexPrefix::RexW as u8);
    object
        .text_section
        .data
        .extend_from_slice(&Instruction::IDiv.as_bytes()); // IDIV opcode (0xF7)

    // ModR/M: Mod=11 (register direct), Reg=/7 (IDIV extension), R/M=RCX
    let digit = Instruction::IDiv.modrm_extension().unwrap(); // /7
    object
        .text_section
        .data
        .push(modrm_digit_reg(digit, Register::RCX));

    // Step 6: RDX(나머지)를 RAX로 이동
    // MOV RAX, RDX
    emit_rex_prefix(object, Some(Register::RAX), Some(Register::RDX));
    object.text_section.data.push(Instruction::MovLoad as u8);
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RAX, Register::RDX));

    Ok(())
}
