pub mod lexer;

use lexer::Lexer;

use crate::lexer::tokens::Token;

fn main() {
    let mut lexer = Lexer::new("code.c").expect("Was not able to open file");
    while let token = lexer.next() {
        if token == Token::EOF {
            break;
        }
        println!("{:?}", token);
    }
}
