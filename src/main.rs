pub mod ast;
pub mod lexer;
pub mod parser;
mod generator;

use std::{env, fs::File, io::Read, process::ExitCode};

use ast::program::Program;
use lexer::LexerError;
use parser::{Parse, Parser};

use crate::generator::Generator;

fn main() -> ExitCode {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() < 3 {
        println!("Usage: ./rust-compiler 'code.c' 'output.s'");
        return ExitCode::FAILURE;
    }
    let code = &args[1];
    let output = &args[2];
    let mut file = File::open(code).expect("can't open file!");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("can't read file");

    let mut parser = Parser::new(&content);
    let program: Result<Program, LexerError> = parser.parse();

    match program {
        Ok(program) => {
            let mut gen = Generator::new(output).expect("can't write to output File");
            // println!("Program: {:#?}", program);
            let result = gen.generate(&program);
            match result {
                Ok(_) => println!("Finished compiling!"),
                Err(x) => println!("{}", x.to_string()),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("{:#?}", e);
            ExitCode::FAILURE
        }
    }
}
