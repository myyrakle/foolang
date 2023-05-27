use crate::{
    ast::expression::{variable::VariableExpression, Expression},
    error::all_error::{parser_error::ParserError, AllError},
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
            return Err(ParserError::new(400, "Unexpected end of tokens".to_string()).into());
        };

        let current_identifer = if let Token::Primary(PrimaryToken::Identifier(id)) = current_token
        {
            id
        } else {
            return Err(ParserError::new(
                401,
                format!("Expected identifier, found {:?}", current_token),
            )
            .into());
        };

        let variable_expression = VariableExpression {
            name: current_identifer,
        };

        if let Some(next_token) = self.get_next_token() {
            if next_token.is_binary_operator() {
                self.next();
                let binary_expression =
                    self.parse_binary_expression(variable_expression.into(), _context)?;

                Ok(binary_expression)
            } else {
                Err(ParserError::new(
                    402,
                    format!("Expected binary operator, found {:?}", next_token),
                )
                .into())
            }
        } else {
            self.next();
            Ok(variable_expression.into())
        }
    }
}
