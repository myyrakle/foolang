use super::{
    general::GeneralToken, keyword::Keyword, operator::OperatorToken, primary::PrimaryToken,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Operator(OperatorToken),
    Primary(PrimaryToken),
    GeneralToken(GeneralToken),

    // exception handling
    EOF,
}

impl Token {
    pub fn is_binary_operator(&self) -> bool {
        match self {
            Token::Operator(operator) => operator.is_binary_operator(),
            _ => false,
        }
    }
}
