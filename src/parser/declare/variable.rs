use crate::{
    ast::statement::{define_variable::VariableDefinitionStatement, Statement},
    error::{parser_error::ParserError, Errors},
    lexer::{keyword::Keyword, operator::OperatorToken, primary::PrimaryToken, token::Token},
    parser::{Parser, ParserContext},
};

impl Parser {
    pub(crate) fn parse_declare_variable(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, Errors> {
        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(2, "Unexpected end of tokens".to_string()).into());
        };

        match current_token {
            Token::Keyword(Keyword::Let) => {
                let statement = self.parse_let_variable(_context)?;
                Ok(statement)
            }
            Token::Keyword(Keyword::Mut) => {
                let statement = self.parse_mut_variable(_context)?;
                Ok(statement)
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub(crate) fn parse_let_variable(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, Errors> {
        // eat let
        self.next();

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(3, "Unexpected end of tokens".to_string()).into());
        };

        let variable_name =
            if let Token::Primary(PrimaryToken::Identifier(identifier)) = current_token {
                identifier
            } else {
                return Err(ParserError::new(
                    4,
                    format!(
                        "Expected identifier for variable name. but found {:?}",
                        current_token
                    ),
                )
                .into());
            };

        self.next();

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(ParserError::new(5, "Unexpected end of tokens".to_string()).into());
        };

        match current_token {
            Token::Operator(OperatorToken::Assign) => {
                self.next();

                let expression = self.parse_expression(ParserContext::new())?;

                let statement = VariableDefinitionStatement {
                    name: variable_name,
                    value: Some(expression),
                    mutable: false,
                }
                .into();

                Ok(statement)
            }
            _ => Err(ParserError::new(
                6,
                format!(
                    "Expected = for variable assignment. but found {:?}",
                    current_token
                ),
            )
            .into()),
        }
    }

    pub(crate) fn parse_mut_variable(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, Errors> {
        // eat mut
        self.next();

        todo!()
    }
}
