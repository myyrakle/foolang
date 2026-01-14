#![allow(clippy::match_like_matches_macro)]
#![allow(dead_code)]

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
pub mod platforms;
mod utils;

use clap::Parser;

use crate::error::Errors;

fn setup_logging() {
    unsafe {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<(), Errors> {
    setup_logging();

    let command = Command::parse();

    match command.action {
        SubCommand::Build(action) => {
            let result = execute_build(action).await?;
            log::debug!("executable: {}", result.executable_filename);
        }
    }

    Ok(())
}
