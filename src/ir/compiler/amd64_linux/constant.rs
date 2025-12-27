use crate::platforms::linux::elf::{
    object::ELFObject,
    section::SectionType,
    symbol::{Symbol, SymbolBinding, SymbolType},
};

/// AMD64/x86-64 아키텍처에서 상수 컴파일
pub fn compile_constant(
    constant: &crate::ir::ast::global::constant::ConstantDefinition,
    object: &mut ELFObject,
) -> Result<(), crate::ir::error::IRError> {
    use crate::ir::ast::common::literal::LiteralValue;

    // 상수는 .rodata 섹션에 배치
    let offset = object.rodata_section.data.len();
    let size_before = object.rodata_section.data.len();

    // 리틀 엔디안으로 바이너리 데이터 추가
    match &constant.value {
        LiteralValue::Int64(val) => {
            // 64비트 정수를 8바이트 리틀 엔디안으로 변환
            object
                .rodata_section
                .data
                .extend_from_slice(&val.to_le_bytes());
        }
        LiteralValue::Float64(val) => {
            // 64비트 부동소수점을 8바이트 리틀 엔디안으로 변환
            object
                .rodata_section
                .data
                .extend_from_slice(&val.to_le_bytes());
        }
        LiteralValue::Boolean(val) => {
            // 불리언을 1바이트로 저장 (0 또는 1)
            object.rodata_section.data.push(if *val { 1 } else { 0 });
        }
        LiteralValue::String(s) => {
            // 문자열을 UTF-8 바이트로 저장 (null-terminated)
            object.rodata_section.data.extend_from_slice(s.as_bytes());
            object.rodata_section.data.push(0); // null terminator
        }
    }

    let size = object.rodata_section.data.len() - size_before;

    // 심볼 테이블에 등록
    object.symbol_table.add_symbol(Symbol {
        name: constant.constant_name.name.clone(),
        section: SectionType::RoData,
        offset,
        size,
        symbol_type: SymbolType::Object,
        binding: SymbolBinding::Global,
    });

    Ok(())
}
