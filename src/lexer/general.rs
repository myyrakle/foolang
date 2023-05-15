use super::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum GeneralToken {
    // general syntax
    Arrow,            // ->
    Comma,            // ,
    SemiColon,        // ;
    LeftParentheses,  // (
    RightParentheses, // )
}

impl From<GeneralToken> for Token {
    fn from(token: GeneralToken) -> Self {
        Token::GeneralToken(token)
    }
}
