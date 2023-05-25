#![cfg(test)]

use crate::{
    ast::expression::{literal::LiteralExpression, variable::VariableExpression, Expression},
    lexer::tokenizer::Tokenizer,
    parser::Parser,
};

#[test]
pub fn integer() {
    let text = r#"123234"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Literal(LiteralExpression::Integer(123234)).into()]
    );
}

#[test]
pub fn float() {
    let text = r#"123.234"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Literal(LiteralExpression::Float(123.234)).into()]
    );
}

#[test]
pub fn string() {
    let text = r#""123.234""#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Literal(LiteralExpression::String("123.234".to_owned())).into()]
    );
}

#[test]
pub fn boolean_true() {
    let text = r#"true"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Literal(LiteralExpression::Boolean(true)).into()]
    );
}

#[test]
pub fn boolean_false() {
    let text = r#"false"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Literal(LiteralExpression::Boolean(false)).into()]
    );
}

#[test]
pub fn variable() {
    let text = r#"a"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();
    println!("test");

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Variable(VariableExpression { name: "a".into() }).into()]
    );
}
