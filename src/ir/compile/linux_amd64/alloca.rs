use crate::{
    ir::{
        ast::local::instruction::alloca::AllocaInstruction,
        compile::linux_amd64::function::FunctionContext, error::IRError,
    },
    platforms::linux::elf::object::ELFObject,
};

pub fn compile_alloca_instruction(
    alloca_instruction: &AllocaInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::platforms::amd64::{
        instruction::Instruction,
        register::{modrm_reg_base_disp32, modrm_reg_base_disp8, Register},
        rex::RexPrefix,
    };

    // 타입 크기 계산
    let type_size = alloca_instruction.type_.size_in_bytes();

    // 스택 오프셋 계산: 일반 로컬 변수 다음부터 alloca 공간 할당
    // stack_offset은 음수이므로, 거기서 더 빼면 더 깊은 스택 위치로 이동
    //
    // 중요: prescan_statements에서 이미 모든 alloca의 크기를 pending_alloca_size에
    // 누적했고, required_stack_size()가 이를 포함하여 prologue에서 충분한 스택을
    // 미리 할당했으므로, 여기서 stack_offset을 조정해도 안전함
    context.stack_offset -= type_size as i32;
    let stack_offset = context.stack_offset;

    // LEA rax, [rbp + offset] - 스택 주소를 RAX에 로드
    // REX.W prefix (64-bit operand)
    object.text_section.data.push(RexPrefix::RexW as u8);

    // LEA opcode
    object.text_section.data.push(Instruction::Lea as u8);

    // ModR/M + displacement
    if stack_offset >= -128 && stack_offset < 0 {
        // disp8 범위: [rbp + disp8] 인코딩
        object
            .text_section
            .data
            .push(modrm_reg_base_disp8(Register::RAX, Register::RBP));
        object.text_section.data.push(stack_offset as i8 as u8);
    } else {
        // disp32 범위: [rbp + disp32] 인코딩
        object
            .text_section
            .data
            .push(modrm_reg_base_disp32(Register::RAX, Register::RBP));
        object
            .text_section
            .data
            .extend_from_slice(&(stack_offset as i32).to_le_bytes());
    }

    Ok(())
}
