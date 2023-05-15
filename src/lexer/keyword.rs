use super::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Let,
    Const,
    Mut,
    Static,

    Fn,
    Return,

    If,
    Else,
    Match,

    Break,
    Continue,

    As,
    In,

    For,
    While,
    Loop,

    Async,
    Await,

    Use,

    Struct,
    Class,
    Impl,

    True,
    False,

    Where,

    Type,

    Unsafe,

    Void,      // void
    _Self,     // self
    _SelfType, // Self
}

impl From<Keyword> for Token {
    fn from(token: Keyword) -> Self {
        Token::Keyword(token)
    }
}
