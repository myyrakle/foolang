/// ELF 식별자 정보 (e_ident 필드)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ELFIdent {
    /// ELF 클래스 (32-bit or 64-bit)
    pub class: ELFClass,

    /// 데이터 인코딩 (Little Endian or Big Endian)
    pub data: ELFData,

    /// ELF 버전
    pub version: u8,

    /// OS/ABI 식별
    pub osabi: ELFOSABI,

    /// ABI 버전
    pub abiversion: u8,
}

impl ELFIdent {
    /// 64-bit Linux ELF 기본값
    pub fn elf64_linux() -> Self {
        Self {
            class: ELFClass::ELF64,
            data: ELFData::LittleEndian,
            version: 1,
            osabi: ELFOSABI::SystemV,
            abiversion: 0,
        }
    }

    /// e_ident 바이트 배열 생성 (16바이트)
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut ident = [0u8; 16];

        // ELF 매직 넘버
        ident[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);

        // 클래스
        ident[4] = self.class as u8;

        // 데이터 인코딩
        ident[5] = self.data as u8;

        // 버전
        ident[6] = self.version;

        // OS/ABI
        ident[7] = self.osabi as u8;

        // ABI 버전
        ident[8] = self.abiversion;

        // 나머지는 패딩 (0으로 채워짐)

        ident
    }
}

/// ELF 클래스 (32-bit or 64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ELFClass {
    /// 32-bit ELF
    ELF32 = 1,

    /// 64-bit ELF
    ELF64 = 2,
}

/// 데이터 인코딩 (엔디안)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ELFData {
    /// Little Endian
    LittleEndian = 1,

    /// Big Endian
    BigEndian = 2,
}

/// OS/ABI 식별자
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ELFOSABI {
    /// UNIX System V ABI
    SystemV = 0,

    /// HP-UX
    HPUX = 1,

    /// NetBSD
    NetBSD = 2,

    /// Linux (GNU extensions)
    Linux = 3,

    /// Solaris
    Solaris = 6,

    /// FreeBSD
    FreeBSD = 9,
}

/// ELF 파일 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ELFType {
    /// 알 수 없음
    None = 0,

    /// 재배치 가능한 파일 (.o) - ET_REL
    Relocatable = 1,

    /// 고정 주소 실행 파일 - ET_EXEC (레거시, 보안 취약)
    Executable = 2,

    /// Position Independent Executable / 공유 라이브러리 - ET_DYN
    /// PIE 실행 파일 또는 .so 파일
    Dynamic = 3,

    /// 코어 덤프 파일 - ET_CORE
    Core = 4,
}

/// ELF 머신 아키텍처
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ELFMachine {
    /// No machine
    None = 0,

    /// AT&T WE 32100
    M32 = 1,

    /// SPARC
    SPARC = 2,

    /// Intel 80386
    I386 = 3,

    /// Motorola 68000
    M68K = 4,

    /// Motorola 88000
    M88K = 5,

    /// Intel 80860
    I860 = 7,

    /// MIPS RS3000
    MIPS = 8,

    /// ARM
    ARM = 40,

    /// Intel x86-64 (AMD64)
    X86_64 = 62,

    /// ARM 64-bit (AArch64)
    AARCH64 = 183,

    /// RISC-V
    RISCV = 243,
}

/// ELF 헤더 (64-bit)
#[derive(Debug, Clone)]
pub struct ELFHeader64 {
    /// ELF 식별자 정보
    pub ident: ELFIdent,

    /// 파일 타입
    pub file_type: ELFType,

    /// 머신 아키텍처
    pub machine: ELFMachine,

    /// ELF 버전
    pub version: u32,

    /// 엔트리 포인트 주소 (실행 파일만)
    pub entry: u64,

    /// Program Header 테이블 오프셋
    pub phoff: u64,

    /// Section Header 테이블 오프셋
    pub shoff: u64,

    /// 프로세서별 플래그
    pub flags: u32,

    /// ELF 헤더 크기
    pub ehsize: u16,

    /// Program Header 엔트리 크기
    pub phentsize: u16,

    /// Program Header 엔트리 개수
    pub phnum: u16,

    /// Section Header 엔트리 크기
    pub shentsize: u16,

    /// Section Header 엔트리 개수
    pub shnum: u16,

    /// Section Header 문자열 테이블 인덱스
    pub shstrndx: u16,
}

