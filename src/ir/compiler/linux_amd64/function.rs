use crate::{
    ir::{ast::global::function::FunctionDefinition, error::IRError},
    platforms::{
        amd64::{
            register::{modrm_digit_reg, Register},
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
    // push rbp
    object.text_section.data.push(0x55);
    // mov rbp, rsp
    object.text_section.data.extend_from_slice(&[
        RexPrefix::RexW as u8,
        0x89,                              // mov r/m64, r64
        modrm_digit_reg(3, Register::RBP), // ModR/M: mod=11(register), reg=RSP, r/m=RBP
    ]);

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
