use crate::lexer::operator::OperatorToken;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,      // -
    Not,         // !
    Plus,        // +
    Minus,       // -
    Dereference, // *
    Reference,   // &
    BitwiseNot,  // ~
}

impl From<OperatorToken> for UnaryOperator {
    fn from(operator: OperatorToken) -> Self {
        match operator {
            OperatorToken::Minus => UnaryOperator::Negate,
            OperatorToken::Not => UnaryOperator::Not,
            OperatorToken::Plus => UnaryOperator::Plus,
            OperatorToken::Minus => UnaryOperator::Minus,
            OperatorToken::Star => UnaryOperator::Dereference,
            OperatorToken::Ampersand => UnaryOperator::Reference,
            OperatorToken::BitwiseNot => UnaryOperator::BitwiseNot,
            _ => panic!("Invalid unary operator: {:?}", operator),
        }
    }
}
