#[derive(Clone, Debug, PartialEq)]
pub enum PrimaryToken {
    // primary expression
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Comment(String),
}
