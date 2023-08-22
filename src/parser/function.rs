use std::io::Error;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::generator::Generator;
use super::statement::Statement;


#[derive(Debug)]
pub struct Function {
    statements: Vec<Statement>,
    name: String
}

impl ASTNode for Function {
    fn parse(lexer: &mut Lexer) -> Result<Self, LexerError> where Self: Sized {
        let mut statements: Vec<Statement> = Vec::new();
        lexer.expect(Token::INT)?;
        let name = lexer.expect(Token::IDENT)?.to_string();
        lexer.expect_tokens(&[Token::LPAREN, Token::RPAREN, Token::LCURL])?;
        while lexer.peek() != Token::RCURL {
            let statement = Statement::parse(lexer)?;
            statements.push(statement);
        }
        lexer.expect(Token::RCURL)?;
        Ok(Function {statements: statements, name: name})
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        //right now we don't have to worry about stack size
        gen.emit(format!("{}:\n",self.name))?;
        for child in &self.statements {
            child.generate(gen)?;
        }
        Ok(0)
    }
}