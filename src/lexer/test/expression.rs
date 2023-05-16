#![cfg(test)]

use crate::lexer::{operator::OperatorToken, primary::PrimaryToken, tokenizer::Tokenizer};

#[test]
pub fn binary_expression() {
    let text = r#"1+20"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(
        tokens,
        vec![
            PrimaryToken::Integer(1).into(),
            OperatorToken::Plus.into(),
            PrimaryToken::Integer(20).into()
        ]
    );
}
