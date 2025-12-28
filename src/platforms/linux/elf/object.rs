use super::{
    header::ELFHeader64,
    relocation::{self, Relocation},
    section::{self, LinkedSection, Section, SectionHeaderType},
    symbol::{self, SymbolTable},
};

/// ELF 출력 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ELFOutputType {
    /// 재배치 가능한 오브젝트 파일 (.o)
    Relocatable,
    /// 실행 파일
    Executable,
}

/// ELF 섹션 인덱스 상수
#[allow(dead_code)]
mod section_indices {
    pub const NULL: usize = 0;
    pub const TEXT: usize = 1;
    pub const RODATA: usize = 2;
    pub const DATA: usize = 3;
    pub const BSS: usize = 4;
    pub const SYMTAB: usize = 5;
    pub const STRTAB: usize = 6;
    pub const RELA_TEXT: usize = 7;
    pub const SHSTRTAB: usize = 8;
}

/// .shstrtab 섹션 내의 섹션 이름 문자열 오프셋
#[allow(dead_code)]
mod section_name_offsets {
    pub const NULL: u32 = 0;
    pub const TEXT: u32 = 1; // ".text\0" (6 bytes)
    pub const RODATA: u32 = 7; // ".rodata\0" (8 bytes)
    pub const DATA: u32 = 15; // ".data\0" (6 bytes)
    pub const BSS: u32 = 21; // ".bss\0" (5 bytes)
    pub const SYMTAB: u32 = 26; // ".symtab\0" (8 bytes)
    pub const STRTAB: u32 = 34; // ".strtab\0" (8 bytes)
    pub const RELA_TEXT: u32 = 42; // ".rela.text\0" (11 bytes)
    pub const SHSTRTAB: u32 = 53; // ".shstrtab\0" (10 bytes)
}

/// ELF 상수
mod elf_constants {
    /// sizeof(Elf64_Sym) - 심볼 테이블 엔트리 크기
    pub const SIZEOF_ELF64_SYM: u64 = 24;
    /// sizeof(Elf64_Rela) - 재배치 엔트리 크기
    pub const SIZEOF_ELF64_RELA: u64 = 24;

    /// PIE 실행 파일 기본 로드 주소 (0 = 로더가 결정)
    pub const BASE_ADDR: u64 = 0x0;
    /// 페이지 크기 (4KB)
    pub const PAGE_SIZE: u64 = 0x1000;
}

/// Program Header 타입 (p_type)
#[allow(dead_code)]
mod program_header_type {
    pub const PT_NULL: u32 = 0;
    pub const PT_LOAD: u32 = 1;
    pub const PT_DYNAMIC: u32 = 2;
    pub const PT_INTERP: u32 = 3;
    pub const PT_NOTE: u32 = 4;
}

/// Program Header 플래그 (p_flags)
#[allow(dead_code)]
mod program_header_flags {
    pub const PF_X: u32 = 0x1; // Execute
    pub const PF_W: u32 = 0x2; // Write
    pub const PF_R: u32 = 0x4; // Read
    pub const PF_RX: u32 = PF_R | PF_X; // Read + Execute
    pub const PF_RW: u32 = PF_R | PF_W; // Read + Write
}

#[derive(Debug, Clone)]
pub enum ELFObjectType {
    Relocatable,
    Executable,
    Shared,
}

/// 컴파일된 오브젝트 파일
/// Unix/Linux: ELF
#[derive(Debug, Clone)]
pub struct ELFObject {
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

impl Default for ELFObject {
    fn default() -> Self {
        Self::new()
    }
}

impl ELFObject {
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

impl ELFObject {
    /// ELF 바이너리 인코딩 (통합 함수)
    pub fn encode(&self, output_type: ELFOutputType) -> Vec<u8> {
        match output_type {
            ELFOutputType::Relocatable => self.encode_relocatable(),
            ELFOutputType::Executable => self.encode_executable(),
        }
    }

