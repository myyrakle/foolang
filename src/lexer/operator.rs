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
