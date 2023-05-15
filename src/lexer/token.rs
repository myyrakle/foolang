use super::{keyword::Keyword, operator::OperatorToken, primary::PrimaryToken};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Operator(OperatorToken),
    Primary(PrimaryToken),

    // general syntax
    Arrow,            // ->
    Comma,            // ,
    SemiColon,        // ;
    LeftParentheses,  // (
    RightParentheses, // )

    // exception handling
    EOF,
    Error(String),
    UnknownCharacter(char),
}
