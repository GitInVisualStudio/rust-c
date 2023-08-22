pub mod generator;
pub mod expression;
pub mod function;
pub mod statement;
pub mod program;
pub mod scope;
pub mod variable;

use std::{fmt::Debug, io::Error, rc::Rc};
use crate::lexer::{Lexer, LexerError};

use self::{generator::Generator, program::Program, scope::Scope};

pub trait ASTNode: Debug{
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> where Self: Sized;
    /**
     * i dont like the result type here, i have to change that later on!
     */
    fn generate(&self, gen: &mut Generator) -> Result<usize, Error>;
}

pub fn parse(file_name: &str) -> Result<Rc<Program>, LexerError> {
    // i have to propagate the error message correctly!
    let mut lexer = Lexer::new(file_name).expect("was not able to open file!");
    let mut scope = Scope::new();
    scope.push();
    Program::parse(&mut lexer, &mut scope)
}