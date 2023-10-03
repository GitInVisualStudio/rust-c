pub mod error;
mod generator;
pub mod lexer;
pub mod parser;

use std::{env, process::ExitCode};

use bumpalo::Bump;

use lexer::Lexer;
use parser::Parser;

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
    println!("{:#?}", program);
    ExitCode::SUCCESS
    // let mut parser = Parser::new(&content);
    // if args.contains(&"-tokens".into()) {
    //     loop {
    //         let token = parser.next();
    //         println!("{:?}", token);
    //         if token == Token::EOF {
    //             break;
    //         }
    //     }
    //     return ExitCode::SUCCESS;
    // }
    // let program: Result<Program, > = parser.parse();

    // match program {
    //     Ok(program) => {
    //         let mut gen = Generator::new(output).expect("can't write to output File");
    //         if args.contains(&"-ast".into()) {
    //             println!("{:#?}", program);
    //         }
    //         let result = gen.generate(&program);
    //         match result {
    //             Ok(_) => (),
    //             Err(x) => println!("{}", x.to_string()),
    //         }
    //         ExitCode::SUCCESS
    //     }
    //     Err(e) => {
    //         println!("{:#?}", e);
    //         ExitCode::FAILURE
    //     }
    // }
}
