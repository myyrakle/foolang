#![cfg(test)]

use crate::lexer::{
    general::GeneralToken, operator::OperatorToken, primary::PrimaryToken, tokenizer::Tokenizer,
};

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

#[test]
pub fn binary_expression_more() {
    let text = r#"1+20*55"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(
        tokens,
        vec![
            PrimaryToken::Integer(1).into(),
            OperatorToken::Plus.into(),
            PrimaryToken::Integer(20).into(),
            OperatorToken::Star.into(),
            PrimaryToken::Integer(55).into()
        ]
    );
}

#[test]
pub fn parentheses_expression() {
    let text = r#"1+(20*55)"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(
        tokens,
        vec![
            PrimaryToken::Integer(1).into(),
            OperatorToken::Plus.into(),
            GeneralToken::LeftParentheses.into(),
            PrimaryToken::Integer(20).into(),
            OperatorToken::Star.into(),
            PrimaryToken::Integer(55).into(),
            GeneralToken::RightParentheses.into()
        ]
    );
}
