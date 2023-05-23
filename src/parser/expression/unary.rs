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
    pub(super) fn parse_unary_expression(
        &mut self,
        _context: ParserContext,
    ) -> Result<Expression, AllError> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        if !current_token.is_unary_operator() {
            return Err(AllError::ParserError(format!(
                "Expected unary operator, found {:?}",
                current_token
            )));
        }

        let operator: UnaryOperator = if let Token::Operator(operator) = current_token {
            operator.into()
        } else {
            return Err(AllError::ParserError(format!(
                "Expected binary operator, found {:?}",
                current_token
            )));
        };

        // rhs에 괄호 연산자가 있는 경우
        let operand = self.parse_expression(_context)?;
        let operand = Box::new(operand);

        Ok(UnaryExpression { operator, operand }.into())
    }
}
