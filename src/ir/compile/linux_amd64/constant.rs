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

    // 상수 타입에 따라 섹션 선택
    // - 문자열: .rodata (주소 참조용)
    // - 정수/부울: .data (값 로드용)
    let (section_type, offset, size) = match &constant.value {
        LiteralValue::Int8(val) => {
            let offset = object.data_section.data.len();
            object.data_section.data.push(*val as u8);
            (SectionType::Data, offset, 1)
        }
        LiteralValue::Int16(val) => {
            let offset = object.data_section.data.len();
            object
                .data_section
                .data
                .extend_from_slice(&val.to_le_bytes());
            (SectionType::Data, offset, 2)
        }
        LiteralValue::Int32(val) => {
            let offset = object.data_section.data.len();
            object
                .data_section
                .data
                .extend_from_slice(&val.to_le_bytes());
            (SectionType::Data, offset, 4)
        }
        LiteralValue::Int64(val) => {
            let offset = object.data_section.data.len();
            object
                .data_section
                .data
                .extend_from_slice(&val.to_le_bytes());
            (SectionType::Data, offset, 8)
        }
        LiteralValue::Float64(val) => {
            let offset = object.data_section.data.len();
            object
                .data_section
                .data
                .extend_from_slice(&val.to_le_bytes());
            (SectionType::Data, offset, 8)
        }
        LiteralValue::Boolean(val) => {
            let offset = object.data_section.data.len();
            object.data_section.data.push(if *val { 1 } else { 0 });
            (SectionType::Data, offset, 1)
        }
        LiteralValue::String(s) => {
            let offset = object.rodata_section.data.len();
            object.rodata_section.data.extend_from_slice(s.as_bytes());
            object.rodata_section.data.push(0); // null terminator
            (SectionType::RoData, offset, s.len() + 1)
        }
    };

    // 심볼 테이블에 등록
    object.symbol_table.add_symbol(Symbol {
        name: constant.constant_name.name.clone(),
        section: section_type,
        offset,
        size,
        symbol_type: SymbolType::Object,
        binding: SymbolBinding::Global,
    });

    Ok(())
}
