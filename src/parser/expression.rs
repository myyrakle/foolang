pub(crate) mod binary;
pub(crate) mod function_call;
pub(crate) mod parentheses;
pub(crate) mod unary;
pub(crate) mod variable;

use crate::{
    ast::expression::Expression,
    error::all_error::AllError,
    lexer::{general::GeneralToken, primary::PrimaryToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_expression(
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

        match current_token {
            Token::Primary(PrimaryToken::Comment(comment)) => {
                self.next();
                Ok(Expression::Comment(comment))
            }
            Token::Primary(PrimaryToken::Identifier(_)) => {
                if let Some(Token::GeneralToken(GeneralToken::LeftParentheses)) =
                    self.get_next_token()
                {
                    let function_call_expression = self.parse_function_call_expression(context)?;

                    Ok(function_call_expression)
                } else {
                    let variable_expression = self.parse_variable_expression(context)?;

                    Ok(variable_expression)
                }
            }
            Token::Primary(primary) => {
                if let Some(next_token) = self.get_next_token() {
                    if next_token.is_binary_operator() {
                        self.next();

                        let binary_expression =
                            self.parse_binary_expression(Expression::from(primary), context)?;

                        Ok(binary_expression)
                    } else {
                        self.next();
                        Ok(primary.into())
                    }
                } else {
                    self.next();
                    Ok(primary.into())
                }
            }
            Token::Operator(operator) => {
                if operator.is_unary_operator() {
                    let unary_expression = self.parse_unary_expression(context)?;

                    Ok(unary_expression)
                } else {
                    Err(AllError::ParserError(format!(
                        "Expected unary operator, found {:?}",
                        operator
                    )))
                }
            }
            Token::GeneralToken(GeneralToken::LeftParentheses) => {
                let parentheses_expression = self.parse_parentheses_expression(context.clone())?;

                if let Some(current_token) = self.get_current_token() {
                    if current_token.is_binary_operator() {
                        let binary_expression =
                            self.parse_binary_expression(parentheses_expression, context)?;

                        Ok(binary_expression)
                    } else {
                        self.next();
                        Ok(parentheses_expression)
                    }
                } else {
                    self.next();
                    Ok(parentheses_expression)
                }
            }
            _ => todo!(),
        }
    }
}