    fn encode_relocatable(&self) -> Vec<u8> {
        let mut binary = Vec::new();

        // ELF Header (64-bit)
        self.write_elf_header(&mut binary);

        // 섹션 데이터 작성 (실제 내용)

        let text_offset = binary.len();
        binary.extend_from_slice(&self.text_section.data);
        let text_size = self.text_section.data.len();

        let rodata_offset = binary.len();
        binary.extend_from_slice(&self.rodata_section.data);
        let rodata_size = self.rodata_section.data.len();

        let data_offset = binary.len();
        binary.extend_from_slice(&self.data_section.data);
        let data_size = self.data_section.data.len();

        // BSS는 실제 데이터 없음 (0으로 초기화)
        let bss_size = self.bss_section.data.len();

        // 문자열 테이블 생성 (.strtab)
        let strtab_offset = binary.len();
        let (strtab_data, string_offsets) = self.build_string_table();
        binary.extend_from_slice(&strtab_data);
        let strtab_size = strtab_data.len();

        // 섹션 이름 문자열 테이블 (.shstrtab)
        let shstrtab_offset = binary.len();
        let shstrtab_data = self.build_section_string_table();
        binary.extend_from_slice(&shstrtab_data);
        let shstrtab_size = shstrtab_data.len();

        // 심볼 테이블 (.symtab)
        let symtab_offset = binary.len();
        let symtab_data = self.build_symbol_table(&string_offsets);
        binary.extend_from_slice(&symtab_data);
        let symtab_size = symtab_data.len();

        // 재배치 테이블 (.rela.text)
        let rela_text_offset = binary.len();
        let rela_text_data = self.build_relocation_table();
        binary.extend_from_slice(&rela_text_data);
        let rela_text_size = rela_text_data.len();

        // Section Headers 작성
        let section_headers_start = binary.len();

        // 0: NULL section
        self.write_null_section_header(&mut binary);

        // 1: .text (SHT_PROGBITS, flags=AX)
        self.write_section_header(
            &mut binary,
            section_name_offsets::TEXT,
            SectionHeaderType::ProgBits as u32,
            self.text_section.flags.to_elf_flags(),
            text_offset,
            text_size,
            16,
            0,
            0,
            0,
        );

        // 2: .rodata (SHT_PROGBITS, flags=A)
        self.write_section_header(
            &mut binary,
            section_name_offsets::RODATA,
            SectionHeaderType::ProgBits as u32,
            self.rodata_section.flags.to_elf_flags(),
            rodata_offset,
            rodata_size,
            1,
            0,
            0,
            0,
        );

        // 3: .data (SHT_PROGBITS, flags=WA)
        self.write_section_header(
            &mut binary,
            section_name_offsets::DATA,
            SectionHeaderType::ProgBits as u32,
            self.data_section.flags.to_elf_flags(),
            data_offset,
            data_size,
            8,
            0,
            0,
            0,
        );

        // 4: .bss (SHT_NOBITS, flags=WA)
        self.write_section_header(
            &mut binary,
            section_name_offsets::BSS,
            SectionHeaderType::NoBits as u32,
            self.bss_section.flags.to_elf_flags(),
            0,
            bss_size,
            8,
            0,
            0,
            0,
        );

        // 5: .symtab (SHT_SYMTAB, link=strtab section, info=first non-local symbol index)
        // link: 6 (.strtab), info: 2 (첫 번째 글로벌 심볼의 인덱스)
        self.write_section_header(
            &mut binary,
            section_name_offsets::SYMTAB,
            SectionHeaderType::SymTab as u32,
            0,
            symtab_offset,
            symtab_size,
            8,
            section_indices::STRTAB as u32,
            2,
            elf_constants::SIZEOF_ELF64_SYM,
        );

        // 6: .strtab (SHT_STRTAB)
        self.write_section_header(
            &mut binary,
            section_name_offsets::STRTAB,
            SectionHeaderType::StrTab as u32,
            0,
            strtab_offset,
            strtab_size,
            1,
            0,
            0,
            0,
        );

        // 7: .rela.text (SHT_RELA, link=symtab, info=.text section index)
        // link: 5 (.symtab), info: 1 (.text), entsize: 24 (sizeof(Elf64_Rela))
        self.write_section_header(
            &mut binary,
            section_name_offsets::RELA_TEXT,
            SectionHeaderType::Rela as u32,
            section::section_flags::SHF_INFO_LINK,
            rela_text_offset,
            rela_text_size,
            8,
            section_indices::SYMTAB as u32,
            section_indices::TEXT as u32,
            elf_constants::SIZEOF_ELF64_RELA,
        );

        // 8: .shstrtab (SHT_STRTAB)
        self.write_section_header(
            &mut binary,
            section_name_offsets::SHSTRTAB,
            SectionHeaderType::StrTab as u32,
            0,
            shstrtab_offset,
            shstrtab_size,
            1,
            0,
            0,
            0,
        );

        // ELF Header의 섹션 헤더 오프셋 업데이트
        let section_headers_offset = section_headers_start as u64;
        self.patch_elf_header(&mut binary, section_headers_offset);

        binary
    }