impl ELFHeader64 {
    /// 재배치 가능한 오브젝트 파일(.o) 기본 헤더
    pub fn relocatable_x86_64() -> Self {
        Self {
            ident: ELFIdent::elf64_linux(),
            file_type: ELFType::Relocatable,
            machine: ELFMachine::X86_64,
            version: 1,
            entry: 0,
            phoff: 0,
            shoff: 0, // 나중에 패치됨
            flags: 0,
            ehsize: 64,
            phentsize: 0,
            phnum: 0,
            shentsize: 64,
            shnum: 9, // null, text, rodata, data, bss, symtab, strtab, rela.text, shstrtab
            shstrndx: 8, // .shstrtab 섹션 인덱스
        }
    }

    /// PIE 실행 파일 기본 헤더 (ET_DYN)
    pub fn executable_x86_64(entry_point: u64) -> Self {
        Self {
            ident: ELFIdent::elf64_linux(),
            file_type: ELFType::Dynamic, // PIE uses ET_DYN
            machine: ELFMachine::X86_64,
            version: 1,
            entry: entry_point,
            phoff: 64, // ELF 헤더 직후
            shoff: 0,  // 나중에 패치됨
            flags: 0,
            ehsize: 64,
            phentsize: 56, // Program Header 크기 (64-bit)
            phnum: 2,      // .text + .rodata
            shentsize: 64, // Section Header 크기 (64-bit)
            shnum: 6,      // null, .text, .rodata, .symtab, .strtab, .shstrtab
            shstrndx: 5,   // .shstrtab 섹션 인덱스
        }
    }

    /// ELF 헤더를 바이트 배열로 변환 (64바이트)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);

        // e_ident (16 bytes)
        bytes.extend_from_slice(&self.ident.to_bytes());

        // e_type (2 bytes)
        bytes.extend_from_slice(&(self.file_type as u16).to_le_bytes());

        // e_machine (2 bytes)
        bytes.extend_from_slice(&(self.machine as u16).to_le_bytes());

        // e_version (4 bytes)
        bytes.extend_from_slice(&self.version.to_le_bytes());

        // e_entry (8 bytes)
        bytes.extend_from_slice(&self.entry.to_le_bytes());

        // e_phoff (8 bytes)
        bytes.extend_from_slice(&self.phoff.to_le_bytes());

        // e_shoff (8 bytes)
        bytes.extend_from_slice(&self.shoff.to_le_bytes());

        // e_flags (4 bytes)
        bytes.extend_from_slice(&self.flags.to_le_bytes());

        // e_ehsize (2 bytes)
        bytes.extend_from_slice(&self.ehsize.to_le_bytes());

        // e_phentsize (2 bytes)
        bytes.extend_from_slice(&self.phentsize.to_le_bytes());

        // e_phnum (2 bytes)
        bytes.extend_from_slice(&self.phnum.to_le_bytes());

        // e_shentsize (2 bytes)
        bytes.extend_from_slice(&self.shentsize.to_le_bytes());

        // e_shnum (2 bytes)
        bytes.extend_from_slice(&self.shnum.to_le_bytes());

        // e_shstrndx (2 bytes)
        bytes.extend_from_slice(&self.shstrndx.to_le_bytes());

        bytes
    }

    /// Section Header 오프셋 패치
    pub fn patch_shoff(&self, bytes: &mut [u8], shoff: u64) {
        // e_shoff는 40번째 바이트부터 8바이트
        const SHOFF_OFFSET: usize = 40;
        bytes[SHOFF_OFFSET..SHOFF_OFFSET + 8].copy_from_slice(&shoff.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_ident_size() {
        let ident = ELFIdent::elf64_linux();
        let bytes = ident.to_bytes();
        assert_eq!(bytes.len(), 16);

        // 매직 넘버 확인
        assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);

        // 64-bit 클래스
        assert_eq!(bytes[4], 2);

        // Little Endian
        assert_eq!(bytes[5], 1);
    }

    #[test]
    fn test_elf_header_size() {
        let header = ELFHeader64::relocatable_x86_64();
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn test_relocatable_header() {
        let header = ELFHeader64::relocatable_x86_64();
        assert_eq!(header.file_type as u16, 1); // ET_REL
        assert_eq!(header.machine as u16, 62);  // x86-64
        assert_eq!(header.entry, 0);            // No entry point
    }

    #[test]
    fn test_executable_header() {
        let header = ELFHeader64::executable_x86_64(0x401000);
        assert_eq!(header.file_type as u16, 3);   // ET_DYN (PIE executable)
        assert_eq!(header.entry, 0x401000);        // Entry point
        assert_eq!(header.phoff, 64);              // Program headers after ELF header
        assert_eq!(header.phnum, 2);               // 2 segments
    }
}
