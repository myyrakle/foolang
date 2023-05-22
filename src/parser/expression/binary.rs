use crate::{
    ast::{expression::Expression, operator::binary::BinaryOperator},
    error::all_error::AllError,
    lexer::{operator::OperatorToken, primary::PrimaryToken, token::Token},
};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_binary_expression(
        &mut self,
        lhs: Expression,
        _context: ParserContext,
    ) -> Result<Expression, AllError> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        if !current_token.is_binary_operator() {
            return Err(AllError::ParserError(format!(
                "Expected binary operator, found {:?}",
                current_token
            )));
        }

        let operator: BinaryOperator = if let Token::Operator(operator) = current_token {
            operator.into()
        } else {
            return Err(AllError::ParserError(format!(
                "Expected binary operator, found {:?}",
                current_token
            )));
        };

        match current_token {
            Token::Operator(operator) => {
                self.next();
                let rhs = self.parse_expression(_context)?;
                let binary_expression = Expression::Binary(BinaryExpression {
                    lhs: Box::new(lhs),
                    operator,
                    rhs: Box::new(rhs),
                });

                Ok(binary_expression)
            }
            _ => todo!(),
        }
    }
}
