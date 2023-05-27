use crate::{
    ast::expression::{call::CallExpression, Expression},
    error::all_error::{parser_error::ParserError, AllError},
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
            return Err(ParserError::new(100, "Unexpected end of tokens".to_string()).into());
        };

        let function_name = if let Token::Primary(PrimaryToken::Identifier(id)) = current_token {
            id
        } else {
            return Err(ParserError::new(
                101,
                format!("Expected identifier, found {:?}", current_token),
            )
            .into());
        };

        self.next();
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(102, "Unexpected end of tokens".to_string()).into());
        };

        if let Token::GeneralToken(GeneralToken::LeftParentheses) = current_token {
        } else {
            return Err(
                ParserError::new(103, format!("Expected '(', found {:?}", current_token)).into(),
            );
        }

        let mut arguments = vec![];

        loop {
            let next_token = self.get_next_token();

            if let Some(next_token) = next_token {
                // ) 만나면 종료
                if let Token::GeneralToken(GeneralToken::RightParentheses) = next_token {
                    self.next();
                    self.next();
                    break;
                }
            } else {
                return Err(ParserError::new(104, "Unexpected end of tokens".to_string()).into());
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
                return Err(ParserError::new(105, "Unexpected end of tokens".to_string()).into());
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
                match next_token {
                    Token::GeneralToken(GeneralToken::SemiColon) => {
                        self.next();
                        Ok(function_call_expression.into())
                    }
                    _ => Err(ParserError::new(
                        106,
                        format!("Expected binary operator, found {:?}", next_token),
                    )
                    .into()),
                }
            }
        } else {
            Ok(function_call_expression.into())
        }
    }
}
