use crate::{
    ast::{
        expression::{variable::VariableExpression, Expression},
        statement::{define_variable::VariableDefinitionStatement, Statement},
    },
    error::all_error::AllError,
    lexer::{keyword::Keyword, operator::OperatorToken, primary::PrimaryToken, token::Token},
    parser::{Parser, ParserContext},
};

impl Parser {
    pub(crate) fn parse_declare_variable(
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
    ) -> Result<Statement, AllError> {
        // eat let
        self.next();

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
        };

        let variable_name =
            if let Token::Primary(PrimaryToken::Identifier(identifier)) = current_token {
                identifier
            } else {
                return Err(AllError::ParserError(format!(
                    "Expected identifier for variable name. but found {:?}",
                    current_token
                )));
            };

        self.next();

        let current_token = if let Some(token) = self.get_current_token() {
            token
        } else {
            return Err(AllError::ParserError(
                "Unexpected end of tokens".to_string(),
            ));
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
            _ => {
                return Err(AllError::ParserError(format!(
                    "Expected = for variable assignment. but found {:?}",
                    current_token
                )));
            }
        }
    }

    pub(crate) fn parse_mut_variable(
        &mut self,
        _context: ParserContext,
    ) -> Result<Statement, AllError> {
        // eat mut
        self.next();

        todo!()
    }
}
