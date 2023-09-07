pub mod lexer;
pub mod parser;

use parser::{generator::Generator, ASTNode};
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() < 3 {
        println!("Usage: ./rust-compiler 'code.c' 'output.s'");
        return ExitCode::FAILURE;
    }
    let code = &args[1];
    let output = &args[2];
    let result = parser::parse(code);
    match result {
        Ok(value) => {
            let gen = Generator::new(output);
            println!("Program: {:#?}", value);
            if let Ok(mut gen) = gen {
                let _ = value.generate(&mut gen);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("{:#?}", e);
            ExitCode::FAILURE
        }
    }
    // use lexer::tokens::Token;
    // let lexer = lexer::Lexer::new("code.c");
    // if let Ok(mut lexer) = lexer {
    //     while lexer.peek() != Token::EOF {
    //         println!("{:?}",lexer.next());
    //     }
    // }
    // ExitCode::SUCCESS
}
