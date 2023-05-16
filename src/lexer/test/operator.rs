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

#[test]
pub fn star() {
    let text = r#"*"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Star.into()]);
}

#[test]
pub fn star_assign() {
    let text = r#"*="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::StarAssign.into()]);
}

#[test]
pub fn slash() {
    let text = r#"/"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Slash.into()]);
}

#[test]
pub fn slash_assign() {
    let text = r#"/="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::SlashAssign.into()]);
}

#[test]
pub fn modulo() {
    let text = r#"%"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Modulo.into()]);
}

#[test]
pub fn modulo_assign() {
    let text = r#"%="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::ModuloAssign.into()]);
}

#[test]
pub fn ampersand() {
    let text = r#"&"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Ampersand.into()]);
}

#[test]
pub fn and_assign() {
    let text = r#"&="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::AndAssign.into()]);
}

#[test]
pub fn bitwise_or() {
    let text = r#"|"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::BitwiseOr.into()]);
}

#[test]
pub fn or_assign() {
    let text = r#"|="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::OrAssign.into()]);
}

#[test]
pub fn bitwise_xor() {
    let text = r#"^"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::BitwiseXor.into()]);
}

#[test]
pub fn xor_assign() {
    let text = r#"^="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::XorAssign.into()]);
}

#[test]
pub fn bitwise_not() {
    let text = r#"~"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::BitwiseNot.into()]);
}

#[test]
pub fn not_assign() {
    let text = r#"~="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::NotAssign.into()]);
}
