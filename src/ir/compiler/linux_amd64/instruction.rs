use crate::{
    ir::{ast::local::LocalStatement, error::IRError},
    platforms::{
        amd64::{
            instruction::Instruction,
            modrm::{LEA_RSI_RIP_REL, XOR_RDI_RDI},
            register::{modrm_digit_reg, Register},
            rex::RexPrefix,
        },
        linux::{
            elf::{
                object::ELFObject,
                relocation::{Relocation, RelocationType},
                section::SectionType,
            },
            fd::STDOUT,
            syscall::amd64::{SYS_EXIT, SYS_WRITE},
        },
    },
};

pub fn compile_statements(
    statements: &[LocalStatement],
    object: &mut ELFObject,
) -> Result<(), IRError> {
    for statement in statements {
        compile_statement(statement, object)?;
    }
    Ok(())
}

fn compile_statement(stmt: &LocalStatement, object: &mut ELFObject) -> Result<(), IRError> {
    match stmt {
        LocalStatement::Instruction(instr_stmt) => {
            use crate::ir::ast::local::instruction::InstructionStatement;
            match instr_stmt {
                InstructionStatement::Call(call_instr) => {
                    compile_call_instruction(call_instr, object)?;
                }
                InstructionStatement::Add(_) => {
                    return Err(IRError::new("Add instruction not yet implemented"));
                }
            }
        }
        LocalStatement::Assignment(_) => {
            return Err(IRError::new("Assignment statement not yet implemented"));
        }
        LocalStatement::Label(_) => {
            return Err(IRError::new("Label statement not yet implemented"));
        }
    }
    Ok(())
}

fn compile_call_instruction(
    call_instr: &crate::ir::ast::local::instruction::call::CallInstruction,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    // printf 호출을 sys_write + sys_exit syscall로 변환
    if call_instr.function_name.name == "printf" {
        compile_printf_as_syscall(object)?;
    } else {
        return Err(IRError::new(&format!(
            "Unsupported function call: {}",
            call_instr.function_name.name
        )));
    }
    Ok(())
}

fn compile_printf_as_syscall(object: &mut ELFObject) -> Result<(), IRError> {
    // HELLWORLD_TEXT 상수의 길이를 symbol table에서 조회
    let hello_symbol = object
        .symbol_table
        .symbols
        .iter()
        .find(|s| s.name == "HELLWORLD_TEXT")
        .ok_or_else(|| IRError::new("HELLWORLD_TEXT symbol not found"))?;

    // constant.rs가 null terminator를 추가하므로, 실제 문자열 길이는 size - 1
    let string_length = hello_symbol.size - 1;

    // lea rsi 명령어의 offset 위치 (relocation이 필요한 부분)
    // mov rax (7) + mov rdi (7) + lea opcode (3) = 17
    let lea_offset_position = object.text_section.data.len() + 17;

    // x86-64 기계어 코드 생성
    let mut machine_code = vec![
        // mov rax, 1 (SYS_WRITE)
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        modrm_digit_reg(0, Register::RAX),
        SYS_WRITE,
        0x00,
        0x00,
        0x00,
        // mov rdi, 1 (STDOUT)
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        modrm_digit_reg(0, Register::RDI),
        STDOUT,
        0x00,
        0x00,
        0x00,
        // lea rsi, [rip+offset] - HELLWORLD_TEXT 상수 참조 (재배치 필요)
        RexPrefix::RexW as u8,
        Instruction::Lea as u8,
        LEA_RSI_RIP_REL,
        0x00,
        0x00,
        0x00,
        0x00,
        // mov rdx, <string_length>
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        modrm_digit_reg(0, Register::RDX),
        string_length as u8,
        0x00,
        0x00,
        0x00,
        // syscall
        Instruction::SYSCALL_BYTES[0],
        Instruction::SYSCALL_BYTES[1],
        // mov rax, 60 (SYS_EXIT)
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        modrm_digit_reg(0, Register::RAX),
        SYS_EXIT,
        0x00,
        0x00,
        0x00,
        // xor rdi, rdi (exit code 0)
        RexPrefix::RexW as u8,
        Instruction::Xor as u8,
        XOR_RDI_RDI,
        // syscall
        Instruction::SYSCALL_BYTES[0],
        Instruction::SYSCALL_BYTES[1],
    ];

    // .text 섹션에 기계어 코드 추가
    object.text_section.data.append(&mut machine_code);

    // .rodata 섹션의 HELLWORLD_TEXT에 대한 재배치 정보 추가
    // lea rsi, [rip+offset] 명령어의 offset 부분을 패치해야 함
    object.relocations.push(Relocation {
        section: SectionType::Text,
        offset: lea_offset_position,
        symbol: "HELLWORLD_TEXT".to_string(),
        reloc_type: RelocationType::PcRel32,
        addend: 0, // PC-relative 계산 (RIP는 이미 offset 필드 끝을 가리킴)
    });

    Ok(())
}
