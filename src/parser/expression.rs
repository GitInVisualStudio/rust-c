use std::io::Error;
use std::rc::Rc;

use super::function_call::FunctionCall;
use super::generator::Generator;
use super::scope::{IScope, Scope};
use super::variable::{DataType, Variable};
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::{Lexer, LexerError};
use crate::parser::generator::register::Reg;

#[derive(Debug, PartialEq)]
pub enum BinaryOps {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    AND,
    OR,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
}

#[derive(Debug)]
pub enum UnaryOps {
    NEG,
    LOGNEG,
    REF,
    DEREF,
}

#[derive(Debug)]
pub enum Expression {
    LongLiteral(i64),
    IntLiteral(i32),
    CharLiteral(u8),
    FunctionCall(Rc<FunctionCall>),
    NamedVariable {
        stack_offset: usize,
        data_type: DataType,
    },
    VariableAssign {
        stack_offset: usize,
        expression: Rc<Expression>,
    },
    Unary {
        expression: Rc<Expression>,
        operation: UnaryOps,
    },
    BinaryExpression {
        first: Rc<Expression>,
        second: Rc<Expression>,
        operation: BinaryOps,
    },
}

impl ASTNode for Expression {
    fn parse(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError>
    where
        Self: Sized,
    {
        Expression::parse_expressions(lexer, scope)
    }

    fn generate(&self, gen: &mut Generator) -> Result<usize, Error> {
        match self {
            Expression::IntLiteral(value) => {
                Reg::set_size(4);
                gen.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            Expression::LongLiteral(value) => {
                Reg::set_size(8);
                gen.mov(Reg::IMMEDIATE(*value), Reg::current())
            }
            Expression::CharLiteral(value) => {
                Reg::set_size(1);
                gen.mov(Reg::IMMEDIATE(*value as i64), Reg::current())
            }
            Expression::NamedVariable {
                stack_offset,
                data_type,
            } => {
                Reg::set_size(data_type.size());
                gen.mov(
                    Reg::STACK {
                        offset: *stack_offset,
                    },
                    Reg::current(),
                )
            }
            Expression::VariableAssign {
                stack_offset,
                expression,
            } => {
                Reg::set_size(expression.data_type().size());
                expression.generate(gen)?;
                gen.mov(
                    Reg::current(),
                    Reg::STACK {
                        offset: *stack_offset,
                    },
                )
            }
            Expression::Unary {
                expression,
                operation,
            } => {
                match operation {
                    UnaryOps::NEG => {
                        expression.generate(gen)?;
                        let reg = Reg::current();
                        gen.emit_sins("neg", reg)
                    },
                    UnaryOps::LOGNEG => {
                        expression.generate(gen)?;
                        let reg = Reg::current();
                        gen.cmp(Reg::IMMEDIATE(0), reg)?;
                        gen.mov(Reg::IMMEDIATE(0), reg)?;
                        let prev = Reg::set_size(1);
                        let result = gen.emit_sins("sete", reg);
                        Reg::set_size(prev);
                        result
                    }
                    UnaryOps::REF => {
                        match expression.as_ref() {
                            Expression::NamedVariable { stack_offset, data_type: _ } => {
                                Reg::set_size(8);
                                gen.lea(Reg::STACK { offset: *stack_offset }, Reg::current())
                            },
                            _ => panic!("should not happen!")
                        }
                    },
                    UnaryOps::DEREF => {
                        expression.generate(gen)?;
                        gen.emit(&format!("\tmov \t({}),{}\n", Reg::current(), Reg::current()))
                    },
                }
            }
            Expression::BinaryExpression {
                first,
                second,
                operation,
            } => {
                if *operation == BinaryOps::AND || *operation == BinaryOps::OR {
                    return self.generate_and_or(gen);
                }

                let first_reg = Reg::current();
                first.generate(gen)?;
                Reg::push();
                second.generate(gen)?;
                let second_reg = Reg::pop();
                Reg::set_size(first.data_type().size());
                match *operation {
                    BinaryOps::ADD => gen.add(second_reg, first_reg)?,
                    BinaryOps::SUB => gen.sub(second_reg, first_reg)?,
                    BinaryOps::MUL => gen.mul(second_reg, first_reg)?,
                    BinaryOps::DIV => {
                        gen.mov(second_reg, Reg::RBX)?;
                        gen.mov(first_reg, Reg::RAX)?;
                        gen.emit("\tcdq\n")?;
                        gen.emit_sins("idiv", Reg::RBX)?;
                        gen.mov(Reg::RAX, Reg::current())?
                    }
                    BinaryOps::MOD => {
                        gen.mov(second_reg, Reg::RBX)?;
                        gen.mov(first_reg, Reg::RAX)?;
                        gen.emit("\tcdq\n")?;
                        gen.emit_sins("idiv", Reg::RBX)?;
                        gen.mov(Reg::RDX, Reg::current())?
                    }
                    BinaryOps::EQ => gen.gen_cmp("sete", second_reg, first_reg)?,
                    BinaryOps::NE => gen.gen_cmp("setne", second_reg, first_reg)?,
                    BinaryOps::LT => gen.gen_cmp("setl", second_reg, first_reg)?,
                    BinaryOps::GT => gen.gen_cmp("setg", second_reg, first_reg)?,
                    BinaryOps::LE => gen.gen_cmp("setle", second_reg, first_reg)?,
                    BinaryOps::GE => gen.gen_cmp("setge", second_reg, first_reg)?,
                    _ => panic!("Something went wrong"),
                };
                Ok(0)
            }
            Expression::FunctionCall(call) => call.generate(gen),
        }
    }
}

impl Expression {
    fn generate_and_or(&self, gen: &mut Generator) -> Result<usize, Error> {
        if let Expression::BinaryExpression {
            first,
            second,
            operation,
        } = self
        {
            let first_reg = Reg::current();
            first.generate(gen)?;

            let (second_expression_label, end_label) = Generator::generate_clause_names();
            match *operation {
                BinaryOps::AND => {
                    gen.cmp(Reg::IMMEDIATE(0), first_reg)?;
                    gen.emit(&format!("\tjne\t\t{}\n", second_expression_label))?;
                    gen.emit(&format!("\tjmp\t\t{}\n", end_label))
                }
                BinaryOps::OR => {
                    gen.cmp(Reg::IMMEDIATE(0), first_reg)?;
                    gen.emit(&format!("\tje\t\t{}\n", second_expression_label))?;
                    gen.emit(&format!("\tjmp\t\t{}\n", end_label))
                }
                _ => panic!("Wrong operation for boolean comparision!"),
            }?;
            gen.emit_label(&second_expression_label)?;
            second.generate(gen)?;
            let second_reg = Reg::current();

            gen.cmp(Reg::IMMEDIATE(0), second_reg)?;
            gen.mov(Reg::IMMEDIATE(1), second_reg)?;

            let prev = Reg::set_size(1);
            gen.emit_sins("setne", second_reg)?;
            Reg::set_size(prev);

            return gen.emit_label(&end_label);
        }
        panic!("this should not happen!");
    }

    fn parse_literal(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        match lexer.peek() {
            Token::INTLITERAL => {
                let value: i32 = lexer
                    .expect(Token::INTLITERAL)?
                    .trim_start()
                    .parse()
                    .expect("was not able to parse int literal");
                Ok(Rc::new(Self::IntLiteral(value)))
            }
            Token::CHARLITERAL => {
                let string = lexer.expect(Token::CHARLITERAL)?;
                if string.len() > 3 {
                    let string = &string[1..3];
                    return match string {
                        "\\n" => Ok(Rc::new(Self::CharLiteral(b'\n'))),
                        "\\t" => Ok(Rc::new(Self::CharLiteral(b'\t'))),
                        _ => {
                            println!("{}", string);
                            let value: u8 = string[1..2].as_bytes().first().unwrap().clone();
                            Ok(Rc::new(Self::CharLiteral(value)))
                        }
                    };
                }
                let value: u8 = string[1..2].as_bytes().first().unwrap().clone();
                Ok(Rc::new(Self::CharLiteral(value)))
            }
            Token::IDENT => {
                let name = lexer.expect(Token::IDENT)?.trim_start().to_string();

                match lexer.peek() {
                    Token::LPAREN => Ok(Rc::new(Self::FunctionCall(FunctionCall::parse_name(
                        name, lexer, scope,
                    )?))),
                    _ => {
                        let contains: Option<&Variable> = scope.get(&name);
                        if let None = contains {
                            return lexer.error(format!("Variable {} not found!", name));
                        }
                        let var = contains.unwrap();
                        let offset = var.offset();
                        if lexer.peek() == Token::ASSIGN {
                            lexer.next();
                            let expression = Expression::parse(lexer, scope)?;
                            return Ok(Rc::new(Self::VariableAssign {
                                stack_offset: offset,
                                expression: expression,
                            }));
                        }
                        Ok(Rc::new(Self::NamedVariable {
                            stack_offset: offset,
                            data_type: var.data_type(),
                        }))
                    }
                }
            }
            token => panic!("No literal for {:?}", token),
        }
    }

    fn parse_unary(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let token = lexer.next();
        let e = Self::parse_factor(lexer, scope)?;
        let op = match token {
            Token::SUB => UnaryOps::NEG,
            Token::LOGNEG => UnaryOps::LOGNEG,
            Token::REF => {
                if let Expression::NamedVariable {
                    stack_offset: _,
                    data_type: _,
                } = e.as_ref()
                {
                    UnaryOps::REF
                } else {
                    lexer.error("Cannot get address from non-variable expresion!".to_string())?
                }
            }
            Token::MUL => {
                if let DataType::PTR(_) = e.as_ref().data_type() {
                    UnaryOps::DEREF
                } else {
                    lexer.error("Cannot de-refrence non-pointer expresion!".to_string())?
                }
            }
            _ => panic!("Cannot parse binary operation!"),
        };
        Ok(Rc::new(Self::Unary {
            expression: e,
            operation: op,
        }))
    }

    fn parse_factor(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        match lexer.peek() {
            Token::SUB | Token::LOGNEG | Token::MUL | Token::REF => Self::parse_unary(lexer, scope),
            Token::LPAREN => {
                lexer.expect(Token::LPAREN)?;
                let result = Self::parse_expressions(lexer, scope);
                lexer.expect(Token::RPAREN)?;
                result
            }
            // only literal left to parse
            _ => Self::parse_literal(lexer, scope),
        }
    }

    fn parse_binary(
        lexer: &mut Lexer,
        scope: &mut Scope,
        operations: &[Vec<Token>],
        index: usize,
    ) -> Result<Rc<Self>, LexerError> {
        let op = operations.get(index);
        // if we are at the end of the binary operations we parse a factor
        if op.is_none() {
            return Self::parse_factor(lexer, scope);
        }
        let op = op.unwrap();
        let mut expression = Self::parse_binary(lexer, scope, operations, index + 1)?;

        while let Some(operand) = op.iter().find(|x| lexer.peek() == **x) {
            lexer.next();
            let first_operand = expression;
            let second_operand = Self::parse_binary(lexer, scope, operations, index + 1)?;
            if first_operand.data_type() != second_operand.data_type()
                && !first_operand
                    .data_type()
                    .can_convert(second_operand.data_type())
            {
                return lexer.error(
                    "cannot perform binary operation on 2 different data types!".to_string(),
                );
            }
            expression = match *operand {
                Token::ADD => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::ADD,
                }),
                Token::SUB => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::SUB,
                }),
                Token::MUL => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::MUL,
                }),
                Token::DIV => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::DIV,
                }),
                Token::MOD => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::MOD,
                }),
                Token::AND => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::AND,
                }),
                Token::OR => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::OR,
                }),
                Token::EQ => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::EQ,
                }),
                Token::NE => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::NE,
                }),
                Token::LT => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::LT,
                }),
                Token::GT => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::GT,
                }),
                Token::LE => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::LE,
                }),
                Token::GE => Rc::new(Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::GE,
                }),
                // this should not happen!
                _ => panic!("Unknown operation"),
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
            vec![Token::MUL, Token::DIV, Token::MOD],
        ];
        Self::parse_binary(lexer, scope, &operations, 0)
    }

    pub fn data_type(&self) -> DataType {
        match self {
            Expression::IntLiteral(_) => DataType::INT,
            Expression::CharLiteral(_) => DataType::CHAR,
            Expression::LongLiteral(_) => DataType::LONG,
            Expression::NamedVariable {
                data_type,
                stack_offset: _,
            } => data_type.clone(),
            Expression::VariableAssign {
                stack_offset: _,
                expression,
            } => expression.data_type(),
            //TODO: match deref & ref to the base type
            Expression::Unary {
                expression,
                operation: op,
            } => match op {
                UnaryOps::NEG | UnaryOps::LOGNEG => expression.data_type(),
                UnaryOps::REF => DataType::PTR(Rc::new(expression.data_type())),
                UnaryOps::DEREF => match expression.data_type() {
                    DataType::PTR(x) => x.as_ref().clone(),
                    _ => panic!("cannot deref non pointer data-type expression!"),
                },
            },
            Expression::BinaryExpression {
                first,
                second: _second,
                operation: _operation,
            } => first.data_type(),
            Expression::FunctionCall(call) => call.return_type(),
        }
    }
}
