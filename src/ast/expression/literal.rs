use crate::lexer::primary::PrimaryToken;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpression {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl From<PrimaryToken> for LiteralExpression {
    fn from(token: PrimaryToken) -> Self {
        match token {
            PrimaryToken::String(string) => Self::String(string),
            PrimaryToken::Integer(integer) => Self::Integer(integer),
            PrimaryToken::Float(float) => Self::Float(float),
            PrimaryToken::Boolean(boolean) => Self::Boolean(boolean),
            _ => panic!("Cannot convert {:?} to LiteralExpression", token),
        }
    }
}
