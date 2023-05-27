#![cfg(test)]

use crate::{
    ast::expression::{call::CallExpression, Expression},
    lexer::tokenizer::Tokenizer,
    parser::Parser,
};

#[test]
pub fn function_call_no_arguments() {
    let text = r#"foo()"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();
    println!("{:?}", tokens);

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![Expression::Call(CallExpression {
            function_name: "foo".to_owned(),
            arguments: vec![],
        })
        .into()]
    );
}
