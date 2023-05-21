use super::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum OperatorToken {
    // comparison operators
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=

    // logical operators
    And, // &&
    Or,  // ||
    Not, // !

    // arithmetic operators
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Modulo, // %

    // bitwise operators
    Ampersand,  // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseNot, // ~
    LeftShift,  // <<
    RightShift, // >>

    // assignment operators
    Assign,           // =
    PlusAssign,       // +=
    MinusAssign,      // -=
    StarAssign,       // *=
    SlashAssign,      // /=
    ModuloAssign,     // %=
    AndAssign,        // &=
    OrAssign,         // |=
    XorAssign,        // ^=
    NotAssign,        // ~=
    LeftShiftAssign,  // <<=
    RightShiftAssign, // >>=

    // Others
    Dot,      // .
    Range,    // ..
    Question, // ?
}

impl From<OperatorToken> for Token {
    fn from(token: OperatorToken) -> Self {
        Token::Operator(token)
    }
}

impl OperatorToken {
    pub fn is_binary_operator(&self) -> bool {
        match self {
            OperatorToken::Equal
            | OperatorToken::NotEqual
            | OperatorToken::LessThan
            | OperatorToken::LessThanOrEqual
            | OperatorToken::GreaterThan
            | OperatorToken::GreaterThanOrEqual
            | OperatorToken::And
            | OperatorToken::Or
            | OperatorToken::Plus
            | OperatorToken::Minus
            | OperatorToken::Star
            | OperatorToken::Slash
            | OperatorToken::Modulo
            | OperatorToken::Ampersand
            | OperatorToken::BitwiseOr
            | OperatorToken::BitwiseXor
            | OperatorToken::LeftShift
            | OperatorToken::RightShift
            | OperatorToken::Assign
            | OperatorToken::PlusAssign
            | OperatorToken::MinusAssign
            | OperatorToken::StarAssign
            | OperatorToken::SlashAssign
            | OperatorToken::ModuloAssign
            | OperatorToken::AndAssign
            | OperatorToken::OrAssign
            | OperatorToken::XorAssign
            | OperatorToken::NotAssign
            | OperatorToken::LeftShiftAssign
            | OperatorToken::RightShiftAssign => true,
            _ => false,
        }
    }

    pub fn is_unary_operator(&self) -> bool {
        match self {
            OperatorToken::Not
            | OperatorToken::BitwiseNot
            | OperatorToken::Minus
            | OperatorToken::Plus => true,
            _ => false,
        }
    }
}
