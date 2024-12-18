#![allow(clippy::match_like_matches_macro)]

use action::build::execute_build;
use command::{Command, SubCommand};
use error::all_error::AllError;

mod action;
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

use libc::c_int;
#[link(name="llvm", kind="static")]
extern
{
    fn add(_: c_int, _: c_int)->c_int;
}



#[tokio::main]
async fn main() -> Result<(), AllError> {
    println!("Hello, world!");

    let a = 1;
    let b = 2;
    let c = unsafe { add(a, b) };
    println!("{} + {} = {}", a, b, c);

    // let command = Command::parse();

    // match command.action {
    //     SubCommand::Build(action) => {
    //         let executable_filename = execute_build(action).await?;
    //         println!("executable: {}", executable_filename);
    //     }
    // }

    Ok(())
}
