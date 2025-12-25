use crate::ir::data::{
    relocation::Relocation,
    section::{LinkedSection, Section},
    symbol::SymbolTable,
};

/// 컴파일된 오브젝트 파일 (ELF/PE 형식의 .o 파일과 유사)
#[derive(Debug, Clone)]
pub struct IRCompileObject {
    /// 데이터 섹션 (.data) - 초기화된 변경 가능한 전역 변수
    pub data_section: Section,

    /// 읽기 전용 데이터 섹션 (.rodata) - 상수
    pub rodata_section: Section,

    /// BSS 섹션 (.bss) - 0으로 초기화된 전역 변수
    pub bss_section: Section,

    /// 텍스트/코드 섹션 (.text) - 실행 가능한 기계어 코드
    pub text_section: Section,

    /// 심볼 테이블 - 전역 심볼과 주소 매핑
    pub symbol_table: SymbolTable,

    /// 재배치 정보 - 링킹 시 주소 패치가 필요한 위치
    pub relocations: Vec<Relocation>,
}

impl IRCompileObject {
    pub fn new() -> Self {
        Self {
            data_section: Section::new_data(),
            rodata_section: Section::new_rodata(),
            bss_section: Section::new_bss(),
            text_section: Section::new_text(),
            symbol_table: SymbolTable::new(),
            relocations: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct IRLinkObject {
    /// 최종 바이너리 데이터
    pub binary: Vec<u8>,

    /// 엔트리 포인트 주소
    pub entry_point: u64,

    /// 섹션들의 최종 메모리 배치
    pub sections: Vec<LinkedSection>,
}
