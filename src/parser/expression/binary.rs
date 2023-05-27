use crate::{
    ast::{
        expression::{binary::BinaryExpression, Expression},
        operator::binary::BinaryOperator,
    },
    error::all_error::{parser_error::ParserError, AllError},
    lexer::token::Token,
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
            return Err(ParserError::new(9, "Unexpected end of tokens".to_string()).into());
        };

        if !current_token.is_binary_operator() {
            return Err(ParserError::new(
                7,
                format!("Expected binary operator, found {:?}", current_token),
            )
            .into());
        }

        let operator: BinaryOperator = if let Token::Operator(operator) = current_token {
            operator.into()
        } else {
            return Err(ParserError::new(
                8,
                format!("Expected binary operator, found {:?}", current_token),
            )
            .into());
        };

        // 현재 연산자의 우선순위
        let current_precedence = operator.get_precedence();

        // rhs에 괄호 연산자가 있는 경우
        let mut rhs_has_parentheses = false;

        // lhs에 괄호 연산자가 있는 경우
        let mut lhs_has_parentheses = false;

        self.next();
        let rhs = self.parse_expression(_context)?;

        // 소괄호가 있다면 벗기고 플래그값 설정
        let rhs = if let Expression::Parentheses(paren) = rhs {
            rhs_has_parentheses = true;
            *paren.expression
        } else {
            rhs
        };

        let lhs = if let Expression::Parentheses(paren) = lhs {
            lhs_has_parentheses = true;
            *paren.expression
        } else {
            lhs
        };

        // rhs에 또 binary operation이 중첩되는 경우 처리
        if let Expression::Binary(rhs_binary_expression) = rhs.clone() {
            if lhs.is_unary() {
                let lhs = Box::new(lhs);

                let new_lhs = Box::new(
                    BinaryExpression {
                        lhs,
                        rhs: rhs_binary_expression.lhs,
                        operator,
                    }
                    .into(),
                );
                Ok(BinaryExpression {
                    lhs: new_lhs,
                    rhs: rhs_binary_expression.rhs,
                    operator: rhs_binary_expression.operator,
                }
                .into())
            } else {
                if lhs_has_parentheses {
                    return Ok(BinaryExpression {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        operator,
                    }
                    .into());
                }

                let next_precedence = rhs_binary_expression.operator.get_precedence();

                let lhs = Box::new(lhs);
                let rhs = Box::new(rhs);

                // 오른쪽 연산자의 우선순위가 더 크거나, 소괄호가 있을 경우 오른쪽을 먼저 묶어서 바인딩
                if next_precedence > current_precedence || rhs_has_parentheses {
                    Ok(BinaryExpression { lhs, rhs, operator }.into())
                }
                // 아니라면 왼쪽으로 묶어서 바인딩
                else {
                    let new_lhs = BinaryExpression {
                        lhs,
                        rhs: rhs_binary_expression.lhs,
                        operator,
                    };
                    Ok(BinaryExpression {
                        lhs: Box::new(new_lhs.into()),
                        rhs: rhs_binary_expression.rhs,
                        operator: rhs_binary_expression.operator,
                    }
                    .into())
                }
            }
        } else {
            let lhs = Box::new(lhs);
            let rhs = Box::new(rhs);

            Ok(BinaryExpression { lhs, rhs, operator }.into())
        }
    }
}
