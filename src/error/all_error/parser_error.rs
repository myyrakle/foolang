use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]

pub struct ParserError {
    pub message: String,
    pub uid: i32,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parser error: {} ({})", self.message, self.uid)
    }
}

impl ParserError {
    pub fn new(uid: i32, message: String) -> Self {
        Self { message, uid }
    }
}
