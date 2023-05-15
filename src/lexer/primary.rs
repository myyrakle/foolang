use super::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum PrimaryToken {
    // primary expression
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Comment(String),
}

impl From<PrimaryToken> for Token {
    fn from(token: PrimaryToken) -> Self {
        Token::Primary(token)
    }
}
