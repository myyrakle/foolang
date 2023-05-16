#![cfg(test)]

use crate::lexer::{operator::OperatorToken, tokenizer::Tokenizer};

#[test]
pub fn plus() {
    let text = r#"+"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Plus.into()]);
}

#[test]
pub fn plus_assign() {
    let text = r#"+="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::PlusAssign.into()]);
}

#[test]
pub fn minus() {
    let text = r#"-"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Minus.into()]);
}

#[test]
pub fn minus_assign() {
    let text = r#"-="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::MinusAssign.into()]);
}
