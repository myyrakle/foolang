#![cfg(test)]

use crate::{
    ast::{
        expression::{literal::LiteralExpression, Expression},
        statement::define_variable::VariableDefinitionStatement,
    },
    lexer::tokenizer::Tokenizer,
    parser::Parser,
};

#[test]
pub fn declare_let_variable() {
    let text = r#"let foo = 10"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    let mut parser = Parser::new();
    parser.set_tokens(tokens);

    let statements = parser.parse().unwrap();

    assert_eq!(
        statements,
        vec![VariableDefinitionStatement {
            name: "foo".to_owned(),
            value: Expression::Literal(LiteralExpression::Integer(10)).into(),
            mutable: false
        }
        .into()]
    );
}
