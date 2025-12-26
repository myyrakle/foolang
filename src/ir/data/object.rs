use crate::ir::data::{
    relocation::{self, Relocation},
    section::{self, LinkedSection, Section},
    symbol::{self, SymbolTable},
};

/// 컴파일된 오브젝트 파일
/// Unix/Linux: ELF
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
    pub fn to_elf_binary(&self) -> Vec<u8> {
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
        self.write_section_header(&mut binary, 1, 1, 6, text_offset, text_size, 16, 0, 0, 0);

        // 2: .rodata (SHT_PROGBITS, flags=A)
        self.write_section_header(&mut binary, 7, 1, 2, rodata_offset, rodata_size, 1, 0, 0, 0);

        // 3: .data (SHT_PROGBITS, flags=WA)
        self.write_section_header(&mut binary, 15, 1, 3, data_offset, data_size, 8, 0, 0, 0);

        // 4: .bss (SHT_NOBITS, flags=WA)
        self.write_section_header(&mut binary, 21, 8, 3, 0, bss_size, 8, 0, 0, 0);

        // 5: .symtab (SHT_SYMTAB, link=strtab section, info=first non-local symbol index)
        // link: 6 (.strtab), info: 2 (첫 번째 글로벌 심볼의 인덱스)
        self.write_section_header(&mut binary, 26, 2, 0, symtab_offset, symtab_size, 8, 6, 2, 24);

        // 6: .strtab (SHT_STRTAB)
        self.write_section_header(&mut binary, 34, 3, 0, strtab_offset, strtab_size, 1, 0, 0, 0);

        // 7: .rela.text (SHT_RELA, link=symtab, info=.text section index)
        // link: 5 (.symtab), info: 1 (.text), entsize: 24 (sizeof(Elf64_Rela))
        self.write_section_header(&mut binary, 42, 4, 0x40, rela_text_offset, rela_text_size, 8, 5, 1, 24);

        // 8: .shstrtab (SHT_STRTAB)
        self.write_section_header(&mut binary, 52, 3, 0, shstrtab_offset, shstrtab_size, 1, 0, 0, 0);

        // ELF Header의 섹션 헤더 오프셋 업데이트
        let section_headers_offset = section_headers_start as u64;
        self.patch_elf_header(&mut binary, section_headers_offset);

        binary
    }

    fn write_elf_header(&self, buffer: &mut Vec<u8>) {
        // ELF Magic Number
        buffer.extend_from_slice(&[0x7f, b'E', b'L', b'F']);

        // Class (64-bit)
        buffer.push(2);

        // Data (Little Endian)
        buffer.push(1);

        // Version
        buffer.push(1);

        // OS/ABI (UNIX System V)
        buffer.push(0);

        // ABI Version
        buffer.push(0);

        // Padding
        buffer.extend_from_slice(&[0; 7]);

        // Type (Relocatable)
        buffer.extend_from_slice(&1u16.to_le_bytes()); // ET_REL

        // Machine (x86-64)
        buffer.extend_from_slice(&0x3eu16.to_le_bytes());

        // Version
        buffer.extend_from_slice(&1u32.to_le_bytes());

        // Entry point (0 for object files)
        buffer.extend_from_slice(&0u64.to_le_bytes());

        // Program header offset (0 for object files)
        buffer.extend_from_slice(&0u64.to_le_bytes());

        // Section header offset (placeholder, will be patched)
        buffer.extend_from_slice(&0u64.to_le_bytes());

        // Flags
        buffer.extend_from_slice(&0u32.to_le_bytes());

        // ELF header size
        buffer.extend_from_slice(&64u16.to_le_bytes());

        // Program header entry size
        buffer.extend_from_slice(&0u16.to_le_bytes());

        // Program header count
        buffer.extend_from_slice(&0u16.to_le_bytes());

        // Section header entry size
        buffer.extend_from_slice(&64u16.to_le_bytes());

        // Section header count (9 sections: null, text, rodata, data, bss, symtab, strtab, rela.text, shstrtab)
        buffer.extend_from_slice(&9u16.to_le_bytes());

        // Section header string table index (8 = .shstrtab)
        buffer.extend_from_slice(&8u16.to_le_bytes());
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
        strtab.push(0);

        // Section names
        strtab.extend_from_slice(b".text\0");      // offset 1
        strtab.extend_from_slice(b".rodata\0");    // offset 9
        strtab.extend_from_slice(b".data\0");      // offset 17
        strtab.extend_from_slice(b".bss\0");       // offset 23
        strtab.extend_from_slice(b".symtab\0");    // offset 28
        strtab.extend_from_slice(b".strtab\0");    // offset 36
        strtab.extend_from_slice(b".rela.text\0"); // offset 44
        strtab.extend_from_slice(b".shstrtab\0");  // offset 54

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

    fn build_symbol_table(&self, string_offsets: &std::collections::HashMap<String, u32>) -> Vec<u8> {
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
            let symbol_index = self.symbol_table.symbols
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
