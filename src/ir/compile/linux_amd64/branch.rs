use crate::{
    ir::{
        ast::local::{instruction::branch::JumpInstruction, label::LabelDefinition},
        compile::linux_amd64::function::{FunctionContext, LabelLocation},
        error::IRError,
    },
    platforms::{amd64::instruction::Instruction, linux::elf::object::ELFObject},
};

/// 라벨 정의 컴파일
///
/// 라벨은 실제 코드를 생성하지 않고, 현재 위치만 기록합니다.
/// 이미 이 라벨을 참조하는 점프 명령이 있었다면 (forward reference),
/// 해당 점프 명령들의 오프셋을 패치합니다.
pub fn compile_label_definition(
    label_definition: &LabelDefinition,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    let label_name = label_definition.name.name.clone();
    let current_offset = object.text_section.data.len();

    // 라벨이 이미 정의되어 있는지 확인
    if let Some(LabelLocation::Defined(_)) = context.get_label_location(&label_name) {
        return Err(IRError::new(&format!(
            "Label '{}' is already defined",
            label_name
        )));
    }

    // 이미 이 라벨을 참조하는 점프가 있었다면 패치
    if let Some(LabelLocation::Undefined(refs)) = context.get_label_location(&label_name).cloned()
    {
        for jump_offset in refs {
            // jump_offset는 JMP 명령어의 opcode 위치
            // displacement는 opcode 다음 바이트부터 시작
            let displacement_offset = jump_offset + 1;

            // 상대 오프셋 계산: target - (jump_instruction_end)
            // jump_instruction_end = displacement_offset + 4
            let relative_offset =
                (current_offset as i32) - ((displacement_offset + 4) as i32);

            // displacement 패치
            let bytes = relative_offset.to_le_bytes();
            object.text_section.data[displacement_offset..displacement_offset + 4]
                .copy_from_slice(&bytes);
        }
    }

    // 라벨 위치 등록
    context.define_label(label_name, current_offset);

    Ok(())
}

/// 무조건 점프 명령 컴파일
///
/// JMP near (E9 cd) - 32비트 상대 오프셋 점프
/// 라벨이 이미 정의되어 있으면 즉시 오프셋 계산,
/// 아직 정의되지 않았으면 placeholder를 넣고 나중에 패치
pub fn compile_jump_instruction(
    instruction: &JumpInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    let label_name = instruction.label.name.clone();
    let jump_offset = object.text_section.data.len();

    // JMP near opcode
    object.text_section.data.push(Instruction::Jmp as u8);

    // displacement 위치
    let displacement_offset = object.text_section.data.len();

    // 라벨이 이미 정의되어 있는지 확인
    match context.get_label_location(&label_name) {
        Some(LabelLocation::Defined(target_offset)) => {
            // Backward reference: 라벨이 이미 정의됨
            // 상대 오프셋 계산: target - (current + 5)
            // current + 5는 JMP 명령 다음 명령의 시작 위치
            let relative_offset =
                (*target_offset as i32) - ((displacement_offset + 4) as i32);

            object
                .text_section
                .data
                .extend_from_slice(&relative_offset.to_le_bytes());
        }
        Some(LabelLocation::Undefined(_)) | None => {
            // Forward reference: 라벨이 아직 정의되지 않음
            // placeholder로 0을 넣고, 나중에 라벨 정의 시 패치
            object
                .text_section
                .data
                .extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

            // forward reference 등록
            context.add_label_reference(label_name, jump_offset);
        }
    }

    Ok(())
}
