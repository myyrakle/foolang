use crate::{
    ast::expression::{call::CallExpression, Expression},
    error::all_error::AllError,
    lexer::{general::GeneralToken, primary::PrimaryToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_function_call_expression(
        &mut self,
        context: ParserContext,
    ) -> Result<Expression, AllError> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        let function_name = if let Token::Primary(PrimaryToken::Identifier(id)) = current_token {
            id
        } else {
            return Err(AllError::ParserError(format!(
                "Expected identifier, found {:?}",
                current_token
            )));
        };

        self.next();
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

        let mut arguments = vec![];

        loop {
            let next_token = self.get_next_token();

            if let Some(next_token) = next_token {
                // ) 만나면 종료
                if let Token::GeneralToken(GeneralToken::RightParentheses) = next_token {
                    self.next();
                    break;
                }
            } else {
                return Err(AllError::ParserError(
                    "Unexpected end of tokens".to_string(),
                ));
            }

            self.next();

            // 각 argument를 파싱
            let expression = self.parse_expression(context.clone())?;
            arguments.push(expression);

            let next_token = self.get_next_token();

            if let Some(next_token) = next_token {
                if let Token::GeneralToken(GeneralToken::RightParentheses) = next_token {
                    self.next();
                    break;
                } else if let Token::GeneralToken(GeneralToken::Comma) = next_token {
                    self.next();
                }
            } else {
                return Err(AllError::ParserError(
                    "Unexpected end of tokens".to_string(),
                ));
            }
        }

        let function_call_expression = CallExpression {
            function_name,
            arguments,
        };

        if let Some(next_token) = self.get_next_token() {
            if next_token.is_binary_operator() {
                self.next();
                let binary_expression =
                    self.parse_binary_expression(function_call_expression.into(), context)?;

                Ok(binary_expression)
            } else {
                Err(AllError::ParserError(format!(
                    "Expected binary operator, found {:?}",
                    next_token
                )))
            }
        } else {
            self.next();

            Ok(function_call_expression.into())
        }
    }
}
