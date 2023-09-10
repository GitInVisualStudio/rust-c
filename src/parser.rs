pub mod array_expression;
pub mod assignment;
pub mod data_type;
pub mod expression;
pub mod for_statement;
pub mod function;
pub mod function_call;
pub mod generator;
pub mod if_statement;
pub mod program;
pub mod scope;
pub mod statement;
pub mod statement_list;
pub mod struct_expression;
pub mod type_definition;
pub mod type_expression;
pub mod variable;
pub mod while_statement;

use crate::lexer::{Lexer, LexerError};
use std::{
    fmt::Debug,
    fs::File,
    io::{Error, Read},
    rc::Rc,
};

use self::{generator::Generator, program::Program, scope::Scope};

pub trait ASTNode: Debug {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized;
    /**
     * i dont like the result type here, i have to change that later on!
     */
    fn generate(&self, gen: &mut Generator) -> Result<usize, Error>;
}

pub fn parse(file_name: &str) -> Result<Rc<Program>, LexerError> {
    let mut file = File::open(file_name).expect("cannot open file!");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("error while reading file!");

    let mut lexer = Lexer::new(&content);
    let mut scope = Scope::new();
    scope.push();
    Program::parse(&mut lexer, &mut scope)
}
