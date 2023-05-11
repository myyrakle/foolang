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

    _Self,     // self
    _SelfType, // Self
}
