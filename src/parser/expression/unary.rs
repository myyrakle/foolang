use crate::{
    ast::{
        expression::{unary::UnaryExpression, Expression},
        operator::unary::UnaryOperator,
    },
    error::all_error::{parser_error::ParserError, AllError},
    lexer::token::Token,
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_unary_expression(
        &mut self,
        _context: ParserContext,
    ) -> Result<Expression, AllError> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(300, "Unexpected end of tokens".to_string()).into());
        };

        if !current_token.is_unary_operator() {
            return Err(ParserError::new(
                301,
                format!("Expected unary operator, found {:?}", current_token),
            )
            .into());
        }

        let operator: UnaryOperator = if let Token::Operator(operator) = current_token {
            operator.into()
        } else {
            return Err(ParserError::new(
                302,
                format!("Expected unary operator, found {:?}", current_token),
            )
            .into());
        };

        // rhs에 괄호 연산자가 있는 경우
        let operand = self.parse_expression(_context)?;
        let operand = Box::new(operand);

        Ok(UnaryExpression { operator, operand }.into())
    }
}
