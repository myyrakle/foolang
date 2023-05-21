use command::{Command, SubCommand};
use error::all_error::AllError;

use crate::lexer::tokenizer::Tokenizer;

mod ast;
mod builder;
mod codegen;
mod command;
mod constant;
mod error;
mod lexer;
mod parser;
mod utils;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), AllError> {
    let command = Command::parse();

    match command.action {
        SubCommand::Build(action) => {
            let text = if let Ok(text) = tokio::fs::read_to_string(&action.value.filename).await {
                text
            } else {
                return Err(AllError::FileNotFound(action.value.filename));
            };

            let tokens = Tokenizer::string_to_tokens(text)?;

            let mut parser = parser::Parser::new();
            parser.set_tokens(tokens);
            let statements = parser.parse()?;

            let mut codegen = codegen::CodeGenerator::new();
            codegen.set_statements(statements);
            // let code = codegen.generate()?;
        }
    }

    Ok(())
}
