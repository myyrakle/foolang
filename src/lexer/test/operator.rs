#![cfg(test)]

use crate::lexer::{operator::OperatorToken, tokenizer::Tokenizer};

#[test]
pub fn plus() {
    let text = r#"+"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Plus.into()]);
}
