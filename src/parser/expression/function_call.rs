use crate::{
    ast::{
        expression::{unary::UnaryExpression, Expression},
        operator::unary::UnaryOperator,
    },
    error::all_error::AllError,
    lexer::token::Token,
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_function_call_expression(
        &mut self,
        _context: ParserContext,
    ) -> Result<Expression, AllError> {
        todo!()
    }
}
