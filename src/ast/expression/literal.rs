#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpression {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}
