use std::io::Error;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::generator::Generator;
use super::scope::Scope;


#[derive(Debug)]
pub enum BinaryOps {
    ADD,
    SUB,
    MUL,
    DIV
}

#[derive(Debug)]
pub enum UnaryOps {
    NEG,
    LOGNEG
}

#[derive(Debug)]
pub enum Expression {
    Literal(i32),
    Unary { expression: Box<Expression>, operation: UnaryOps },
    BinaryExpression {
        first: Box<Expression>,
        second: Box<Expression>,
        operation: BinaryOps
    }
}

impl ASTNode for Expression {

    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> where Self: Sized {
        Expression::parse_expressions(lexer, scope)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        match self {
            Expression::Literal(value) => {
                gen.mov(*value, "eax")
            },
            Expression::Unary { expression, operation } => todo!(),
            Expression::BinaryExpression { first, second, operation } => {
                first.generate(gen)?;
                // push first expression
                gen.push("rax")?;
                second.generate(gen)?;
                // pop first expression into rcx
                gen.pop("rcx")?;
                match operation {
                    BinaryOps::ADD => gen.emit_ins("addl", "%ecx", "%eax"),
                    BinaryOps::SUB => gen.emit("    subl    %eax, %ecx
    movl    %ecx, %eax
".to_string()),
                    BinaryOps::MUL => gen.emit_ins("imul", "%ecx", "%eax"),
                    BinaryOps::DIV => gen.emit("    movl    %eax, %ebx
    movl    %ecx, %eax
    cdq
    idivl   %ebx
".to_string()),
                }?;
                Ok(0)
            },
        }
    }
}

impl Expression {
    fn parse_literal(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        let value: i32 = lexer.expect(Token::INTLITERAL)?.trim_start().parse().expect("was not able to parse int literal");
        Ok(Self::Literal(value))
    }

    fn parse_unary(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        lexer.next();
        let e = Self::parse_factor(lexer, scope)?;
        Ok(Self::Unary { expression: Box::new(e), operation: UnaryOps::NEG })
    }

    fn parse_factor(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        match lexer.peek() {
            Token::SUB => Self::parse_unary(lexer, scope),
            Token::LPAREN => {
                lexer.expect(Token::LPAREN)?;
                let result = Self::parse_expressions(lexer, scope);
                lexer.expect(Token::RPAREN)?;    
                result
            },
            // only literal left to parse
            _ => Self::parse_literal(lexer, scope)
        }
    }

    fn parse_binary(lexer: &mut Lexer, scope: &mut Scope, operations: &Vec<Token>, index: usize) -> Result<Self, LexerError>{
        let op = operations.get(index);
        // if we are at the end of the binary operations we parse a factor
        if op.is_none() {
            return Self::parse_factor(lexer, scope)
        }
        let op = op.unwrap();
        let mut expression = Self::parse_binary(lexer, scope, operations, index + 1)?;
        while lexer.peek() == *op {
            lexer.next();
            let first_operand = Box::new(expression);
            let second_operand = Box::new(Self::parse_binary(lexer, scope, operations, index + 1)?);
            expression = match *op {
                Token::ADD => Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::ADD },
                Token::SUB => Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::SUB },
                Token::MUL => Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::MUL },
                Token::DIV => Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::DIV },
                // this should not happen!
                _ => panic!("Unknown operation")
            }
        }
        Ok(expression)
    }

    fn parse_expressions(lexer: &mut Lexer, scope: &mut Scope) -> Result<Self, LexerError> {
        let operations = vec![Token::ADD, Token::SUB, Token::MUL, Token::DIV];
        Self::parse_binary(lexer, scope, &operations, 0)
    }

}