use std::fs;

use crate::{
    ir::data::object::IRCompileObject,
    platforms::amd64::{Instruction, RexPrefix},
};

pub mod ir;
pub mod platforms;

pub fn main() {
    let mut object = IRCompileObject::new();

    // Hello World 문자열을 .rodata 섹션에 추가
    let hello_str = b"Hello, World!\n";
    object.rodata_section.data.extend_from_slice(hello_str);

    // 심볼 테이블에 문자열 심볼 추가
    object.symbol_table.add_symbol(ir::data::symbol::Symbol {
        name: "hello_msg".to_string(),
        section: ir::data::section::SectionType::RoData,
        offset: 0,
        size: hello_str.len(),
        symbol_type: ir::data::symbol::SymbolType::Object,
        binding: ir::data::symbol::SymbolBinding::Local,
    });

    // x86-64 리눅스에서 write 시스템콜로 Hello World 출력하는 기계어 코드
    // mov rax, 1        ; sys_write
    // mov rdi, 1        ; stdout
    // lea rsi, [rel hello_msg]  ; 문자열 주소
    // mov rdx, 14       ; 길이
    // syscall
    // mov rax, 60       ; sys_exit
    // xor rdi, rdi      ; exit code 0
    // syscall
    let machine_code = vec![
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        0xc0,
        0x01,
        0x00,
        0x00,
        0x00, // mov rax, 1
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        0xc7,
        0x01,
        0x00,
        0x00,
        0x00, // mov rdi, 1
        RexPrefix::RexW as u8,
        Instruction::Lea as u8,
        0x35,
        0x00,
        0x00,
        0x00,
        0x00, // lea rsi, [rip+0] (재배치 필요)
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        0xc2,
        0x0e,
        0x00,
        0x00,
        0x00, // mov rdx, 14
        0x0f,
        0x05, // syscall
        RexPrefix::RexW as u8,
        Instruction::MovImm as u8,
        0xc0,
        0x3c,
        0x00,
        0x00,
        0x00, // mov rax, 60
        RexPrefix::RexW as u8,
        Instruction::Xor as u8,
        0xff, // xor rdi, rdi
        0x0f,
        0x05, // syscall
    ];

    object.text_section.data = machine_code;

    // _start 함수 심볼 추가 (엔트리 포인트)
    object.symbol_table.add_symbol(ir::data::symbol::Symbol {
        name: "_start".to_string(),
        section: ir::data::section::SectionType::Text,
        offset: 0,
        size: object.text_section.data.len(),
        symbol_type: ir::data::symbol::SymbolType::Function,
        binding: ir::data::symbol::SymbolBinding::Global,
    });

    // .rodata 섹션의 hello_msg에 대한 재배치 정보 추가
    // lea rsi, [rip+offset] 명령어의 오프셋 부분(바이트 17-20)을 패치해야 함
    object.relocations.push(ir::data::relocation::Relocation {
        section: ir::data::section::SectionType::Text,
        offset: 17, // lea 명령어의 오프셋 필드 위치
        symbol: "hello_msg".to_string(),
        reloc_type: ir::data::relocation::RelocationType::PcRel32,
        addend: -4, // PC-relative 계산 조정
    });

    let bytes = object.to_elf_binary();

    fs::write("output.o", bytes).expect("Failed to write ELF object file");

    println!("ELF object file created: output.o");
    println!("To create executable, run: ld output.o -o hello");
    println!("To run: ./hello");
}
