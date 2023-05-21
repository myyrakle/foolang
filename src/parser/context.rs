#[derive(Debug, Clone, PartialEq)]
pub struct ParserContext {}

impl ParserContext {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new()
    }
}
