use std::io::Error;
use std::rc::Rc;

use crate::lexer::{Lexer, LexerError};
use crate::lexer::tokens::Token;
use super::ASTNode;
use super::generator::Generator;
use super::scope::{Scope, IScope};
use super::variable::Variable;

#[derive(Debug, PartialEq)]
pub enum BinaryOps {
    ADD,
    SUB,
    MUL,
    DIV,
    AND,
    OR,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE
}

#[derive(Debug)]
pub enum UnaryOps {
    NEG,
    LOGNEG
}

#[derive(Debug)]
pub enum Expression {
    Literal(i32),
    NamedVariable{stack_offset: usize},
    Unary { expression: Rc<Expression>, operation: UnaryOps },
    BinaryExpression {
        first: Rc<Expression>,
        second: Rc<Expression>,
        operation: BinaryOps
    }
}

impl ASTNode for Expression {

    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> where Self: Sized {
        Expression::parse_expressions(lexer, scope)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        match self {
            Expression::Literal(value) => {
                gen.mov(*value, "eax")
            },
            Expression::Unary { expression, operation } => {
                expression.generate(gen)?;
                match operation {
                    UnaryOps::NEG => gen.emit("\tneg\t\t%eax\n".to_string()),
                    UnaryOps::LOGNEG => {
                        gen.emit_ins("cmp ", "$0", "%eax")?;
                        gen.mov(0, "eax")?;
                        gen.emit("\tsete\t%al\n".to_string())
                    },
                }
            },
            Expression::BinaryExpression { first, second, operation } => {
                first.generate(gen)?;
                // push first expression
                gen.push("rax")?;

                if *operation == BinaryOps::AND || *operation == BinaryOps::OR {
                    return self.generate_and_or(operation, second, gen);
                }

                second.generate(gen)?;
                // pop first expression into rcx
                gen.pop("rcx")?;
                match operation {
                    BinaryOps::ADD => gen.emit_ins("add ", "%ecx", "%eax"),
                    BinaryOps::SUB => gen.emit("    sub     %eax, %ecx
    mov     %ecx, %eax
".to_string()),
                    BinaryOps::MUL => gen.emit_ins("imul", "%ecx", "%eax"),
                    BinaryOps::DIV => gen.emit("    mov     %eax, %ebx
    mov     %ecx, %eax
    cdq
    idiv    %ebx
".to_string()),
                    BinaryOps::EQ => gen.emit_cmp("sete"),
                    BinaryOps::NE => gen.emit_cmp("setne"),
                    BinaryOps::LT => gen.emit_cmp("setl"),
                    BinaryOps::GT => gen.emit_cmp("setg"),
                    BinaryOps::LE => gen.emit_cmp("setle"),
                    BinaryOps::GE => gen.emit_cmp("setge"),
                    _ => Ok(0)
                }?;
                Ok(0)
            },
            Expression::NamedVariable { stack_offset } => {
                gen.emit_ins("mov ", format!("-{}(%rbp)", stack_offset).as_str(), "%eax")?;
                Ok(0)
            },
        }
    }
}

impl Expression {

    fn generate_and_or(&self, operation: &BinaryOps, second: &Rc<Expression>, gen: &mut Generator) -> Result<usize, Error> {
        let (second_expression_label, end_label) = Generator::generate_clause_names();
        match operation {
            BinaryOps::AND => {
                gen.emit_ins("cmp ", "$0", "%eax")?;
                gen.emit(format!("\tjne\t\t{}\n", second_expression_label))?;
                gen.emit(format!("\tjmp\t\t{}\n", end_label))
            },
            BinaryOps::OR => {
                gen.emit_ins("cmp ", "$0", "%eax")?;
                gen.emit(format!("\tje\t\t{}\n", second_expression_label))?;
                gen.mov(1, "eax")?;
                gen.emit(format!("\tjmp\t\t{}\n", end_label))
            },
            _ => panic!("Wrong operation for boolean comparision!")
        }?;
        gen.emit_label(&second_expression_label)?;
        second.generate(gen)?;

        gen.emit_ins("cmp ", "$0", "%eax")?;
        gen.mov(1, "eax")?;
        gen.emit("\tsetne\t%al\n".to_string())?;
        gen.emit_label(&end_label)
    }

    fn parse_literal(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        match lexer.peek() {
            Token::INTLITERAL => {
                let value: i32 = lexer.expect(Token::INTLITERAL)?.trim_start().parse().expect("was not able to parse int literal");
                Ok(Rc::new(Self::Literal(value)))
            }
            Token::IDENT => {
                let name = lexer.expect(Token::IDENT)?.trim_start().to_string();
                let contains: Option<&Variable> = scope.get(&name);
                if let None = contains {
                    return lexer.error(format!("Variable {} not found!", name))
                }
                Ok(Rc::new(Self::NamedVariable { stack_offset: contains.as_ref().unwrap().offset() }))
            }
            token => panic!("No literal for {:?}", token)
        }
    }

    fn parse_unary(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let op = match lexer.next() {
            Token::SUB => UnaryOps::NEG,
            Token::LOGNEG => UnaryOps::LOGNEG,
            _ => panic!("Cannot parse binary operation!")
        };
        let e = Self::parse_factor(lexer, scope)?;
        Ok(Rc::new(Self::Unary { expression: e, operation: op}))
    }

    fn parse_factor(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        match lexer.peek() {
            Token::SUB | Token::LOGNEG => Self::parse_unary(lexer, scope),
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

    fn parse_binary(lexer: &mut Lexer, scope: &mut Scope, operations: &[Vec<Token>], index: usize) -> Result<Rc<Self>, LexerError>{
        let op = operations.get(index);
        // if we are at the end of the binary operations we parse a factor
        if op.is_none() {
            return Self::parse_factor(lexer, scope)
        }
        let op = op.unwrap();
        let mut expression = Self::parse_binary(lexer, scope, operations, index + 1)?;

        while let Some(operand) = op.iter().find(|x| lexer.peek() == **x) {
            lexer.next();
            let first_operand = expression;
            let second_operand = Self::parse_binary(lexer, scope, operations, index + 1)?;
            expression = match *operand {
                Token::ADD => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::ADD }),
                Token::SUB => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::SUB }),
                Token::MUL => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::MUL }),
                Token::DIV => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::DIV }),
                Token::AND => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::AND }),
                Token::OR => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::OR }),
                Token::EQ => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::EQ }),
                Token::NE => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::NE }),
                Token::LT => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::LT }),
                Token::GT => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::GT }),
                Token::LE => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::LE }),
                Token::GE => Rc::new(Self::BinaryExpression{ first: first_operand, second: second_operand, operation: BinaryOps::GE }),
                // this should not happen!
                _ => panic!("Unknown operation")
            }
        }
        Ok(expression)
    }

    fn parse_expressions(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let operations = [
            vec![Token::OR],
            vec![Token::AND],
            vec![Token::EQ, Token::NE],
            vec![Token::GT, Token::GE, Token::LT, Token::LE],
            vec![Token::ADD, Token::SUB],
            vec![Token::MUL, Token::DIV]
        ];
        Self::parse_binary(lexer, scope, &operations, 0)
    }

}