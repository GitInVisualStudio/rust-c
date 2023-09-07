use std::io::Error;
use std::rc::Rc;

use super::array_expression::ArrayExpression;
use super::assignment::Assignment;
use super::data_type::{DataType, Struct};
use super::function_call::FunctionCall;
use super::generator::Generator;
use super::scope::{IScope, Scope};
use super::struct_expression::SturctExpression;
use super::type_definition::TypeDefinition;
use super::type_expression::TypeExpression;
use super::variable::Variable;
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

#[derive(Debug, PartialEq)]
pub enum UnaryOps {
    NEG,
    LOGNEG,
    REF,
    DEREF,
    COMPLEMENT,
    CAST(DataType),
}

#[derive(Debug)]
pub enum Expression {
    LongLiteral(i64),
    IntLiteral(i32),
    CharLiteral(u8),
    FunctionCall(Rc<FunctionCall>),
    ArrayExpression(Rc<ArrayExpression>),
    StructExpresion(Rc<SturctExpression>),
    Assignment(Rc<Assignment>),
    TypeExpression(Rc<TypeExpression>),
    FieldAccess {
        offset: usize,
        data_type: DataType,
        operand: Rc<Expression>,
    },
    Indexing {
        index: Rc<Expression>,
        operand: Rc<Expression>,
    },
    NamedVariable {
        stack_offset: usize,
        data_type: DataType,
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
            } => match data_type {
                DataType::STRUCT(_) => gen.lea(
                    Reg::STACK {
                        offset: *stack_offset,
                    },
                    Reg::current(),
                ),
                x => {
                    Reg::set_size(x.size());
                    gen.mov(
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                        Reg::current(),
                    )
                }
            },
            Expression::Unary {
                expression,
                operation,
            } => match operation {
                UnaryOps::NEG => {
                    expression.generate(gen)?;
                    let reg = Reg::current();
                    gen.emit_sins("neg", reg)
                }
                UnaryOps::COMPLEMENT => {
                    expression.generate(gen)?;
                    let reg = Reg::current();
                    gen.emit_sins("not", reg)
                }
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
                UnaryOps::REF => match expression.as_ref() {
                    Expression::NamedVariable {
                        stack_offset,
                        data_type: _,
                    } => gen.lea(
                        Reg::STACK {
                            offset: *stack_offset,
                        },
                        Reg::current(),
                    ),
                    _ => panic!("should not happen!"),
                },
                UnaryOps::DEREF => {
                    let base_data_type = match expression.data_type() {
                        DataType::PTR(x) => x,
                        _ => panic!("cannot get base data-type from index"),
                    };
                    match base_data_type.as_ref() {
                        DataType::STRUCT(_) => expression.generate(gen),
                        _ => {
                            expression.generate(gen)?;
                            let address = Reg::current().as_address();
                            Reg::set_size(base_data_type.size());
                            gen.mov(address, Reg::current())
                        }
                    }
                }
                UnaryOps::CAST(_) => expression.generate(gen),
            },
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
                Reg::set_size(self.data_type().size());
                match *operation {
                    BinaryOps::ADD => gen.add(second_reg, first_reg)?,
                    BinaryOps::SUB => gen.sub(second_reg, first_reg)?,
                    BinaryOps::MUL => gen.mul(second_reg, first_reg)?,
                    BinaryOps::DIV => {
                        gen.mov(second_reg, Reg::RBX)?;
                        gen.mov(first_reg, Reg::RAX)?;
                        gen.cdq()?;
                        gen.emit_sins("idiv", Reg::RBX)?;
                        gen.mov(Reg::RAX, Reg::current())?
                    }
                    BinaryOps::MOD => {
                        gen.mov(second_reg, Reg::RBX)?;
                        gen.mov(first_reg, Reg::RAX)?;
                        gen.cdq()?;
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
            Expression::ArrayExpression(arr) => arr.generate(gen),
            Expression::Indexing { index, operand } => {
                let base_data_type = match operand.data_type() {
                    DataType::PTR(x) => x,
                    _ => panic!("cannot get base data-type from index"),
                };

                index.generate(gen)?;
                let index = Reg::push();
                operand.generate(gen)?;
                let address = Reg::pop();

                Reg::set_size(8);
                gen.mul(Reg::IMMEDIATE(self.data_type().size() as i64), index)?;
                gen.add(index, address)?;

                match base_data_type.as_ref() {
                    DataType::STRUCT(_) => {
                        Reg::set_size(base_data_type.size());
                        gen.mov(address, Reg::current())
                    }
                    _ => {
                        let address = address.as_address();
                        Reg::set_size(base_data_type.size());
                        gen.mov(address, Reg::current())
                    }
                }
            }
            Expression::Assignment(assignment) => assignment.generate(gen),
            Expression::TypeExpression(expression) => expression.generate(gen),
            Expression::FieldAccess {
                offset,
                data_type,
                operand,
            } => {
                let offset = *offset;
                match data_type {
                    DataType::STRUCT(_) => {
                        operand.generate(gen)?;
                        gen.add(Reg::IMMEDIATE(offset as i64), Reg::current())
                    }
                    data_type => {
                        operand.generate(gen)?;
                        Reg::set_size(data_type.size());
                        gen.mov(Reg::current().as_address().offset(offset), Reg::current())
                    }
                }
            }
            Expression::StructExpresion(expr) => expr.generate(gen),
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
                    gen.jne(&second_expression_label)?;
                    gen.jmp(&end_label)
                }
                BinaryOps::OR => {
                    gen.cmp(Reg::IMMEDIATE(0), first_reg)?;
                    gen.je(&second_expression_label)?;
                    gen.jmp(&end_label)
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
            Token::LCURL => {
                lexer.next();
                match lexer.peek() {
                    Token::DOT => Ok(Rc::new(Self::StructExpresion(SturctExpression::parse(
                        lexer, scope,
                    )?))),
                    _ => Ok(Rc::new(Self::ArrayExpression(ArrayExpression::parse(
                        lexer, scope,
                    )?))),
                }
            }
            Token::STRINGLIT => Ok(Rc::new(Self::ArrayExpression(ArrayExpression::parse(
                lexer, scope,
            )?))),
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
                lexer.next();
                let contains: Option<Rc<Struct>> = scope.get(lexer.last_string());
                lexer.set_back(lexer.last_string().len());
                if let Some(_) = contains {
                    Ok(Rc::new(Self::TypeExpression(TypeExpression::parse(
                        lexer, scope,
                    )?)))
                } else {
                    let contains: Option<Rc<TypeDefinition>> = scope.get(lexer.last_string());
                    if let Some(_) = contains {
                        Ok(Rc::new(Self::TypeExpression(TypeExpression::parse(
                            lexer, scope,
                        )?)))
                    } else {
                        let name = lexer.expect(Token::IDENT)?.to_string();

                        match lexer.peek() {
                            Token::LPAREN => Ok(Rc::new(Self::FunctionCall(
                                FunctionCall::parse_name(name, lexer, scope)?,
                            ))),
                            _ => {
                                let contains: Option<Rc<Variable>> = scope.get(&name);
                                if let None = contains {
                                    return lexer.error(format!("Variable {} not found!", name));
                                }
                                let var = contains.unwrap();
                                let offset = var.offset();
                                Ok(Rc::new(Self::NamedVariable {
                                    stack_offset: offset,
                                    data_type: var.data_type(),
                                }))
                            }
                        }
                    }
                }
            }
            Token::LPAREN => {
                lexer.expect(Token::LPAREN)?;
                let result = Self::parse_expressions(lexer, scope);
                lexer.expect(Token::RPAREN)?;
                result
            }
            Token::SIZEOF => {
                lexer.next();
                lexer.expect(Token::LPAREN)?;
                let expression = Self::parse_expressions(lexer, scope)?;
                lexer.expect(Token::RPAREN)?;
                Ok(Rc::new(Self::IntLiteral(
                    expression.data_type().size() as i32
                )))
            }
            _ => Ok(Rc::new(Self::TypeExpression(TypeExpression::parse(
                lexer, scope,
            )?))),
        }
    }

    fn parse_indexing(
        mut operand: Rc<Expression>,
        lexer: &mut Lexer,
        scope: &mut Scope,
    ) -> Result<Rc<Self>, LexerError> {
        while lexer.peek() == Token::LBRACE {
            lexer.next();
            let expression = Self::parse(lexer, scope)?;
            lexer.expect(Token::RBRACE)?;
            operand = Rc::new(Self::Indexing {
                index: expression,
                operand: operand,
            });
        }
        Ok(operand)
    }

    fn parse_field_access(
        mut operand: Rc<Expression>,
        lexer: &mut Lexer,
        _: &mut Scope,
    ) -> Result<Rc<Self>, LexerError> {
        while lexer.peek() == Token::DOT {
            lexer.next();
            let data_type = operand.data_type();
            match data_type {
                DataType::STRUCT(data_type) => {
                    let name = lexer.expect(Token::IDENT)?.to_string();
                    match data_type.get(&name) {
                        Some(var) => {
                            operand = Rc::new(Self::FieldAccess {
                                offset: var.offset() - var.data_type().size(),
                                data_type: var.data_type(),
                                operand: operand,
                            })
                        }
                        None => lexer.error(format!("No field {} for {:?}", name, data_type))?,
                    };
                }
                x => lexer.error(format!("Cannot access field for datatype: {:?}", x))?,
            };
        }
        Ok(operand)
    }

    fn parse_arrow_access(
        mut operand: Rc<Expression>,
        lexer: &mut Lexer,
        _: &mut Scope,
    ) -> Result<Rc<Self>, LexerError> {
        while lexer.peek() == Token::ARROW {
            lexer.next();
            let data_type = operand.data_type();
            match data_type {
                DataType::PTR(data_type) => {
                    operand = Rc::new(Self::Unary {
                        expression: operand,
                        operation: UnaryOps::DEREF,
                    });
                    match data_type.as_ref() {
                        DataType::STRUCT(data_type) => {
                            let name = lexer.expect(Token::IDENT)?.to_string();
                            match data_type.get(&name) {
                                Some(var) => {
                                    operand = Rc::new(Self::FieldAccess {
                                        offset: var.offset() - var.data_type().size(),
                                        data_type: var.data_type(),
                                        operand: operand,
                                    })
                                }
                                None => {
                                    lexer.error(format!("No field {} for {:?}", name, data_type))?
                                }
                            };
                        }
                        x => lexer.error(format!("Cannot access field for datatype: {:?}", x))?,
                    }
                }
                _ => lexer.error("cannot deref non pointer expression!".to_string())?,
            };
        }
        Ok(operand)
    }

    fn parse_postfix(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let mut result = Self::parse_literal(lexer, scope)?;
        while lexer.peek() == Token::LBRACE
            || lexer.peek() == Token::DOT
            || lexer.peek() == Token::ARROW
        {
            result = match lexer.peek() {
                Token::LBRACE => Self::parse_indexing(result, lexer, scope),
                Token::DOT => Self::parse_field_access(result, lexer, scope),
                Token::ARROW => Self::parse_arrow_access(result, lexer, scope),
                _ => Ok(result),
            }?;
        }
        Ok(result)
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
            Token::COMPLEMENT => UnaryOps::COMPLEMENT,
            _ => panic!("Cannot parse binary operation!"),
        };
        Ok(Rc::new(Self::Unary {
            expression: e,
            operation: op,
        }))
    }

    fn parse_factor(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let can_be_cast = lexer.peek() == Token::LPAREN;

        let result = match lexer.peek() {
            Token::SUB | Token::LOGNEG | Token::MUL | Token::REF | Token::COMPLEMENT => {
                Self::parse_unary(lexer, scope)
            }
            // only literal left to parse
            _ => Self::parse_postfix(lexer, scope),
        };

        if can_be_cast {
            return match result.clone()?.as_ref() {
                Expression::TypeExpression(t) => {
                    let factor = Self::parse_factor(lexer, scope)?;
                    Ok(Rc::new(Self::Unary {
                        expression: factor,
                        operation: UnaryOps::CAST(t.data_type()),
                    }))
                }
                _ => result,
            };
        }

        result
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
                    .can_operate(second_operand.data_type())
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
        let result = Self::parse_binary(lexer, scope, &operations, 0)?;
        match lexer.peek() {
            Token::ASSIGN => Ok(Rc::new(Self::Assignment(Assignment::parse(
                &result, lexer, scope,
            )?))),
            _ => Ok(result),
        }
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
            Expression::Unary {
                expression,
                operation: op,
            } => match op {
                UnaryOps::COMPLEMENT | UnaryOps::NEG | UnaryOps::LOGNEG => expression.data_type(),
                UnaryOps::REF => DataType::PTR(Rc::new(expression.data_type())),
                UnaryOps::DEREF => match expression.data_type() {
                    DataType::PTR(x) => x.as_ref().clone(),
                    _ => panic!("cannot deref non pointer data-type expression!"),
                },
                UnaryOps::CAST(x) => x.clone(),
            },
            Expression::BinaryExpression {
                first,
                second,
                operation: _operation,
            } => {
                if first.data_type().size() > second.data_type().size() {
                    first.data_type()
                } else {
                    second.data_type()
                }
            }
            Expression::FunctionCall(call) => call.return_type(),
            Expression::ArrayExpression(arr) => arr.data_type(),
            Expression::Indexing { index: _, operand } => match operand.data_type() {
                DataType::PTR(x) => x.as_ref().clone(),
                _ => panic!("cannot deref non pointer data-type expression!"),
            },
            Expression::Assignment(assignment) => assignment.data_type(),
            Expression::TypeExpression(typeexpression) => typeexpression.data_type(),
            Expression::FieldAccess {
                offset: _,
                data_type,
                operand: _,
            } => data_type.clone(),
            Expression::StructExpresion(x) => x.data_type(),
        }
    }
}
