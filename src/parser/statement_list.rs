use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{statement::Statement, ASTNode};

#[derive(Debug)]
pub struct StatementList {
    statements: Vec<Rc<Statement>>,
    stack_size: usize
}

impl ASTNode for StatementList {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<std::rc::Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        if lexer.peek() != Token::LCURL {
            let statements = vec![Statement::parse(lexer, scope)?];
            return Ok(Rc::new(StatementList {
                statements: statements,
                stack_size: 0
            }));
        }
        lexer.next();
        scope.push();
        let mut statements: Vec<Rc<Statement>> = Vec::new();
        while lexer.peek() != Token::RCURL {
            statements.push(Statement::parse(lexer, scope)?);
        }
        let size = scope.stack_size();
        scope.pop();
        lexer.next();
        Ok(Rc::new(StatementList {
            statements: statements,
            stack_size: size
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        for statement in &self.statements {
            statement.generate(gen)?;
        }
        Ok(0)
    }
}

impl StatementList {
    pub fn stack_size(&self) -> usize {
        self.stack_size
    }
}
