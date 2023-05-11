use super::keyword::Keyword;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),

    Arrow, // ->

    // primary expression
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),

    // Operator(OperatorToken),

    // general syntax
    Comma,            // ,
    Period,           // .
    SemiColon,        // ;
    LeftParentheses,  // (
    RightParentheses, // )
    Backslash,        // \

    // exception handling
    EOF,
    Error(String),
    UnknownCharacter(char),
}
