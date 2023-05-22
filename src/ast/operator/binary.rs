use crate::lexer::operator::OperatorToken;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,                // +
    Subtract,           // -
    Multiply,           // *
    Divide,             // /
    Modulo,             // %
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=
    And,                // &&
    Or,                 // ||
}

impl From<OperatorToken> for BinaryOperator {
    fn from(token: OperatorToken) -> Self {
        match token {
            OperatorToken::Plus => Self::Add,
            OperatorToken::Minus => Self::Subtract,
            OperatorToken::Star => Self::Multiply,
            OperatorToken::Slash => Self::Divide,
            OperatorToken::Modulo => Self::Modulo,
            OperatorToken::Equal => Self::Equal,
            OperatorToken::NotEqual => Self::NotEqual,
            OperatorToken::LessThan => Self::LessThan,
            OperatorToken::LessThanOrEqual => Self::LessThanOrEqual,
            OperatorToken::GreaterThan => Self::GreaterThan,
            OperatorToken::GreaterThanOrEqual => Self::GreaterThanOrEqual,
            OperatorToken::And => Self::And,
            OperatorToken::Or => Self::Or,
            _ => panic!("Cannot convert {:?} to BinaryOperator", token),
        }
    }
}
