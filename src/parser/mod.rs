pub mod context;
pub use context::ParserContext;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    context: ParserContext,
}
