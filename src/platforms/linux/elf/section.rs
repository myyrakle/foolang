/// ELF 섹션 헤더 타입 (sh_type)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SectionHeaderType {
    /// NULL section
    Null = 0,
    /// Program data
    ProgBits = 1,
    /// Symbol table
    SymTab = 2,
    /// String table
    StrTab = 3,
    /// Relocation entries with addends
    Rela = 4,
    /// Symbol hash table
    Hash = 5,
    /// Dynamic linking information
    Dynamic = 6,
    /// Notes
    Note = 7,
    /// No space in file (BSS)
    NoBits = 8,
}

/// ELF 섹션 플래그 (sh_flags)
pub mod section_flags {
    /// Writable
    pub const SHF_WRITE: u64 = 0x1;
    /// Occupies memory during execution
    pub const SHF_ALLOC: u64 = 0x2;
    /// Executable
    pub const SHF_EXECINSTR: u64 = 0x4;
    /// Info link field
    pub const SHF_INFO_LINK: u64 = 0x40;
}

/// 섹션 플래그 (읽기/쓰기/실행 권한)
#[derive(Debug, Clone)]
pub struct SectionFlags {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl SectionFlags {
    /// ELF 섹션 플래그 비트 값으로 변환
    ///
    /// ELF 섹션 플래그 규칙:
    /// - readable이 true인 경우: SHF_ALLOC 설정 (메모리에 로드, 읽기 가능)
    /// - readable이 false인 경우: 플래그 없음 (디버그 정보 등 비할당 섹션)
    /// - writable이 true인 경우: SHF_WRITE 추가
    /// - executable이 true인 경우: SHF_EXECINSTR 추가
    ///
    /// 참고: ELF에는 명시적인 "읽기" 플래그가 없습니다.
    /// 읽기는 SHF_ALLOC이 있으면 기본으로 가능합니다.
    pub fn to_elf_flags(&self) -> u64 {
        let mut flags = 0;

        // readable이 true면 메모리에 로드 (읽기 가능)
        if self.readable {
            flags |= section_flags::SHF_ALLOC;
        }

        if self.writable {
            flags |= section_flags::SHF_WRITE;
        }

        if self.executable {
            flags |= section_flags::SHF_EXECINSTR;
        }

        flags
    }
}

/// 섹션 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionType {
    Text,
    Data,
    RoData,
    Bss,
    /// UNDEFINED - 외부 심볼용
    Undefined,
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
