#[derive(Debug)]
pub enum LiteralValue {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float64(f64),
    Boolean(bool),
    String(String),
}
