use crate::{
    ast::{
        expression::{binary::BinaryExpression, Expression},
        operator::binary::BinaryOperator,
    },
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

        // 현재 연산자의 우선순위
        let current_precedence = operator.get_precedence();

        let rhs = self.parse_expression(_context)?;

        // rhs에 괄호 연산자가 있는 경우
        let mut rhs_has_parentheses = false;

        // rhs에 또 binary operation이 중첩되는 경우 처리
        if let Expression::Binary(rhs_binary_expression) = rhs.clone() {
            let next_precedence = rhs_binary_expression.operator.get_precedence();

            if lhs.is_unary() {
                let lhs = Box::new(lhs);

                let new_lhs = BinaryExpression {
                    lhs,
                    rhs: rhs_binary_expression.lhs,
                    operator,
                }
                .into();
                Ok(BinaryExpression {
                    lhs: new_lhs,
                    rhs: rhs_binary_expression.rhs,
                    operator: rhs_binary_expression.operator,
                }
                .into())
            } else {
                todo!();
            }
        } else {
            let lhs = Box::new(lhs);
            let rhs = Box::new(rhs);

            Ok(BinaryExpression { lhs, rhs, operator }.into())
        }
    }
}
