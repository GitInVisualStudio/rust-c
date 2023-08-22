use std::io::Error;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::expression::Expression;
use super::generator::Generator;
use super::scope::Scope;


#[derive(Debug)]
pub enum Statement {
    Return {
        expression: Option<Box<dyn ASTNode>>
    },
    VariableDeclaration {
        name: String,
        expression: Option<Box<dyn ASTNode>>
    }
}

impl ASTNode for Statement {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> where Self: Sized {
        let result = match lexer.peek() {
            Token::INT => Self::parse_variable_declaration(lexer, scope),
            Token::RETURN => Self::parse_return(lexer, scope),
            _ => lexer.error("Cannot parse statement".to_string())
        };
        lexer.expect(Token::SEMIC)?;
        result
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        match self {
            Statement::Return { expression } => {
                if expression.is_some() {
                    expression.as_ref().unwrap().generate(gen)?;
                }
                gen.emit("\tret\n".to_string())?;
                Ok(0)
            },
            // will care about that later
            Statement::VariableDeclaration { name, expression } => todo!(),
        }
    }

}

impl Statement {
    fn parse_variable_declaration(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        lexer.expect(Token::INT)?;
        let name = lexer.expect(Token::IDENT)?.to_string();
        match lexer.peek() {
            Token::ASSIGN => {
                lexer.next();
                let expression = Expression::parse(lexer, scope)?;
                Ok (Statement::VariableDeclaration { name: name, expression: Some(Box::new(expression)) })
            }
            _ => Ok (Statement::VariableDeclaration { name: name, expression: None })
        }
    }

    fn parse_return(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        lexer.expect(Token::RETURN)?;
        let expression = Expression::parse(lexer, scope)?;
        Ok(Statement::Return {
            expression: Some(Box::new(expression))
        })
    }
}