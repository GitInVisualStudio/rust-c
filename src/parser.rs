
use std::rc::Rc;

use crate::{
    ast::{data_type::DataType, expression::Expression},
    lexer::{tokens::Token, Lexer, LexerError},
};

use self::scope::Scope;

pub mod scope;

pub trait Parse<T> {
    fn parse(&mut self) -> Result<T, LexerError>;
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub scope: Scope,
    pub assignee: Option<Rc<Expression>>,
    label_index: usize,
}

impl Parser<'_> {
    pub fn new<'a>(content: &'a str) -> Parser<'a> {
        let lexer = Lexer::new(&content);
        let scope = Scope::new();
        Parser {
            lexer: lexer,
            scope: scope,
            assignee: None,
            label_index: 0,
        }
    }

    pub fn check_data_types(&self, from: &DataType, to: &DataType) -> Result<bool, LexerError> {
        if *from != *to && !from.can_convert(to.clone()) {
            return self
                .lexer
                .error(format!("Cannot convert {:?} to {:?}!", from, to));
        }
        Ok(true)
    }

    pub fn peek(&mut self) -> Token {
        self.lexer.peek()
    }

    pub fn peek_str(&mut self) -> &str {
        self.lexer.peek_str()
    }

    pub fn next(&mut self) -> Token {
        self.lexer.next()
    }

    pub fn expect(&mut self, token: Token) -> Result<&str, LexerError> {
        self.lexer.expect(token)
    }

    pub fn error<T>(&mut self, msg: String) -> Result<T, LexerError> {
        self.lexer.error(msg)
    }

    pub fn last_string(&self) -> &str {
        self.lexer.last_string()
    }

    pub fn next_label_index(&mut self) -> usize {
        self.label_index += 1;
        self.label_index
    }

    pub fn label_index(&self) -> usize {
        self.label_index
    }
}
