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
    Error(String),
    UnknownCharacter(char),
}
