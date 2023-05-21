pub mod context;
pub use context::ParserContext;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    context: ParserContext,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            context: ParserContext::new(),
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
