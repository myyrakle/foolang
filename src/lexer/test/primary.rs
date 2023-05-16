#![cfg(test)]

use crate::lexer::{primary::PrimaryToken, tokenizer::Tokenizer};

#[test]
pub fn integer() {
    let text = r#"123234"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![PrimaryToken::Integer(123234).into()]);
}

#[test]
pub fn float() {
    let text = r#"123.234"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![PrimaryToken::Float(123.234).into()]);
}
