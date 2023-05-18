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
