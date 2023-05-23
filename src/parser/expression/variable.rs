use crate::{
    ast::expression::{variable::VariableExpression, Expression},
    error::all_error::AllError,
    lexer::{primary::PrimaryToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_variable_expression(
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

        let current_identifer = if let Token::Primary(PrimaryToken::Identifier(id)) = current_token
        {
            id
        } else {
            return Err(AllError::ParserError(format!(
                "Expected identifier, found {:?}",
                current_token
            )));
        };

        let variable_expression = VariableExpression {
            name: current_identifer.into(),
        };

        if let Some(next_token) = self.get_next_token() {
            if next_token.is_binary_operator() {
                self.next();
                let binary_expression =
                    self.parse_binary_expression(variable_expression.into(), _context)?;

                Ok(binary_expression)
            } else {
                Err(AllError::ParserError(format!(
                    "Expected binary operator, found {:?}",
                    next_token
                )))
            }
        } else {
            Ok(variable_expression.into())
        }
    }
}
