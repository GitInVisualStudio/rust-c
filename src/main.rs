pub mod error;
pub mod generator;
pub mod lexer;
pub mod parser;
pub mod scope_builder;
pub mod visitor;

use std::{env, process::ExitCode};

use bumpalo::Bump;

use lexer::Lexer;
use parser::Parser;
use scope_builder::ScopeBuilder;
use visitor::Visitable;

fn main() -> ExitCode {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() < 3 {
        println!("Usage: ./rust-compiler 'code.c' 'output.s' [-ast] [-tokens]");
        return ExitCode::FAILURE;
    }
    let code = &args[1];
    let output = &args[2];

    let bump = Bump::new();
    let content = std::fs::read_to_string(code).expect("was not able to open file!");
    let tokens = bump.alloc(Lexer::tokenize(&content));
    let mut parser = Parser::new(tokens, &bump);
    let program = parser.program();
    let mut scope_builder = ScopeBuilder::new(&bump);
    match program {
        Ok(program) => {
            let finished = program.accept(&mut scope_builder);
            match finished {
                Ok(program) => println!("{:#?}\neverything passed!", program),
                Err(e) => println!("Error building scope: {:#?}", e),
            }
        }
        Err(e) => println!("Error while parsing: {:#?}", e),
    }
    ExitCode::SUCCESS
}
