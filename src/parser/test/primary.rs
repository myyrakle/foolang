#![cfg(test)]

use crate::{
    ast::expression::{literal::LiteralExpression, Expression},
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
