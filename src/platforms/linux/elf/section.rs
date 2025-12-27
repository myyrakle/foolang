/// 섹션 플래그 (읽기/쓰기/실행 권한)
#[derive(Debug, Clone)]
pub struct SectionFlags {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

/// 섹션 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionType {
    Text,
    Data,
    RoData,
    Bss,
}

/// 섹션 데이터
#[derive(Debug, Clone)]
pub struct Section {
    /// 섹션 이름
    pub name: String,

    /// 바이너리 데이터 (hex 바이트)
    pub data: Vec<u8>,

    /// 섹션의 가상 주소 (링킹 전에는 0)
    pub virtual_address: u64,

    /// 정렬 요구사항 (2^n 바이트)
    pub alignment: usize,

    /// 섹션 플래그 (읽기/쓰기/실행 권한)
    pub flags: SectionFlags,
}

impl Section {
    pub fn new(name: &str, flags: SectionFlags) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            virtual_address: 0,
            alignment: 8, // 기본 8바이트 정렬
            flags,
        }
    }

    pub fn new_text() -> Self {
        Self::new(
            ".text",
            SectionFlags {
                readable: true,
                writable: false,
                executable: true,
            },
        )
    }

    pub fn new_data() -> Self {
        Self::new(
            ".data",
            SectionFlags {
                readable: true,
                writable: true,
                executable: false,
            },
        )
    }

    pub fn new_rodata() -> Self {
        Self::new(
            ".rodata",
            SectionFlags {
                readable: true,
                writable: false,
                executable: false,
            },
        )
    }

    pub fn new_bss() -> Self {
        Self::new(
            ".bss",
            SectionFlags {
                readable: true,
                writable: true,
                executable: false,
            },
        )
    }
}

/// 링킹된 섹션 (최종 메모리 배치)
#[derive(Debug, Clone)]
pub struct LinkedSection {
    /// 섹션 이름
    pub name: String,

    /// 메모리 주소
    pub virtual_address: u64,

    /// 섹션 크기
    pub size: usize,

    /// 섹션 플래그
    pub flags: SectionFlags,
}
