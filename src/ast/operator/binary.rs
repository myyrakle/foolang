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

impl BinaryOperator {
    // 연산자 우선순위
    pub fn get_precedence(&self) -> u8 {
        match self {
            Self::Add => 1,
            Self::Subtract => 1,
            Self::Multiply => 2,
            Self::Divide => 2,
            Self::Modulo => 2,
            Self::Equal => 3,
            Self::NotEqual => 3,
            Self::LessThan => 3,
            Self::LessThanOrEqual => 3,
            Self::GreaterThan => 3,
            Self::GreaterThanOrEqual => 3,
            Self::And => 4,
            Self::Or => 4,
        }
    }
}
