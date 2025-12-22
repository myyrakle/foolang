#![allow(clippy::match_like_matches_macro)]

use action::build::execute_build;
use command::{Command, SubCommand};

mod action;
mod ast;
mod codegen;
mod command;
mod constant;
mod error;
mod ir;
mod lexer;
mod parser;
mod utils;

use clap::Parser;

use crate::error::Errors;

#[tokio::main]
async fn main() -> Result<(), Errors> {
    let command = Command::parse();

    match command.action {
        SubCommand::Build(action) => {
            let result = execute_build(action).await?;
            println!("executable: {}", result.executable_filename);
        }
    }

    Ok(())
}
