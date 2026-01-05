use crate::{
    ir::{
        ast::local::{
            instruction::branch::{BranchInstruction, JumpInstruction},
            label::LabelDefinition,
        },
        compile::linux_amd64::function::{FunctionContext, LabelLocation},
        error::{IRError, IRErrorKind},
    },
    platforms::{amd64::instruction::Instruction, linux::elf::object::ELFObject},
};

// x86-64 점프 명령 관련 상수
/// 2바이트 opcode의 첫 번째 바이트 (0x0F)
/// 조건부 점프 명령(JZ, JNZ, JE, JNE 등)에서 사용
const TWO_BYTE_OPCODE_PREFIX: u8 = 0x0F;

/// JNZ (Jump if Not Zero) 명령어의 두 번째 바이트
/// 전체 opcode: 0x0F 0x85 cd (near jump)
const JNZ_OPCODE_SECOND_BYTE: u8 = 0x85;

/// 1바이트 opcode를 사용하는 명령어의 opcode 크기
const ONE_BYTE_OPCODE_SIZE: usize = 1;

/// 2바이트 opcode를 사용하는 명령어의 opcode 크기
const TWO_BYTE_OPCODE_SIZE: usize = 2;

/// 32비트 displacement 크기 (4바이트)
const DISPLACEMENT_SIZE: usize = 4;

/// REX prefix + MOV 명령어에서 displacement까지의 오프셋
/// REX(1) + MOV opcode(1) + ModR/M(1) = 3
const REX_MOV_TO_DISP_OFFSET: usize = 3;

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
        return Err(IRError::new(
            IRErrorKind::LabelAlreadyDefined,
            &format!("Label '{}' is already defined", label_name),
        ));
    }

    // 이미 이 라벨을 참조하는 점프가 있었다면 패치
    if let Some(LabelLocation::Undefined(refs)) = context.get_label_location(&label_name).cloned() {
        for jump_offset in refs {
            // jump_offset는 점프 명령어의 opcode 시작 위치
            // displacement 위치 결정:
            // - 1바이트 opcode (JMP): opcode + 1
            // - 2바이트 opcode (JNZ 등): opcode + 2
            let opcode_byte = object.text_section.data[jump_offset];
            let displacement_offset = if opcode_byte == TWO_BYTE_OPCODE_PREFIX {
                // 2바이트 opcode (조건부 점프)
                jump_offset + TWO_BYTE_OPCODE_SIZE
            } else {
                // 1바이트 opcode (무조건 점프)
                jump_offset + ONE_BYTE_OPCODE_SIZE
            };

            // 상대 오프셋 계산: target - (jump_instruction_end)
            // jump_instruction_end = displacement_offset + DISPLACEMENT_SIZE
            let relative_offset =
                (current_offset as i32) - ((displacement_offset + DISPLACEMENT_SIZE) as i32);

            // displacement 패치
            let bytes = relative_offset.to_le_bytes();
            object.text_section.data[displacement_offset..displacement_offset + DISPLACEMENT_SIZE]
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
            // 상대 오프셋 계산: target - (displacement_offset + DISPLACEMENT_SIZE)
            // displacement_offset + DISPLACEMENT_SIZE는 JMP 명령 다음 명령의 시작 위치
            let relative_offset =
                (*target_offset as i32) - ((displacement_offset + DISPLACEMENT_SIZE) as i32);

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
                .extend_from_slice(&[0x00; DISPLACEMENT_SIZE]);

            // forward reference 등록
            context.add_label_reference(label_name, jump_offset);
        }
    }

    Ok(())
}

