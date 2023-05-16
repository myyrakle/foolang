use crate::lexer::{operator::OperatorToken, tokenizer::Tokenizer};

mod ast;
mod codegen;
mod command;
mod constant;
mod error;
mod lexer;
mod parser;
mod utils;

fn main() {
    let text = r#"+"#.to_owned();

    let tokens = Tokenizer::string_to_tokens(text).unwrap();

    assert_eq!(tokens, vec![OperatorToken::Plus.into()]);

    println!("Hello, world!");
}
