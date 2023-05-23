use crate::{
    ast::expression::{parentheses::ParenthesesExpression, Expression},
    error::all_error::AllError,
    lexer::{general::GeneralToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_parentheses_expression(
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

        if let Token::GeneralToken(GeneralToken::LeftParentheses) = current_token {
        } else {
            return Err(AllError::ParserError(format!(
                "Expected '(', found {:?}",
                current_token
            )));
        }

        self.next();
        let expression = self.parse_expression(_context)?;

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        if let Token::GeneralToken(GeneralToken::RightParentheses) = current_token {
            self.next();

            let parentheses_expression = ParenthesesExpression {
                expression: Box::new(expression),
            };

            Ok(parentheses_expression.into())
        } else {
            Err(AllError::ParserError(format!(
                "Expected ')', found {:?}",
                current_token
            )))
        }
    }
}
