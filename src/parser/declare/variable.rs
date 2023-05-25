use crate::{
    ast::{
        expression::{variable::VariableExpression, Expression},
        statement::Statement,
    },
    error::all_error::AllError,
    lexer::{primary::PrimaryToken, token::Token},
    parser::{Parser, ParserContext},
};

impl Parser {
    pub(crate) fn parse_declare_variable(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, AllError> {
        todo!()
    }
}
