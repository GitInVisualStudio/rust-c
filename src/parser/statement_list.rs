use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{statement::Statement, ASTNode};

#[derive(Debug)]
pub struct StatementList {
    statements: Vec<Rc<Statement>>
}

impl ASTNode for StatementList {
    
    fn parse(lexer: &mut crate::lexer::Lexer, scope: &mut super::scope::Scope) -> Result<std::rc::Rc<Self>, crate::lexer::LexerError> where Self: Sized {
        lexer.expect(Token::LCURL)?;
        scope.push();
        let mut statements: Vec<Rc<Statement>> = Vec::new();
        while lexer.peek() != Token::RCURL {
            statements.push(Statement::parse(lexer, scope)?);
        }
        scope.pop();
        lexer.next();
        Ok(Rc::new( StatementList { statements: statements }))    
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        for statement in &self.statements {
            statement.generate(gen)?;
        }
        Ok(0)
    }
}