    fn encode_executable(&self) -> Vec<u8> {
        let mut binary = Vec::new();

        // PIE 메모리 주소 설정 (상대 주소, 로더가 실제 주소 결정)
        let text_addr = elf_constants::BASE_ADDR + elf_constants::PAGE_SIZE; // .text at 0x1000
        let rodata_addr = elf_constants::BASE_ADDR + (2 * elf_constants::PAGE_SIZE); // .rodata at 0x2000

        // ELF Header (64-bit PIE executable, ET_DYN)
        self.write_executable_elf_header(&mut binary, text_addr);

        // Program Headers (LOAD 세그먼트)
        // PT_LOAD for .text (executable)
        let text_file_offset = elf_constants::PAGE_SIZE as usize; // 파일 오프셋
        let text_size = self.text_section.data.len();
        self.write_program_header(
            &mut binary,
            program_header_type::PT_LOAD,
            program_header_flags::PF_RX, // PF_R | PF_X (읽기+실행)
            text_file_offset,
            text_addr,
            text_addr,
            text_size as u64,
            text_size as u64,
            elf_constants::PAGE_SIZE, // 페이지 정렬
        );

        // PT_LOAD for .rodata (read-only)
        let rodata_file_offset = (2 * elf_constants::PAGE_SIZE) as usize;
        let rodata_size = self.rodata_section.data.len();
        self.write_program_header(
            &mut binary,
            program_header_type::PT_LOAD,
            program_header_flags::PF_R, // PF_R (읽기 전용)
            rodata_file_offset,
            rodata_addr,
            rodata_addr,
            rodata_size as u64,
            rodata_size as u64,
            elf_constants::PAGE_SIZE,
        );

        // 패딩으로 파일 오프셋까지 채우기
        while binary.len() < text_file_offset {
            binary.push(0);
        }

        // .text 섹션 데이터 (재배치 적용)
        let mut text_data = self.text_section.data.clone();

        // 재배치 처리: PC-relative 주소 계산
        for reloc in &self.relocations {
            if reloc.section == section::SectionType::Text {
                if let relocation::RelocationType::PcRel32 = reloc.reloc_type {
                    // 심볼 테이블에서 참조 대상 심볼 찾기
                    let symbol = self
                        .symbol_table
                        .symbols
                        .iter()
                        .find(|s| s.name == reloc.symbol)
                        .unwrap_or_else(|| panic!("Symbol '{}' not found in symbol table",
                            reloc.symbol));

                    // 심볼의 실제 메모리 주소 계산
                    let symbol_addr = match symbol.section {
                        section::SectionType::Text => text_addr + symbol.offset as u64,
                        section::SectionType::RoData => rodata_addr + symbol.offset as u64,
                        section::SectionType::Data => {
                            panic!("Data section not supported in executable yet")
                        }
                        section::SectionType::Bss => {
                            panic!("BSS section not supported in executable yet")
                        }
                    };

                    // PC-relative 계산: target_addr - (current_addr + 4)
                    let current_addr = text_addr + reloc.offset as u64 + 4;
                    let offset = (symbol_addr as i64 - current_addr as i64 + reloc.addend) as i32;

                    // 오프셋 패치
                    let offset_bytes = offset.to_le_bytes();
                    text_data[reloc.offset..reloc.offset + 4].copy_from_slice(&offset_bytes);
                }
            }
        }

        binary.extend_from_slice(&text_data);

        // 패딩으로 파일 오프셋 0x2000까지 채우기
        while binary.len() < rodata_file_offset {
            binary.push(0);
        }

        // .rodata 섹션 데이터
        binary.extend_from_slice(&self.rodata_section.data);

        binary
    }

