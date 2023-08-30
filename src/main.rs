pub mod lexer;
pub mod parser;

use parser::{generator::Generator, ASTNode};

fn main() {
    let result = parser::parse("code.c");
    match result {
        Ok(value) => {
            let gen = Generator::new("output.s");
            println!("Program: {:#?}", value);
            if let Ok(mut gen) = gen {
                let _ = value.generate(&mut gen);
            }
        }
        Err(e) => println!("{:?}", e),
    }
    // use lexer::tokens::Token;
    // let lexer = lexer::Lexer::new("code.c");
    // if let Ok(mut lexer) = lexer {
    //     while lexer.peek() != Token::EOF {
    //         println!("{:?}",lexer.next());
    //     }
    // }
}
