#![cfg(test)]

use crate::lexer::{general::GeneralToken, tokenizer::Tokenizer};

#[test]
pub fn arrow() {
    let text = r#"->"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![GeneralToken::Arrow.into()]);
}