/// 조건부 분기 명령 컴파일
///
/// condition을 평가하여:
/// - 0이 아니면 (true): true_label로 점프
/// - 0이면 (false): false_label로 점프
///
/// x86-64 명령 시퀀스:
/// 1. condition 값을 레지스터에 로드
/// 2. test reg, reg (값이 0인지 체크)
/// 3. jnz true_label (0이 아니면 true_label로)
/// 4. jmp false_label (0이면 false_label로)
pub fn compile_branch_instruction(
    instruction: &BranchInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::compile::linux_amd64::function::VariableLocation;
    use crate::platforms::amd64::{
        instruction::Instruction,
        register::{modrm_reg_reg, Register},
        rex::RexPrefix,
    };

    let condition_name = &instruction.condition.name;

    // 1. condition 값을 RAX 레지스터에 로드
    // 먼저 로컬 변수 확인
    if let Some(var_location) = context.get_variable(condition_name) {
        match var_location {
            VariableLocation::Register(src_reg) => {
                // 레지스터에 저장된 로컬 변수
                if *src_reg != Register::RAX {
                    // mov rax, src_reg
                    let needs_rex_b = src_reg.requires_rex();
                    if needs_rex_b {
                        object.text_section.data.push(RexPrefix::REX_WB);
                    } else {
                        object.text_section.data.push(RexPrefix::RexW as u8);
                    }
                    object.text_section.data.push(Instruction::MovLoad as u8);
                    object
                        .text_section
                        .data
                        .push(modrm_reg_reg(Register::RAX, *src_reg));
                }
            }
            VariableLocation::Stack(offset) => {
                // 스택에 저장된 로컬 변수
                use crate::platforms::amd64::addressing::{modrm_rbp_disp32, sib_rbp_no_index};

                object.text_section.data.push(RexPrefix::RexW as u8);
                object.text_section.data.push(Instruction::MovLoad as u8);
                object
                    .text_section
                    .data
                    .push(modrm_rbp_disp32(Register::RAX.number()));
                object.text_section.data.push(sib_rbp_no_index());
                object
                    .text_section
                    .data
                    .extend_from_slice(&offset.to_le_bytes());
            }
        }
    } else if let Some(symbol) = object.symbol_table.find_symbol(condition_name) {
        // 전역 상수/변수: 값을 레지스터에 로드
        use crate::platforms::amd64::addressing::modrm_rip_relative;

        // 전역 상수가 정수면 직접 값을 로드
        // 일단 주소를 로드하고 그 값을 읽어옴
        // mov rax, [rip + offset]
        let load_offset = object.text_section.data.len();

        object.text_section.data.push(RexPrefix::RexW as u8);
        object.text_section.data.push(Instruction::MovLoad as u8);
        object
            .text_section
            .data
            .push(modrm_rip_relative(Register::RAX.number()));

        // placeholder for displacement
        object
            .text_section
            .data
            .extend_from_slice(&[0x00; Instruction::DISPLACEMENT_32_SIZE]);

        // relocation 추가
        use crate::platforms::linux::elf::{
            relocation::{Relocation, RelocationType},
            section::SectionType,
        };
        object.relocations.push(Relocation {
            section: SectionType::Text,
            offset: load_offset + REX_MOV_TO_DISP_OFFSET, // MOV 명령어의 disp32 위치
            symbol: symbol.name.clone(),
            reloc_type: RelocationType::PcRel32,
            addend: Instruction::CALL_ADDEND,
        });
    } else {
        return Err(IRError::new(
            IRErrorKind::VariableNotFound,
            &format!(
                "Condition variable '{}' not found (neither local nor global)",
                condition_name
            ),
        ));
    }

    // 2. test rax, rax (RAX가 0인지 체크)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object.text_section.data.push(Instruction::Test as u8);
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RAX, Register::RAX));

    // 3. jnz true_label (0이 아니면 true로)
    let true_label_name = instruction.true_label.name.clone();
    let jnz_offset = object.text_section.data.len();

    // JNZ near opcode (0F 85 cd)
    object.text_section.data.push(TWO_BYTE_OPCODE_PREFIX);
    object.text_section.data.push(JNZ_OPCODE_SECOND_BYTE);

    let true_displacement_offset = object.text_section.data.len();

    // 라벨이 이미 정의되어 있는지 확인
    match context.get_label_location(&true_label_name) {
        Some(LabelLocation::Defined(target_offset)) => {
            // Backward reference
            let relative_offset =
                (*target_offset as i32) - ((true_displacement_offset + DISPLACEMENT_SIZE) as i32);
            object
                .text_section
                .data
                .extend_from_slice(&relative_offset.to_le_bytes());
        }
        Some(LabelLocation::Undefined(_)) | None => {
            // Forward reference
            object
                .text_section
                .data
                .extend_from_slice(&[0x00; DISPLACEMENT_SIZE]);
            context.add_label_reference(true_label_name, jnz_offset);
        }
    }

    // 4. jmp false_label (0이면 false로)
    let false_label_name = instruction.false_label.name.clone();
    let jmp_offset = object.text_section.data.len();

    object.text_section.data.push(Instruction::Jmp as u8);

    let false_displacement_offset = object.text_section.data.len();

    match context.get_label_location(&false_label_name) {
        Some(LabelLocation::Defined(target_offset)) => {
            // Backward reference
            let relative_offset =
                (*target_offset as i32) - ((false_displacement_offset + DISPLACEMENT_SIZE) as i32);
            object
                .text_section
                .data
                .extend_from_slice(&relative_offset.to_le_bytes());
        }
        Some(LabelLocation::Undefined(_)) | None => {
            // Forward reference
            object
                .text_section
                .data
                .extend_from_slice(&[0x00; DISPLACEMENT_SIZE]);
            context.add_label_reference(false_label_name, jmp_offset);
        }
    }

    Ok(())
}
