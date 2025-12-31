/// IR 타입 시스템
/// IR 수준에서 사용 가능한 기본 타입들을 정의합니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
    Primitive(IRPrimitiveType),
    Custom(IRCustomType),
}

// 사용자 정의 타입 (예: 구조체, 클래스 등)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IRCustomType {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRPrimitiveType {
    // 정수 타입
    Int8,
    Int16,
    Int32,
    Int64,

    // 부호 없는 정수 타입
    UInt8,
    UInt16,
    UInt32,
    UInt64,

    // 부동소수점 타입
    Float32,
    Float64,

    // 불리언 타입 (1바이트)
    Bool,

    // 포인터 타입
    Pointer(Box<IRType>),

    // Void 타입 (반환값 없음)
    Void,
}

impl IRPrimitiveType {
    /// 타입의 바이트 크기를 반환합니다.
    pub fn size_in_bytes(&self) -> usize {
        match self {
            IRPrimitiveType::Int8 | IRPrimitiveType::UInt8 | IRPrimitiveType::Bool => 1,
            IRPrimitiveType::Int16 | IRPrimitiveType::UInt16 => 2,
            IRPrimitiveType::Int32 | IRPrimitiveType::UInt32 | IRPrimitiveType::Float32 => 4,
            IRPrimitiveType::Int64 | IRPrimitiveType::UInt64 | IRPrimitiveType::Float64 => 8,
            IRPrimitiveType::Pointer(_) => 8, // 64-bit 포인터
            IRPrimitiveType::Void => 0,
        }
    }

    /// 타입이 정수 타입인지 확인합니다.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            IRPrimitiveType::Int8
                | IRPrimitiveType::Int16
                | IRPrimitiveType::Int32
                | IRPrimitiveType::Int64
                | IRPrimitiveType::UInt8
                | IRPrimitiveType::UInt16
                | IRPrimitiveType::UInt32
                | IRPrimitiveType::UInt64
        )
    }

    /// 타입이 부호 있는 정수인지 확인합니다.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            IRPrimitiveType::Int8
                | IRPrimitiveType::Int16
                | IRPrimitiveType::Int32
                | IRPrimitiveType::Int64
        )
    }

    /// 타입이 부동소수점 타입인지 확인합니다.
    pub fn is_float(&self) -> bool {
        matches!(self, IRPrimitiveType::Float32 | IRPrimitiveType::Float64)
    }

    /// 타입이 포인터 타입인지 확인합니다.
    pub fn is_pointer(&self) -> bool {
        matches!(self, IRPrimitiveType::Pointer(_))
    }

    /// 타입의 문자열 표현을 반환합니다.
    pub fn type_to_string(&self) -> String {
        match self {
            IRPrimitiveType::Int8 => "i8".to_string(),
            IRPrimitiveType::Int16 => "i16".to_string(),
            IRPrimitiveType::Int32 => "i32".to_string(),
            IRPrimitiveType::Int64 => "i64".to_string(),
            IRPrimitiveType::UInt8 => "u8".to_string(),
            IRPrimitiveType::UInt16 => "u16".to_string(),
            IRPrimitiveType::UInt32 => "u32".to_string(),
            IRPrimitiveType::UInt64 => "u64".to_string(),
            IRPrimitiveType::Float32 => "f32".to_string(),
            IRPrimitiveType::Float64 => "f64".to_string(),
            IRPrimitiveType::Bool => "bool".to_string(),
            IRPrimitiveType::Pointer(inner) => format!("*{}", inner.type_to_string()),
            IRPrimitiveType::Void => "void".to_string(),
        }
    }
}

impl IRType {
    /// 타입의 문자열 표현을 반환합니다.
    pub fn type_to_string(&self) -> String {
        match self {
            IRType::Primitive(prim) => prim.type_to_string(),
            IRType::Custom(custom) => custom.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_sizes() {
        assert_eq!(IRPrimitiveType::Int8.size_in_bytes(), 1);
        assert_eq!(IRPrimitiveType::Int16.size_in_bytes(), 2);
        assert_eq!(IRPrimitiveType::Int32.size_in_bytes(), 4);
        assert_eq!(IRPrimitiveType::Int64.size_in_bytes(), 8);
        assert_eq!(IRPrimitiveType::UInt64.size_in_bytes(), 8);
        assert_eq!(IRPrimitiveType::Float32.size_in_bytes(), 4);
        assert_eq!(IRPrimitiveType::Float64.size_in_bytes(), 8);
        assert_eq!(IRPrimitiveType::Bool.size_in_bytes(), 1);
        assert_eq!(
            IRPrimitiveType::Pointer(Box::new(IRType::Primitive(IRPrimitiveType::Int32)))
                .size_in_bytes(),
            8
        );
        assert_eq!(IRPrimitiveType::Void.size_in_bytes(), 0);
    }

    #[test]
    fn test_type_checks() {
        assert!(IRPrimitiveType::Int32.is_integer());
        assert!(IRPrimitiveType::Int32.is_signed());
        assert!(!IRPrimitiveType::UInt32.is_signed());
        assert!(IRPrimitiveType::Float64.is_float());
        assert!(
            IRPrimitiveType::Pointer(Box::new(IRType::Primitive(IRPrimitiveType::Int32)))
                .is_pointer()
        );
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(IRPrimitiveType::Int64.type_to_string(), "i64");
        assert_eq!(IRPrimitiveType::UInt32.type_to_string(), "u32");
        assert_eq!(IRPrimitiveType::Float64.type_to_string(), "f64");
        assert_eq!(
            IRPrimitiveType::Pointer(Box::new(IRType::Primitive(IRPrimitiveType::Int32)))
                .type_to_string(),
            "*i32"
        );
    }
}
