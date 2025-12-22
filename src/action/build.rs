use crate::{
    codegen::CodeGenerator, command::action::build, error::all_error::AllError, ir::IRCompiler,
    lexer::tokenizer::Tokenizer, parser::Parser,
};

#[derive(Debug)]
pub struct ExecuteBuildResult {
    pub executable_filename: String,
}

pub(crate) async fn execute_build(action: build::Action) -> Result<ExecuteBuildResult, AllError> {
    let text = if let Ok(text) = tokio::fs::read_to_string(&action.value.filename).await {
        text
    } else {
        return Err(AllError::FileNotFound(action.value.filename));
    };

    // Lexing
    let tokens = Tokenizer::string_to_tokens(text)?;

    // Parsing (into AST)
    let mut parser = Parser::new();
    parser.set_tokens(tokens);
    let statements = parser.parse()?;

    // Code Generation (from AST to IR)
    let mut codegen = CodeGenerator::new();
    codegen.set_statements(statements);
    let codes = codegen.generate()?;

    // IR Compilation (from IR to executable)
    let ir_compiler = IRCompiler::new();

    let mut compiled_objects = vec![];
    // TODO: Parallelize this loop
    for code_unit in codes {
        let compiled_object = ir_compiler.compile(code_unit)?;
        compiled_objects.push(compiled_object);
    }

    let linked_object = ir_compiler.link(compiled_objects)?;

    // TODO: abstract filesystem operations
    tokio::fs::write(&action.value.filename, format!("{:?}", linked_object))
        .await
        .map_err(|e| AllError::IOError(e.to_string()))?;

    let result = ExecuteBuildResult {
        executable_filename: action.value.filename,
    };

    Ok(result)
}
