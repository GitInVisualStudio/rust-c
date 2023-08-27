use std::io::Error;
use std::rc::Rc;

use super::function_call::FunctionCall;
use super::generator::Generator;
use super::scope::{IScope, Scope};
use super::variable::{DataType, Variable};
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::{Lexer, LexerError};

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
}

#[derive(Debug)]
pub enum Expression {
    Literal(i32),
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
            Expression::Literal(value) => gen.mov(*value, "eax"),
            Expression::Unary {
                expression,
                operation,
            } => {
                expression.generate(gen)?;
                match operation {
                    UnaryOps::NEG => gen.emit("\tneg\t\t%eax\n".to_string()),
                    UnaryOps::LOGNEG => {
                        gen.emit_ins("cmp ", "$0", "%eax")?;
                        gen.mov(0, "eax")?;
                        gen.emit("\tsete\t%al\n".to_string())
                    }
                }
            }
            Expression::BinaryExpression {
                first,
                second,
                operation,
            } => {
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
                    BinaryOps::SUB => gen.emit(
                        "    sub     %eax, %ecx
    mov     %ecx, %eax
"
                        .to_string(),
                    ),
                    BinaryOps::MUL => gen.emit_ins("imul", "%ecx", "%eax"),
                    BinaryOps::DIV => gen.emit(
                        "    mov     %eax, %ebx
    mov     %ecx, %eax
    cdq
    idiv    %ebx
"
                        .to_string(),
                    ),
                    BinaryOps::MOD => gen.emit(
                        "    mov     %eax, %ebx
    mov     %ecx, %eax
    cdq
    idiv    %ebx
    mov     %edx, %eax
"
                        .to_string(),
                    ),
                    BinaryOps::EQ => gen.emit_cmp("sete"),
                    BinaryOps::NE => gen.emit_cmp("setne"),
                    BinaryOps::LT => gen.emit_cmp("setl"),
                    BinaryOps::GT => gen.emit_cmp("setg"),
                    BinaryOps::LE => gen.emit_cmp("setle"),
                    BinaryOps::GE => gen.emit_cmp("setge"),
                    _ => Ok(0),
                }?;
                Ok(0)
            }
            Expression::NamedVariable {
                stack_offset,
                data_type: _,
            } => {
                gen.emit_ins("mov ", format!("-{}(%rbp)", stack_offset).as_str(), "%eax")?;
                Ok(0)
            }
            Expression::VariableAssign {
                stack_offset,
                expression,
            } => {
                expression.generate(gen)?;
                gen.emit(format!("\tmov \t%eax, -{}(%rbp)\n", stack_offset))?;
                Ok(0)
            }
            Expression::FunctionCall(function_call) => function_call.generate(gen),
        }
    }
}

impl Expression {
    fn generate_and_or(
        &self,
        operation: &BinaryOps,
        second: &Rc<Expression>,
        gen: &mut Generator,
    ) -> Result<usize, Error> {
        let (second_expression_label, end_label) = Generator::generate_clause_names();
        match operation {
            BinaryOps::AND => {
                gen.emit_ins("cmp ", "$0", "%eax")?;
                gen.emit(format!("\tjne\t\t{}\n", second_expression_label))?;
                gen.emit(format!("\tjmp\t\t{}\n", end_label))
            }
            BinaryOps::OR => {
                gen.emit_ins("cmp ", "$0", "%eax")?;
                gen.emit(format!("\tje\t\t{}\n", second_expression_label))?;
                gen.mov(1, "eax")?;
                gen.emit(format!("\tjmp\t\t{}\n", end_label))
            }
            _ => panic!("Wrong operation for boolean comparision!"),
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
                let value: i32 = lexer
                    .expect(Token::INTLITERAL)?
                    .trim_start()
                    .parse()
                    .expect("was not able to parse int literal");
                Ok(Rc::new(Self::Literal(value)))
            }
            Token::IDENT => {
                let name = lexer.expect(Token::IDENT)?.trim_start().to_string();

                if lexer.peek() == Token::LPAREN {
                    return Ok(Rc::new(Self::FunctionCall(FunctionCall::parse_name(name, lexer, scope)?)))
                }

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
            token => panic!("No literal for {:?}", token),
        }
    }

    fn parse_unary(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        let op = match lexer.next() {
            Token::SUB => UnaryOps::NEG,
            Token::LOGNEG => UnaryOps::LOGNEG,
            _ => panic!("Cannot parse binary operation!"),
        };
        let e = Self::parse_factor(lexer, scope)?;
        Ok(Rc::new(Self::Unary {
            expression: e,
            operation: op,
        }))
    }

    fn parse_factor(lexer: &mut Lexer, scope: &mut Scope) -> Result<Rc<Self>, LexerError> {
        match lexer.peek() {
            Token::SUB | Token::LOGNEG => Self::parse_unary(lexer, scope),
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
            if first_operand.data_type() != second_operand.data_type() {
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
            Expression::Literal(_) => DataType::INT,
            Expression::NamedVariable {
                data_type,
                stack_offset: _,
            } => data_type.clone(),
            Expression::VariableAssign {
                stack_offset: _,
                expression,
            } => expression.data_type(),
            Expression::Unary {
                expression,
                operation: _,
            } => expression.data_type(),
            Expression::BinaryExpression {
                first,
                second: _second,
                operation: _operation,
            } => first.data_type(),
            Expression::FunctionCall(_) => DataType::INT,
        }
    }
}
