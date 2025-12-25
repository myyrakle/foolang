pub fn compile_constant_definition(
    constant: &crate::ir::ast::global::constant::ConstantDefinition,
    object: &mut crate::ir::IRCompileObject,
) -> Result<(), crate::ir::error::IRError> {
    if cfg!(target_arch = "x86_64") {
        compile_constant_amd64(constant, object)?;
    } else if cfg!(target_arch = "aarch64") {
        unimplemented!("Constant compilation for aarch64 is not yet implemented");
    } else {
        unimplemented!(
            "Constant compilation not implemented for this architecture: {}",
            std::env::consts::ARCH
        );
    }

    Ok(())
}

/// AMD64/x86-64 아키텍처에서 상수 컴파일
fn compile_constant_amd64(
    constant: &crate::ir::ast::global::constant::ConstantDefinition,
    object: &mut crate::ir::IRCompileObject,
) -> Result<(), crate::ir::error::IRError> {
    use crate::ir::ast::common::literal::LiteralValue;
    use crate::ir::{SectionType, Symbol, SymbolBinding, SymbolType};

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
