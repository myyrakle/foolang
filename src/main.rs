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
mod llvm;

use clap::Parser;

const LLVM_IP_SAMPLE: &str = r#"
; 문자열 상수를 전역상수처럼 선언함
@.str = private unnamed_addr constant [13 x i8] c"hello world\0A\00"

; puts 함수의 외부 선언
declare i32 @puts(i8* nocapture) nounwind

; main 함수 정의
define i32 @main()
{   
     ; i32()*
     ; [13 x i8]를 i8 *로 변환함...

    %cast210 = getelementptr [13 x i8],[13 x i8]* @.str, i64 0, i64 0

    ; puts 함수를 호출해서 stdout에 문자열을 출력함
    call i32 @puts(i8* %cast210)

    ret i32 0
}

; 이름 붙인 메타데이터
!0 = !{i32 42, null, !"string"}
!foo = !{!0} )
"#;

#[tokio::main]
async fn main() -> Result<(), AllError> {
    println!("Hello, world!");

    unsafe {
        // let llvm_context = llvm::bindings::LLVMContext::new();
    }

    // let command = Command::parse();

    // match command.action {
    //     SubCommand::Build(action) => {
    //         let executable_filename = execute_build(action).await?;
    //         println!("executable: {}", executable_filename);
    //     }
    // }

    Ok(())
}
