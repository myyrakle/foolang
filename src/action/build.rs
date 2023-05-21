use crate::{
    builder::Builder, codegen::CodeGenerator, command::action::build, error::all_error::AllError,
    lexer::tokenizer::Tokenizer, parser::Parser,
};

pub(crate) async fn execute_build(action: build::Action) -> Result<String, AllError> {
    let text = if let Ok(text) = tokio::fs::read_to_string(&action.value.filename).await {
        text
    } else {
        return Err(AllError::FileNotFound(action.value.filename));
    };

    let tokens = Tokenizer::string_to_tokens(text)?;

    let mut parser = Parser::new();
    parser.set_tokens(tokens);
    let statements = parser.parse()?;

    let mut codegen = CodeGenerator::new();
    codegen.set_statements(statements);
    let codes = codegen.generate()?;

    let mut builder = Builder::new();
    builder.set_filenames(codes);
    let output = builder.build()?;

    Ok(output)
}
