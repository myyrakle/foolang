pub mod context;
pub use context::ParserContext;

pub mod expression;
pub use expression::*;

pub mod variable;
pub use variable::*;

use crate::{
    ast::statement::Statement,
    error::all_error::AllError,
    lexer::{keyword::Keyword, token::Token},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize, // index of current token
    context: ParserContext,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            current: 0,
            context: ParserContext::new(),
        }
    }

    pub fn set_tokens(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens;
    }

    pub fn prev(&mut self) {
        self.current -= 1;
    }

    pub fn next(&mut self) {
        self.current += 1;
    }

    pub fn get_current_token(&self) -> Option<Token> {
        self.tokens.get(self.current).map(|e| e.to_owned())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Statement>, AllError> {
        let mut statements = vec![];

        // top-level parser loop
        loop {
            if let Some(current_token) = self.get_current_token() {
                match current_token {
                    Token::Keyword(Keyword::Let | Keyword::Const) => {
                        let statement = self.parse_variable_definition(self.context.clone())?;
                        statements.push(statement);
                    }
                    Token::Primary(_) => {
                        let statement = self.parse_expression(self.context.clone())?;
                        statements.push(statement);
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        Ok(statements)
    }
}
