use std::io::Error;
use std::rc::Rc;

use super::function::Function;
use super::generator::Generator;
use super::scope::Scope;
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::{Lexer, LexerError};

#[derive(Debug)]
pub struct Program {
    functions: Vec<Rc<dyn ASTNode>>,
}

impl ASTNode for Program {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        let mut funcs: Vec<Rc<dyn ASTNode>> = Vec::new();
        while lexer.peek() == Token::INT {
            funcs.push(Function::parse(lexer, scope)?)
        }
        lexer.expect(Token::EOF)?;
        Ok(Rc::new(Program { functions: funcs }))
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        gen.emit(
            &"
    .text
    .globl	main
    .type	main, @function
"
            .to_string(),
        )?;
        for child in &self.functions {
            child.generate(gen)?;
        }
        Ok(0)
    }
}
