pub mod generator;
pub mod expression;
pub mod function;
pub mod statement;
pub mod program;

use std::{fmt::Debug, io::Error};
use crate::lexer::{Lexer, LexerError};

use self::{generator::Generator, program::Program};
pub trait ASTNode: Debug {
    fn parse(lexer: &mut Lexer) -> Result<Self, LexerError> where Self: Sized;
    /**
     * i dont like the result type here, i have to change that later on!
     */
    fn generate(&self, gen: &mut Generator) -> Result<usize, Error>;
}

pub fn parse(file_name: &str) -> Result<Program, LexerError> {
    // i have to propagate the error message correctly!
    let mut lexer = Lexer::new(file_name).expect("was not able to open file!");
    Program::parse(&mut lexer)
}