#[derive(Debug)]
pub enum LiteralValue {
    Int64(i64),
    Float64(f64),
    Boolean(bool),
    String(String),
}