    fn write_executable_elf_header(&self, buffer: &mut Vec<u8>, entry_point: u64) {
        let header = ELFHeader64::executable_x86_64(entry_point);
        buffer.extend_from_slice(&header.to_bytes());
    }

    fn write_program_header(
        &self,
        buffer: &mut Vec<u8>,
        p_type: u32,
        p_flags: u32,
        p_offset: usize,
        p_vaddr: u64,
        p_paddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_align: u64,
    ) {
        // p_type
        buffer.extend_from_slice(&p_type.to_le_bytes());

        // p_flags
        buffer.extend_from_slice(&p_flags.to_le_bytes());

        // p_offset
        buffer.extend_from_slice(&(p_offset as u64).to_le_bytes());

        // p_vaddr
        buffer.extend_from_slice(&p_vaddr.to_le_bytes());

        // p_paddr
        buffer.extend_from_slice(&p_paddr.to_le_bytes());

        // p_filesz
        buffer.extend_from_slice(&p_filesz.to_le_bytes());

        // p_memsz
        buffer.extend_from_slice(&p_memsz.to_le_bytes());

        // p_align
        buffer.extend_from_slice(&p_align.to_le_bytes());
    }

    fn write_elf_header(&self, buffer: &mut Vec<u8>) {
        let header = ELFHeader64::relocatable_x86_64();
        buffer.extend_from_slice(&header.to_bytes());
    }

    fn patch_elf_header(&self, buffer: &mut Vec<u8>, section_header_offset: u64) {
        // Section header offset은 ELF 헤더의 40번째 바이트
        let offset_pos = 40;
        let bytes = section_header_offset.to_le_bytes();
        buffer[offset_pos..offset_pos + 8].copy_from_slice(&bytes);
    }

    fn write_null_section_header(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&[0u8; 64]);
    }

    fn write_section_header(
        &self,
        buffer: &mut Vec<u8>,
        name_offset: u32,
        sh_type: u32,
        flags: u64,
        offset: usize,
        size: usize,
        align: u64,
        link: u32,
        info: u32,
        entsize: u64,
    ) {
        // sh_name
        buffer.extend_from_slice(&name_offset.to_le_bytes());

        // sh_type
        buffer.extend_from_slice(&sh_type.to_le_bytes());

        // sh_flags
        buffer.extend_from_slice(&flags.to_le_bytes());

        // sh_addr (0 for object files)
        buffer.extend_from_slice(&0u64.to_le_bytes());

        // sh_offset
        buffer.extend_from_slice(&(offset as u64).to_le_bytes());

        // sh_size
        buffer.extend_from_slice(&(size as u64).to_le_bytes());

        // sh_link
        buffer.extend_from_slice(&link.to_le_bytes());

        // sh_info
        buffer.extend_from_slice(&info.to_le_bytes());

        // sh_addralign
        buffer.extend_from_slice(&align.to_le_bytes());

        // sh_entsize
        buffer.extend_from_slice(&entsize.to_le_bytes());
    }

