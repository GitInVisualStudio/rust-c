pub mod lexer;
pub mod parser;

use parser::{ASTNode, generator::Generator};

fn main() {
    let result = parser::parse("code.c");
    match result {
        Ok(value) => {
            let gen = Generator::new("output.s");
            println!("Program: {:?}", value);
            if let Ok(mut gen) = gen {
                let _ = value.generate(&mut gen);
            }
        }
        Err(e) => println!("{:?}", e),
    }
}
