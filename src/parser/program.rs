use std::io::Error;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::function::Function;
use super::generator::Generator;
use super::scope::Scope;

#[derive(Debug)]
pub struct Program {
    functions: Vec<Box<dyn ASTNode>>
}

impl ASTNode for Program {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> where Self: Sized {
        let mut funcs: Vec<Box<dyn ASTNode>> = Vec::new();
        while lexer.peek() == Token::INT {
            funcs.push(Box::new(Function::parse(lexer, scope)?))
        }
        lexer.expect(Token::EOF)?;
        Ok(Program { functions: funcs })
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        gen.emit("
    .text
    .globl	main
    .type	main, @function
".to_string()
        )?;
        for child in &self.functions {
            child.generate(gen)?;
        }
        Ok(0)
    }
}