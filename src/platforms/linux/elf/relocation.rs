use super::section::SectionType;

/// 재배치 정보 (링킹 시 주소 패치)
#[derive(Debug, Clone)]
pub struct Relocation {
    /// 패치할 위치의 섹션
    pub section: SectionType,

    /// 섹션 내 오프셋
    pub offset: usize,

    /// 참조하는 심볼
    pub symbol: String,

    /// 재배치 타입 (AMD64 기준)
    pub reloc_type: RelocationType,

    /// 추가 상수 (addend)
    pub addend: i64,
}

/// 재배치 타입 (AMD64/x86-64 기준)
#[derive(Debug, Clone)]
pub enum RelocationType {
    /// 절대 64비트 주소
    Abs64,

    /// PC-relative 32비트 오프셋
    PcRel32,

    /// 32비트 절대 주소
    Abs32,

    /// PLT를 통한 함수 호출
    PltPcRel32,

    /// GOT 엔트리 접근
    GotPcRel,
}