    fn build_section_string_table(&self) -> Vec<u8> {
        let mut strtab = Vec::new();

        // NULL string
        strtab.push(0); // offset 0

        // Section names
        strtab.extend_from_slice(b".text\0"); // offset 1
        strtab.extend_from_slice(b".rodata\0"); // offset 7
        strtab.extend_from_slice(b".data\0"); // offset 15
        strtab.extend_from_slice(b".bss\0"); // offset 21
        strtab.extend_from_slice(b".symtab\0"); // offset 26
        strtab.extend_from_slice(b".strtab\0"); // offset 34
        strtab.extend_from_slice(b".rela.text\0"); // offset 42
        strtab.extend_from_slice(b".shstrtab\0"); // offset 53

        strtab
    }

    fn build_string_table(&self) -> (Vec<u8>, std::collections::HashMap<String, u32>) {
        let mut strtab = Vec::new();
        let mut offsets = std::collections::HashMap::new();

        // NULL string
        strtab.push(0);

        // Add all symbol names
        for symbol in &self.symbol_table.symbols {
            if !offsets.contains_key(&symbol.name) {
                let offset = strtab.len() as u32;
                offsets.insert(symbol.name.clone(), offset);
                strtab.extend_from_slice(symbol.name.as_bytes());
                strtab.push(0);
            }
        }

        (strtab, offsets)
    }

    fn build_symbol_table(
        &self,
        string_offsets: &std::collections::HashMap<String, u32>,
    ) -> Vec<u8> {
        let mut symtab = Vec::new();

        // NULL symbol (first entry must be null)
        symtab.extend_from_slice(&[0u8; 24]);

        // Add symbols
        for symbol in &self.symbol_table.symbols {
            // st_name
            let name_offset = string_offsets.get(&symbol.name).unwrap_or(&0);
            symtab.extend_from_slice(&name_offset.to_le_bytes());

            // st_info (type and binding)
            let st_type = match symbol.symbol_type {
                symbol::SymbolType::Function => 2,
                symbol::SymbolType::Object => 1,
                symbol::SymbolType::Section => 3,
                symbol::SymbolType::File => 4,
            };
            let st_bind = match symbol.binding {
                symbol::SymbolBinding::Local => 0,
                symbol::SymbolBinding::Global => 1,
                symbol::SymbolBinding::Weak => 2,
            };
            let st_info = (st_bind << 4) | (st_type & 0xf);
            symtab.push(st_info);

            // st_other
            symtab.push(0);

            // st_shndx (section index)
            let section_index: u16 = match symbol.section {
                section::SectionType::Text => 1,
                section::SectionType::RoData => 2,
                section::SectionType::Data => 3,
                section::SectionType::Bss => 4,
            };
            symtab.extend_from_slice(&section_index.to_le_bytes());

            // st_value (offset in section)
            symtab.extend_from_slice(&(symbol.offset as u64).to_le_bytes());

            // st_size
            symtab.extend_from_slice(&(symbol.size as u64).to_le_bytes());
        }

        symtab
    }

