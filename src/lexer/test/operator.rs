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
