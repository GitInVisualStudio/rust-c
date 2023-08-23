use std::io::Error;
use std::rc::Rc;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::generator::Generator;
use super::scope::{Scope, IScope};
use super::statement::Statement;


#[derive(Debug)]
pub struct Function {
    statements: Vec<Rc<Statement>>,
    name: String,
    stack_size: usize
}

impl ASTNode for Function {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> where Self: Sized {
        let mut statements: Vec<Rc<Statement>> = Vec::new();
        lexer.expect(Token::INT)?;
        let name = lexer.expect(Token::IDENT)?.to_string();
        
        // check if function already exists
        let contains: Option<&Function> = scope.get(&name);
        if let Some(_) = contains {
            return lexer.error(format!("Function {} already exists!", name))
        }
        lexer.expect_tokens(&[Token::LPAREN, Token::RPAREN, Token::LCURL])?;

        scope.push();
        while lexer.peek() != Token::RCURL {
            statements.push(Statement::parse(lexer, scope)?);
        }

        lexer.expect(Token::RCURL)?;
        let result = Rc::new(Function {statements: statements, name: name, stack_size: scope.stack_size()});
        
        scope.pop();
        scope.add(result.clone());
        Ok(result)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        //right now we don't have to worry about stack size
        gen.emit(format!("{}:\n",self.name))?;
        gen.push_stack(self.stack_size)?;
        for child in &self.statements {
            child.generate(gen)?;
        }
        Ok(0)
    }
}

impl Function {
    pub fn name(&self) -> &String {
        &self.name
    }
}