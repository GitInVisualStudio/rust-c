pub mod lexer;

use lexer::Lexer;

use crate::lexer::tokens::Token;

fn main() {
    let mut lexer = Lexer::new("code.c").expect("Was not able to open file");
    let result = lexer.expect(Token::ERR);
    match result {
        Ok(value) => println!("Value: {:?}", value),
        Err(e) => println!("{}", e)
    }
}
