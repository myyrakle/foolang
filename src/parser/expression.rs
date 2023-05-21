use crate::{ast::statement::Statement, error::all_error::AllError, lexer::token::Token};

use super::{Parser, ParserContext};

impl Parser {
    pub(super) fn parse_expression(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, AllError> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        match current_token {
            Token::Primary(primary) => {
                
            }
            _ => todo!(),
        }

        todo!()
    }
}
