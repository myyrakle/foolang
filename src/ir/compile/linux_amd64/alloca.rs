use crate::{
    ir::{
        ast::local::instruction::alloca::{AllocaInstruction, LoadInstruction, StoreInstruction},
        compile::linux_amd64::function::FunctionContext,
        error::IRError,
    },
    platforms::{
        amd64::addressing::*,
        linux::elf::object::ELFObject,
    },
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
    //
    // 주의: prologue에서 이미 adjust_stack_offsets()로 callee-saved 공간을 고려했으므로
    // 여기서는 추가 보정이 필요없음
    context.stack_offset -= type_size as i32;
    let stack_offset = context.stack_offset;

    // LEA rax, [rbp + offset] - 스택 주소를 RAX에 로드
    // REX.W prefix (64-bit operand)
    object.text_section.data.push(RexPrefix::RexW as u8);

    // LEA opcode
    object.text_section.data.push(Instruction::Lea as u8);

    // ModR/M + displacement
    if stack_offset >= DISP8_MIN && stack_offset <= DISP8_MAX {
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

pub fn compile_load_instruction(
    load_instruction: &LoadInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::compile::linux_amd64::function::VariableLocation;
    use crate::platforms::amd64::{
        addressing::{modrm_rbp_disp32, sib_rbp_no_index},
        instruction::Instruction,
        register::Register,
        rex::RexPrefix,
    };

    // Step 1: Look up the pointer variable
    let ptr_name = &load_instruction.ptr.name;
    let ptr_location = context.get_variable(ptr_name).ok_or_else(|| {
        IRError::new(
            crate::ir::error::IRErrorKind::VariableNotFound,
            &format!("Pointer variable '{}' not found", ptr_name),
        )
    })?;

    match ptr_location {
        VariableLocation::Register(ptr_reg) => {
            // Pointer is in a register: mov rax, [ptr_reg]
            // Generate: REX.W + 0x8B + ModR/M

            // Determine REX prefix
            let needs_rex_b = ptr_reg.requires_rex();
            if needs_rex_b {
                object.text_section.data.push(RexPrefix::REX_WB);
            } else {
                object.text_section.data.push(RexPrefix::RexW as u8);
            }

            // MOV r64, r/m64 opcode (0x8B)
            object.text_section.data.push(Instruction::MovLoad as u8);

            // ModR/M byte for [ptr_reg]
            let modrm_bytes = generate_modrm_indirect(Register::RAX.number(), ptr_reg.number());
            object.text_section.data.extend_from_slice(&modrm_bytes);
        }

        VariableLocation::Stack(offset) => {
            // Pointer is on stack: Need two steps

            // Step 1: mov rax, [rbp + offset] - Load pointer address into RAX
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

            // Step 2: mov rax, [rax] - Dereference the pointer
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::MovLoad as u8);
            // ModR/M for [rax]: Mod=00, Reg=0(RAX), R/M=0(RAX)
            let modrm_rax_indirect = (MODRM_MOD_MEMORY_NO_DISP << MODRM_MOD_SHIFT)
                | (Register::RAX.number() << MODRM_REG_SHIFT)
                | Register::RAX.number();
            object.text_section.data.push(modrm_rax_indirect);
        }
    }

    Ok(())
}

pub fn compile_store_instruction(
    store_instruction: &StoreInstruction,
    context: &mut FunctionContext,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    use crate::ir::compile::linux_amd64::function::VariableLocation;
    use crate::platforms::amd64::{
        addressing::{modrm_rbp_disp32, sib_rbp_no_index},
        instruction::Instruction,
        register::{modrm_reg_reg, Register},
        rex::RexPrefix,
    };

    // Step 1: Load the value to store into RAX
    use crate::ir::ast::common::Operand;
    match &store_instruction.value {
        Operand::Literal(lit) => {
            use crate::ir::ast::common::literal::LiteralValue;
            match lit {
                LiteralValue::Int64(value) => {
                    // mov rax, immediate (64-bit)
                    object.text_section.data.push(RexPrefix::RexW as u8);
                    object
                        .text_section
                        .data
                        .push(Instruction::MOV_IMM64_BASE + Register::RAX.number());
                    object
                        .text_section
                        .data
                        .extend_from_slice(&value.to_le_bytes());
                }
                _ => {
                    return Err(IRError::new(
                        crate::ir::error::IRErrorKind::NotImplemented,
                        "Only Int64 literals supported for store instruction",
                    ));
                }
            }
        }
        Operand::Identifier(id) => {
            // Load from variable location
            let var_loc = context.get_variable(&id.name).ok_or_else(|| {
                IRError::new(
                    crate::ir::error::IRErrorKind::VariableNotFound,
                    &format!("Variable '{}' not found", id.name),
                )
            })?;

            match var_loc {
                VariableLocation::Register(src_reg) => {
                    if *src_reg != Register::RAX {
                        // mov rax, src_reg
                        if src_reg.requires_rex() {
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
                    // If already in RAX, do nothing
                }
                VariableLocation::Stack(offset) => {
                    // mov rax, [rbp + offset]
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
        }
    }

    // Step 2: Look up the pointer variable
    let ptr_name = &store_instruction.ptr.name;
    let ptr_location = context.get_variable(ptr_name).ok_or_else(|| {
        IRError::new(
            crate::ir::error::IRErrorKind::VariableNotFound,
            &format!("Pointer variable '{}' not found", ptr_name),
        )
    })?;

    // Step 3: Store RAX to the memory address
    match ptr_location {
        VariableLocation::Register(ptr_reg) => {
            // Pointer is in a register: mov [ptr_reg], rax
            // Generate: REX.W + 0x89 + ModR/M

            // Determine REX prefix
            let needs_rex_b = ptr_reg.requires_rex();
            if needs_rex_b {
                object.text_section.data.push(RexPrefix::REX_WB);
            } else {
                object.text_section.data.push(RexPrefix::RexW as u8);
            }

            // MOV r/m64, r64 opcode (0x89)
            object.text_section.data.push(Instruction::Mov as u8);

            // ModR/M byte for [ptr_reg]
            let modrm_bytes = generate_modrm_indirect(Register::RAX.number(), ptr_reg.number());
            object.text_section.data.extend_from_slice(&modrm_bytes);
        }

        VariableLocation::Stack(offset) => {
            // Pointer is on stack: Load pointer to RCX first, then store

            // Step 1: mov rcx, [rbp + offset] - Load pointer address into RCX
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::MovLoad as u8);
            object
                .text_section
                .data
                .push(modrm_rbp_disp32(Register::RCX.number()));
            object.text_section.data.push(sib_rbp_no_index());
            object
                .text_section
                .data
                .extend_from_slice(&offset.to_le_bytes());

            // Step 2: mov [rcx], rax - Store RAX to the address in RCX
            object.text_section.data.push(RexPrefix::RexW as u8);
            object.text_section.data.push(Instruction::Mov as u8);
            // ModR/M for [rcx]: Mod=00, Reg=0(RAX), R/M=1(RCX)
            let modrm_rcx_indirect = (MODRM_MOD_MEMORY_NO_DISP << MODRM_MOD_SHIFT)
                | (Register::RAX.number() << MODRM_REG_SHIFT)
                | Register::RCX.number();
            object.text_section.data.push(modrm_rcx_indirect);
        }
    }

    Ok(())
}
