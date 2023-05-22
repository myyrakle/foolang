use crate::{
    ast::expression::Expression,
    error::all_error::AllError,
    lexer::{primary::PrimaryToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_binary_expression(
        &mut self,
        lhs: Expression,
        _context: ParserContext,
    ) -> Result<Expression, AllError> {
        todo!("binary expression")
    }
}
