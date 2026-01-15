use crate::{
    ir::{
        ast::local::instruction::compare::CompareInstruction,
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

/// COMPARE 인스트럭션 컴파일
///
/// 전략:
/// 1. 두 operand의 타입 검증 (정수끼리만 가능)
/// 2. left operand를 RAX에 로드
/// 3. right operand를 RCX에 로드
/// 4. CMP RAX, RCX 명령 생성 (플래그 설정)
/// 5. SETE AL 명령으로 ZF를 바이트로 변환 (같으면 1, 다르면 0)
/// 6. MOVZX RAX, AL로 8비트를 64비트로 zero-extend
///
/// 결과: left == right이면 1, 다르면 0이 RAX에 저장됨
pub fn compile_compare_instruction(
    compare_instruction: &CompareInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // Step 1: 타입 검증
    validate_operand_types(
        &compare_instruction.left,
        &compare_instruction.right,
        context,
        "COMPARE",
    )?;

    // Step 2: left operand를 RAX에 로드
    load_operand_to_register(&compare_instruction.left, Register::RAX, context, object)?;

    // Step 3: right operand를 RCX에 로드
    load_operand_to_register(&compare_instruction.right, Register::RCX, context, object)?;

    // Step 4: CMP RAX, RCX (RAX와 RCX 비교, 플래그 설정)
    // REX.W + 0x39 /r (CMP r/m64, r64)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object.text_section.data.push(Instruction::Cmp as u8);
    // ModR/M: reg=RCX, r/m=RAX (CMP는 r/m과 r을 비교)
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RCX, Register::RAX));

    // Step 5: SETE AL (ZF=1이면 AL=1, ZF=0이면 AL=0)
    // 0x0F 0x94 /0 (SETE r/m8)
    object
        .text_section
        .data
        .extend_from_slice(&Instruction::Sete.as_bytes());
    // ModR/M: Mod=11 (register), Reg=000 (opcode extension), R/M=000 (AL)
    // AL은 RAX의 하위 8비트 (레지스터 번호 0)
    object
        .text_section
        .data
        .push(Instruction::MODRM_AL_REGISTER);

    // Step 6: MOVZX RAX, AL (AL을 RAX로 zero-extend)
    // REX.W + 0x0F 0xB6 /r (MOVZX r64, r/m8)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object
        .text_section
        .data
        .extend_from_slice(&Instruction::Movzx.as_bytes());
    // ModR/M: reg=RAX, r/m=AL
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RAX, Register::RAX));

    Ok(())
}
