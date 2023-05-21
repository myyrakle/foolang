use crate::{ast::statement::Statement, error::all_error::AllError};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_expression(
        &mut self,
        context: ParserContext,
    ) -> Result<Statement, AllError> {
        todo!()
    }
}
