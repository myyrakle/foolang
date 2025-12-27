use super::section::SectionType;

/// 심볼 테이블 엔트리
#[derive(Debug, Clone)]
pub struct Symbol {
    /// 심볼 이름 (예: "my_constant", "main")
    pub name: String,

    /// 심볼이 속한 섹션
    pub section: SectionType,

    /// 섹션 내 오프셋
    pub offset: usize,

    /// 심볼 크기
    pub size: usize,

    /// 심볼 타입
    pub symbol_type: SymbolType,

    /// 바인딩 (로컬/글로벌)
    pub binding: SymbolBinding,
}

/// 심볼 타입
#[derive(Debug, Clone)]
pub enum SymbolType {
    Function,
    Object, // 변수, 상수
    Section,
    File,
}

/// 심볼 바인딩
#[derive(Debug, Clone)]
pub enum SymbolBinding {
    Local,  // static
    Global, // extern
    Weak,
}

/// 심볼 테이블
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }
}
