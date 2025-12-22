use crate::{
    ast::expression::{parentheses::ParenthesesExpression, Expression},
    error::{parser_error::ParserError, Errors},
    lexer::{general::GeneralToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_parentheses_expression(
        &mut self,
        _context: ParserContext,
    ) -> Result<Expression, Errors> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(200, "Unexpected end of tokens".to_string()).into());
        };

        if let Token::GeneralToken(GeneralToken::LeftParentheses) = current_token {
        } else {
            return Err(
                ParserError::new(201, format!("Expected '(', found {:?}", current_token)).into(),
            );
        }

        self.next();
        let expression = self.parse_expression(_context)?;

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(202, "Unexpected end of tokens".to_string()).into());
        };

        if let Token::GeneralToken(GeneralToken::RightParentheses) = current_token {
            self.next();

            let parentheses_expression = ParenthesesExpression {
                expression: Box::new(expression),
            };

            Ok(parentheses_expression.into())
        } else {
            Err(ParserError::new(203, format!("Expected ')', found {:?}", current_token)).into())
        }
    }
}
