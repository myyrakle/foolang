#![cfg(test)]

use crate::lexer::{general::GeneralToken, tokenizer::Tokenizer};

#[test]
pub fn arrow() {
    let text = r#"->"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::Arrow.into()]);
}

#[test]
pub fn comma() {
    let text = r#","#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::Comma.into()]);
}

#[test]
pub fn semicolon() {
    let text = r#";"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::SemiColon.into()]);
}

#[test]
pub fn colon() {
    let text = r#":"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::Colon.into()]);
}

#[test]
pub fn left_parentheses() {
    let text = r#"("#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::LeftParentheses.into()]);
}

#[test]
pub fn right_parentheses() {
    let text = r#")"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::RightParentheses.into()]);
}

#[test]
pub fn parentheses() {
    let text = r#"()"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(
        tokens,
        vec![
            GeneralToken::LeftParentheses.into(),
            GeneralToken::RightParentheses.into()
        ]
    );
}
