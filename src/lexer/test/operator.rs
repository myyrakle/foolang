#![cfg(test)]

use crate::lexer::{operator::OperatorToken, tokenizer::Tokenizer};

#[test]
pub fn assign() {
    let text = r#"="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Assign.into()]);
}

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
pub fn left_shift() {
    let text = r#"<<"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::LeftShift.into()]);
}

#[test]
pub fn left_shift_assign() {
    let text = r#"<<="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::LeftShiftAssign.into()]);
}

#[test]
pub fn right_shift() {
    let text = r#">>"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::RightShift.into()]);
}

#[test]
pub fn right_shift_assign() {
    let text = r#">>="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::RightShiftAssign.into()]);
}

#[test]
pub fn equal() {
    let text = r#"=="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Equal.into()]);
}

#[test]
pub fn not_equal() {
    let text = r#"!="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::NotEqual.into()]);
}

#[test]
pub fn less_than() {
    let text = r#"<"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::LessThan.into()]);
}

#[test]
pub fn less_than_or_equal() {
    let text = r#"<="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::LessThanOrEqual.into()]);
}

#[test]
pub fn greater_than() {
    let text = r#">"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::GreaterThan.into()]);
}

#[test]
pub fn greater_than_or_equal() {
    let text = r#">="#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::GreaterThanOrEqual.into()]);
}

#[test]
pub fn and() {
    let text = r#"&&"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::And.into()]);
}

#[test]
pub fn or() {
    let text = r#"||"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Or.into()]);
}

#[test]
pub fn not() {
    let text = r#"!"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Not.into()]);
}

#[test]
pub fn dot() {
    let text = r#"."#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Dot.into()]);
}

#[test]
pub fn range() {
    let text = r#".."#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Range.into()]);
}

#[test]
pub fn question() {
    let text = r#"?"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Question.into()]);
}