    fn build_relocation_table(&self) -> Vec<u8> {
        let mut rela = Vec::new();

        for reloc in &self.relocations {
            // r_offset
            rela.extend_from_slice(&(reloc.offset as u64).to_le_bytes());

            // r_info (symbol index and type)
            // 심볼 인덱스 찾기 (1-based, 0은 NULL)
            let symbol_index = self
                .symbol_table
                .symbols
                .iter()
                .position(|s| s.name == reloc.symbol)
                .map(|idx| (idx + 1) as u32)
                .unwrap_or(0);

            // 재배치 타입
            let reloc_type = match reloc.reloc_type {
                relocation::RelocationType::Abs64 => 1,
                relocation::RelocationType::PcRel32 => 2,
                relocation::RelocationType::Abs32 => 10,
                relocation::RelocationType::PltPcRel32 => 4,
                relocation::RelocationType::GotPcRel => 9,
            };

            let r_info = ((symbol_index as u64) << 32) | (reloc_type as u64);
            rela.extend_from_slice(&r_info.to_le_bytes());

            // r_addend
            rela.extend_from_slice(&reloc.addend.to_le_bytes());
        }

        rela
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::platforms::amd64::instruction::Instruction;
    use crate::platforms::amd64::register::Register;
    use crate::platforms::amd64::rex::RexPrefix;
    use crate::platforms::linux::elf::relocation::{Relocation, RelocationType};
    use crate::platforms::linux::elf::{
        object::{ELFObject, ELFOutputType},
        section::SectionType,
        symbol::{Symbol, SymbolBinding, SymbolType},
    };

    #[test]
    fn test_generate_amd64_linux_elf() {
        let mut object = ELFObject::new();

        // Hello World 문자열을 .rodata 섹션에 추가
        let hello_str = b"Hello, World!\n";
        object.rodata_section.data.extend_from_slice(hello_str);

        // 심볼 테이블에 문자열 심볼 추가
        object.symbol_table.add_symbol(Symbol {
            name: "hello_msg".to_string(),
            section: SectionType::RoData,
            offset: 0,
            size: hello_str.len(),
            symbol_type: SymbolType::Object,
            binding: SymbolBinding::Local,
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
        use crate::platforms::amd64::register::modrm_digit_reg;

        let machine_code = vec![
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RAX), // ModR/M byte for mov rax, imm32
            crate::platforms::linux::syscall::amd64::SYS_WRITE,
            0x00,
            0x00,
            0x00, // mov rax, 1 (sys_write)
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RDI), // ModR/M byte for mov rdi, imm32
            crate::platforms::linux::fd::STDOUT,
            0x00,
            0x00,
            0x00, // mov rdi, 1 (stdout)
            RexPrefix::RexW as u8,
            Instruction::Lea as u8,
            crate::platforms::amd64::modrm::LEA_RSI_RIP_REL,
            0x00,
            0x00,
            0x00,
            0x00, // lea rsi, [rip+0] (재배치 필요)
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RDX), // ModR/M byte for mov rdx, imm32
            0x0e,                              // 14 (Hello World 문자열 길이)
            0x00,
            0x00,
            0x00, // mov rdx, 14
            Instruction::SYSCALL_BYTES[0],
            Instruction::SYSCALL_BYTES[1], // syscall
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RAX), // ModR/M byte for mov rax, imm32
            crate::platforms::linux::syscall::amd64::SYS_EXIT,
            0x00,
            0x00,
            0x00, // mov rax, 60 (sys_exit)
            RexPrefix::RexW as u8,
            Instruction::Xor as u8,
            crate::platforms::amd64::modrm::XOR_RDI_RDI, // xor rdi, rdi
            Instruction::SYSCALL_BYTES[0],
            Instruction::SYSCALL_BYTES[1], // syscall
        ];

        object.text_section.data = machine_code;

        // _start 함수 심볼 추가 (엔트리 포인트)
        object.symbol_table.add_symbol(Symbol {
            name: "_start".to_string(),
            section: SectionType::Text,
            offset: 0,
            size: object.text_section.data.len(),
            symbol_type: SymbolType::Function,
            binding: SymbolBinding::Global,
        });

        // .rodata 섹션의 hello_msg에 대한 재배치 정보 추가
        // lea rsi, [rip+offset] 명령어의 오프셋 부분(바이트 17-20)을 패치해야 함
        object.relocations.push(Relocation {
            section: SectionType::Text,
            offset: 17, // lea 명령어의 오프셋 필드 위치
            symbol: "hello_msg".to_string(),
            reloc_type: RelocationType::PcRel32,
            addend: 0, // PC-relative 계산 (RIP는 이미 offset 필드 끝을 가리킴)
        });

