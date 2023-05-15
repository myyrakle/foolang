use super::{keyword::Keyword, operator::OperatorToken};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Operator(OperatorToken),

    // primary expression
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Comment(String),

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
