use crate::{
    ir::{ast::global::function::FunctionDefinition, error::IRError},
    platforms::{
        amd64::{
            instruction::Instruction,
            register::{modrm_reg_reg, Register},
            rex::RexPrefix,
        },
        linux::elf::{
            object::ELFObject,
            section::SectionType,
            symbol::{Symbol, SymbolBinding, SymbolType},
        },
    },
};

use super::instruction;

pub fn compile_function(
    function: &FunctionDefinition,
    object: &mut ELFObject,
) -> Result<(), IRError> {
    let function_start_offset = object.text_section.data.len();

    // Function prologue 생성
    // push rbp (스택 프레임 저장)
    object
        .text_section
        .data
        .push(Instruction::Push as u8 + Register::RBP.number());

    // mov rbp, rsp (새로운 스택 프레임 설정)
    object.text_section.data.push(RexPrefix::RexW as u8);
    object.text_section.data.push(Instruction::Mov as u8);
    object
        .text_section
        .data
        .push(modrm_reg_reg(Register::RSP, Register::RBP));

    // LocalStatements 컴파일
    instruction::compile_statements(&function.function_body.statements, object)?;

    // Function epilogue는 instruction.rs에서 sys_exit을 호출하므로 불필요

    let function_end_offset = object.text_section.data.len();
    let function_size = function_end_offset - function_start_offset;

    // Function symbol을 symbol table에 추가
    object.symbol_table.add_symbol(Symbol {
        name: function.function_name.clone(),
        section: SectionType::Text,
        offset: function_start_offset,
        size: function_size,
        symbol_type: SymbolType::Function,
        binding: SymbolBinding::Global,
    });

    Ok(())
}