        let bytes = object.encode(ELFOutputType::Relocatable);

        fs::write("output.o", bytes).expect("Failed to write ELF object file");

        println!("ELF object file created: output.o");
        println!("To create executable, run: ld output.o -o hello");
        println!("To run: ./hello");
    }

    #[test]
    pub fn test_generate_amd64_linux_executable_elf() {
        let mut object = ELFObject::new();

        // Hello World 문자열을 .rodata 섹션에 추가
        let hello_str = b"Hello, World!\n";
        object.rodata_section.data.extend_from_slice(hello_str);

        // 심볼 테이블에 문자열 심볼 추가
        object.symbol_table.add_symbol(Symbol {
            name: "hello_msg".to_string(),
            section: SectionType::RoData,
            offset: 0,
            size: hello_str.len(),
            symbol_type: SymbolType::Object,
            binding: SymbolBinding::Local,
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
        use crate::platforms::amd64::register::modrm_digit_reg;

        let machine_code = vec![
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RAX), // ModR/M byte for mov rax, imm32
            crate::platforms::linux::syscall::amd64::SYS_WRITE,
            0x00,
            0x00,
            0x00, // mov rax, 1 (sys_write)
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RDI), // ModR/M byte for mov rdi, imm32
            crate::platforms::linux::fd::STDOUT,
            0x00,
            0x00,
            0x00, // mov rdi, 1 (stdout)
            RexPrefix::RexW as u8,
            Instruction::Lea as u8,
            crate::platforms::amd64::modrm::LEA_RSI_RIP_REL,
            0x00,
            0x00,
            0x00,
            0x00, // lea rsi, [rip+0] (재배치 필요)
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RDX), // ModR/M byte for mov rdx, imm32
            0x0e,                              // 14 (Hello World 문자열 길이)
            0x00,
            0x00,
            0x00, // mov rdx, 14
            Instruction::SYSCALL_BYTES[0],
            Instruction::SYSCALL_BYTES[1], // syscall
            RexPrefix::RexW as u8,
            Instruction::MovImm as u8,
            modrm_digit_reg(0, Register::RAX), // ModR/M byte for mov rax, imm32
            crate::platforms::linux::syscall::amd64::SYS_EXIT,
            0x00,
            0x00,
            0x00, // mov rax, 60 (sys_exit)
            RexPrefix::RexW as u8,
            Instruction::Xor as u8,
            crate::platforms::amd64::modrm::XOR_RDI_RDI, // xor rdi, rdi
            Instruction::SYSCALL_BYTES[0],
            Instruction::SYSCALL_BYTES[1], // syscall
        ];

        object.text_section.data = machine_code;

        // _start 함수 심볼 추가 (엔트리 포인트)
        object.symbol_table.add_symbol(Symbol {
            name: "_start".to_string(),
            section: SectionType::Text,
            offset: 0,
            size: object.text_section.data.len(),
            symbol_type: SymbolType::Function,
            binding: SymbolBinding::Global,
        });

        // .rodata 섹션의 hello_msg에 대한 재배치 정보 추가
        // lea rsi, [rip+offset] 명령어의 오프셋 부분(바이트 17-20)을 패치해야 함
        object.relocations.push(Relocation {
            section: SectionType::Text,
            offset: 17, // lea 명령어의 오프셋 필드 위치
            symbol: "hello_msg".to_string(),
            reloc_type: RelocationType::PcRel32,
            addend: 0, // PC-relative 계산 (RIP는 이미 offset 필드 끝을 가리킴)
        });

        let bytes = object.encode(ELFOutputType::Executable);

        fs::write("hello.exe", bytes).expect("Failed to write ELF executable file");

        // 실행 권한 부여
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata("hello.exe")
                .expect("Failed to get file metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions("hello.exe", perms).expect("Failed to set permissions");
        }

        println!("ELF executable file created: hello.exe");
        println!("To run: ./hello.exe");
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
