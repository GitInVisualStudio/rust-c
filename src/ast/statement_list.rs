use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{statement::Statement, ASTNode};

#[derive(Debug, Getters)]
pub struct StatementList {
    statements: Vec<Statement>,
    stack_size: usize,
}

impl ASTNode for StatementList {}

impl Parse<StatementList> for Parser<'_> {
    fn parse(&mut self) -> Result<StatementList, LexerError> {
        if self.peek() != Token::LCURL {
            let statement: Statement = self.parse()?;
            match statement {
                Statement::VariableDeclaration {
                    variable: _,
                    expression: _,
                } => self
                    .error("A depended statement may not be a variable declaration!".to_string())?,
                _ => {
                    let statements = vec![statement];
                    return Ok(StatementList {
                        statements: statements,
                        stack_size: 0,
                    });
                }
            }
        }
        self.next();
        self.scope.push();
        let mut statements: Vec<Statement> = Vec::new();
        while self.peek() != Token::RCURL {
            let statement = self.parse()?;
            statements.push(statement);
        }

        let size = self.scope.stack_size();
        self.scope.pop();
        self.next();
        Ok(StatementList {
            statements: statements,
            stack_size: size,
        })
    }
}
