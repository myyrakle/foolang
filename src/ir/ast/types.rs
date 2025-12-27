/// IR 타입 시스템
/// AMD64 아키텍처에서 사용 가능한 기본 타입들을 정의합니다.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
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

impl IRType {
    /// 타입의 바이트 크기를 반환합니다.
    pub fn size_in_bytes(&self) -> usize {
        match self {
            IRType::Int8 | IRType::UInt8 | IRType::Bool => 1,
            IRType::Int16 | IRType::UInt16 => 2,
            IRType::Int32 | IRType::UInt32 | IRType::Float32 => 4,
            IRType::Int64 | IRType::UInt64 | IRType::Float64 => 8,
            IRType::Pointer(_) => 8, // 64-bit 포인터
            IRType::Void => 0,
        }
    }

    /// 타입이 정수 타입인지 확인합니다.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            IRType::Int8
                | IRType::Int16
                | IRType::Int32
                | IRType::Int64
                | IRType::UInt8
                | IRType::UInt16
                | IRType::UInt32
                | IRType::UInt64
        )
    }

    /// 타입이 부호 있는 정수인지 확인합니다.
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            IRType::Int8 | IRType::Int16 | IRType::Int32 | IRType::Int64
        )
    }

    /// 타입이 부동소수점 타입인지 확인합니다.
    pub fn is_float(&self) -> bool {
        matches!(self, IRType::Float32 | IRType::Float64)
    }

    /// 타입이 포인터 타입인지 확인합니다.
    pub fn is_pointer(&self) -> bool {
        matches!(self, IRType::Pointer(_))
    }

    /// 타입의 문자열 표현을 반환합니다.
    pub fn to_string(&self) -> String {
        match self {
            IRType::Int8 => "i8".to_string(),
            IRType::Int16 => "i16".to_string(),
            IRType::Int32 => "i32".to_string(),
            IRType::Int64 => "i64".to_string(),
            IRType::UInt8 => "u8".to_string(),
            IRType::UInt16 => "u16".to_string(),
            IRType::UInt32 => "u32".to_string(),
            IRType::UInt64 => "u64".to_string(),
            IRType::Float32 => "f32".to_string(),
            IRType::Float64 => "f64".to_string(),
            IRType::Bool => "bool".to_string(),
            IRType::Pointer(inner) => format!("*{}", inner.to_string()),
            IRType::Void => "void".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_sizes() {
        assert_eq!(IRType::Int8.size_in_bytes(), 1);
        assert_eq!(IRType::Int16.size_in_bytes(), 2);
        assert_eq!(IRType::Int32.size_in_bytes(), 4);
        assert_eq!(IRType::Int64.size_in_bytes(), 8);
        assert_eq!(IRType::UInt64.size_in_bytes(), 8);
        assert_eq!(IRType::Float32.size_in_bytes(), 4);
        assert_eq!(IRType::Float64.size_in_bytes(), 8);
        assert_eq!(IRType::Bool.size_in_bytes(), 1);
        assert_eq!(IRType::Pointer(Box::new(IRType::Int32)).size_in_bytes(), 8);
        assert_eq!(IRType::Void.size_in_bytes(), 0);
    }

    #[test]
    fn test_type_checks() {
        assert!(IRType::Int32.is_integer());
        assert!(IRType::Int32.is_signed());
        assert!(!IRType::UInt32.is_signed());
        assert!(IRType::Float64.is_float());
        assert!(IRType::Pointer(Box::new(IRType::Int32)).is_pointer());
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(IRType::Int64.to_string(), "i64");
        assert_eq!(IRType::UInt32.to_string(), "u32");
        assert_eq!(IRType::Float64.to_string(), "f64");
        assert_eq!(
            IRType::Pointer(Box::new(IRType::Int32)).to_string(),
            "*i32"
        );
    }
}
