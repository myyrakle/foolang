#![cfg(test)]

use crate::{
    ast::{
        expression::{binary::BinaryExpression, literal::LiteralExpression, Expression},
        operator::binary::BinaryOperator,
    },
    lexer::tokenizer::Tokenizer,
    parser::Parser,
};

#[test]
pub fn add() {
    let text = r#"10 + 20"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Binary(BinaryExpression {
            operator: BinaryOperator::Add,
            lhs: Box::new(Expression::Literal(LiteralExpression::Integer(10)).into()),
            rhs: Box::new(Expression::Literal(LiteralExpression::Integer(20)).into()),
        })
        .into()]
    );
}

#[test]
pub fn add_then_add() {
    let text = r#"10 + 20 + 30"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Binary(BinaryExpression {
            operator: BinaryOperator::Add,
            lhs: Expression::Binary(BinaryExpression {
                operator: BinaryOperator::Add,
                lhs: Box::new(Expression::Literal(LiteralExpression::Integer(10)).into()),
                rhs: Box::new(Expression::Literal(LiteralExpression::Integer(20)).into()),
            })
            .into(),
            rhs: Box::new(Expression::Literal(LiteralExpression::Integer(30)).into()),
        })
        .into()]
    );
}

#[test]
pub fn add_then_multiply() {
    let text = r#"10 + 20 * 30"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Binary(BinaryExpression {
            operator: BinaryOperator::Add,
            lhs: Box::new(Expression::Literal(LiteralExpression::Integer(10)).into()),
            rhs: Expression::Binary(BinaryExpression {
                operator: BinaryOperator::Multiply,
                lhs: Box::new(Expression::Literal(LiteralExpression::Integer(20)).into()),
                rhs: Box::new(Expression::Literal(LiteralExpression::Integer(30)).into()),
            })
            .into(),
        })
        .into()]
    );
}

#[test]
pub fn add_then_multiply_with_parenthese() {
    let text = r#"(10 + 20) * 30"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    println!("tokens: {:?}", tokens);

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Binary(BinaryExpression {
            operator: BinaryOperator::Multiply,
            lhs: Expression::Binary(BinaryExpression {
                operator: BinaryOperator::Add,
                lhs: Box::new(Expression::Literal(LiteralExpression::Integer(10)).into()),
                rhs: Box::new(Expression::Literal(LiteralExpression::Integer(20)).into()),
            })
            .into(),
            rhs: Box::new(Expression::Literal(LiteralExpression::Integer(30)).into()),
        })
        .into()]
    );
}
