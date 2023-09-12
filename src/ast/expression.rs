use std::rc::Rc;

use super::array_expression::ArrayExpression;
use super::assignment::Assignment;
use super::data_type::{DataType, Struct};
use super::function_call::FunctionCall;
use super::struct_expression::StructExpression;
use super::type_definition::TypeDefinition;
use super::type_expression::TypeExpression;
use super::variable::Variable;
use super::ASTNode;
use crate::lexer::tokens::Token;
use crate::lexer::LexerError;
use crate::parser::scope::IScope;
use crate::parser::{Parse, Parser};

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
    FunctionCall(FunctionCall),
    ArrayExpression(ArrayExpression),
    StructExpresion(StructExpression),
    Assignment(Box<Assignment>),
    TypeExpression(TypeExpression),
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
        first: Box<Expression>,
        second: Box<Expression>,
        operation: BinaryOps,
    },
}

impl ASTNode for Expression {}

impl Parse<Expression> for Parser<'_> {
    fn parse(&mut self) -> Result<Expression, LexerError> {
        Expression::parse_expressions(self)
    }
}

impl Expression {

    fn parse_literal(parser: &mut Parser) -> Result<Self, LexerError> {
        match parser.peek() {
            Token::LCURL => {
                parser.next();
                match parser.peek() {
                    Token::DOT => Ok(Self::StructExpresion(parser.parse()?)),
                    _ => Ok(Self::ArrayExpression(parser.parse()?)),
                }
            }
            Token::STRINGLIT => Ok(Self::ArrayExpression(parser.parse()?)),
            Token::INTLITERAL => {
                let value: i32 = parser
                    .expect(Token::INTLITERAL)?
                    .trim_start()
                    .parse()
                    .expect("was not able to parse int literal");
                Ok(Self::IntLiteral(value))
            }
            Token::CHARLITERAL => {
                let string = parser.expect(Token::CHARLITERAL)?;
                if string.len() > 3 {
                    let string = &string[1..3];
                    return match string {
                        "\\n" => Ok(Self::CharLiteral(b'\n')),
                        "\\t" => Ok(Self::CharLiteral(b'\t')),
                        _ => {
                            println!("{}", string);
                            let value: u8 = string[1..2].as_bytes().first().unwrap().clone();
                            Ok(Self::CharLiteral(value))
                        }
                    };
                }
                let value: u8 = string[1..2].as_bytes().first().unwrap().clone();
                Ok(Self::CharLiteral(value))
            }
            Token::IDENT => {
                let name = parser.peek_str().to_owned();
                if parser.scope.contains::<Rc<Struct>>(&name) {
                    Ok(Self::TypeExpression(parser.parse()?))
                } else if parser.scope.contains::<TypeDefinition>(&name) {
                    Ok(Self::TypeExpression(parser.parse()?))
                } else {
                    let name = parser.expect(Token::IDENT)?.to_string();

                    match parser.peek() {
                        Token::LPAREN => Ok(Self::FunctionCall(parser.parse()?)),
                        _ => {
                            let contains: Option<&Variable> = parser.scope.get(&name);
                            if let None = contains {
                                return parser.error(format!("Variable {} not found!", name));
                            }
                            let var = contains.unwrap();
                            let offset = var.offset();
                            Ok(Self::NamedVariable {
                                stack_offset: offset,
                                data_type: var.data_type(),
                            })
                        }
                    }
                }
            }
            Token::LPAREN => {
                parser.expect(Token::LPAREN)?;
                let result = Self::parse_expressions(parser);
                parser.expect(Token::RPAREN)?;
                result
            }
            Token::SIZEOF => {
                parser.next();
                parser.expect(Token::LPAREN)?;
                let expression = Self::parse_expressions(parser)?;
                parser.expect(Token::RPAREN)?;
                Ok(Self::IntLiteral(expression.data_type().size() as i32))
            }
            _ => Ok(Self::TypeExpression(parser.parse()?)),
        }
    }

    fn parse_indexing(mut operand: Expression, parser: &mut Parser) -> Result<Self, LexerError> {
        while parser.peek() == Token::LBRACE {
            parser.next();
            let expression: Expression = parser.parse()?;
            parser.expect(Token::RBRACE)?;
            operand = Self::Indexing {
                index: Rc::new(expression),
                operand: Rc::new(operand),
            };
        }
        Ok(operand)
    }

    fn parse_field_access(
        mut operand: Expression,
        parser: &mut Parser,
    ) -> Result<Self, LexerError> {
        while parser.peek() == Token::DOT {
            parser.next();
            let data_type = operand.data_type();
            match data_type {
                DataType::STRUCT(data_type) => {
                    let name = parser.expect(Token::IDENT)?.to_string();
                    match data_type.get(&name) {
                        Some(var) => {
                            operand = Self::FieldAccess {
                                offset: var.offset() - var.data_type().size(),
                                data_type: var.data_type(),
                                operand: Rc::new(operand),
                            }
                        }
                        None => parser.error(format!("No field {} for {:?}", name, data_type))?,
                    };
                }
                x => parser.error(format!("Cannot access field for datatype: {:?}", x))?,
            };
        }
        Ok(operand)
    }

    fn parse_arrow_access(
        mut operand: Expression,
        parser: &mut Parser,
    ) -> Result<Self, LexerError> {
        while parser.peek() == Token::ARROW {
            parser.next();
            let data_type = operand.data_type();
            match data_type {
                DataType::PTR(data_type) => {
                    operand = Self::Unary {
                        expression: Rc::new(operand),
                        operation: UnaryOps::DEREF,
                    };
                    match data_type.as_ref() {
                        DataType::STRUCT(data_type) => {
                            let name = parser.expect(Token::IDENT)?.to_string();
                            match data_type.get(&name) {
                                Some(var) => {
                                    operand = Self::FieldAccess {
                                        offset: var.offset() - var.data_type().size(),
                                        data_type: var.data_type(),
                                        operand: Rc::new(operand),
                                    }
                                }
                                None => parser
                                    .error(format!("No field {} for {:?}", name, data_type))?,
                            };
                        }
                        x => parser.error(format!("Cannot access field for datatype: {:?}", x))?,
                    }
                }
                _ => parser.error("cannot deref non pointer expression!".to_string())?,
            };
        }
        Ok(operand)
    }

    fn parse_postfix(parser: &mut Parser) -> Result<Self, LexerError> {
        let mut result = Self::parse_literal(parser)?;
        while parser.peek() == Token::LBRACE
            || parser.peek() == Token::DOT
            || parser.peek() == Token::ARROW
        {
            result = match parser.peek() {
                Token::LBRACE => Self::parse_indexing(result, parser),
                Token::DOT => Self::parse_field_access(result, parser),
                Token::ARROW => Self::parse_arrow_access(result, parser),
                _ => Ok(result),
            }?;
        }
        Ok(result)
    }

    fn parse_unary(parser: &mut Parser) -> Result<Self, LexerError> {
        let token = parser.next();
        let e = Self::parse_factor(parser)?;
        let op = match token {
            Token::SUB => UnaryOps::NEG,
            Token::LOGNEG => UnaryOps::LOGNEG,
            Token::REF => {
                if let Expression::NamedVariable {
                    stack_offset: _,
                    data_type: _,
                } = &e
                {
                    UnaryOps::REF
                } else {
                    parser.error("Cannot get address from non-variable expresion!".to_string())?
                }
            }
            Token::MUL => {
                if let DataType::PTR(_) = e.data_type() {
                    UnaryOps::DEREF
                } else {
                    parser.error("Cannot de-refrence non-pointer expresion!".to_string())?
                }
            }
            Token::COMPLEMENT => UnaryOps::COMPLEMENT,
            _ => panic!("Cannot parse binary operation!"),
        };
        Ok(Self::Unary {
            expression: Rc::new(e),
            operation: op,
        })
    }

    fn parse_factor(parser: &mut Parser) -> Result<Self, LexerError> {
        let can_be_cast = parser.peek() == Token::LPAREN;

        let result = match parser.peek() {
            Token::SUB | Token::LOGNEG | Token::MUL | Token::REF | Token::COMPLEMENT => {
                Self::parse_unary(parser)
            }
            // only literal left to parse
            _ => Self::parse_postfix(parser),
        };

        if can_be_cast {
            return match result? {
                Expression::TypeExpression(t) => {
                    let factor = Self::parse_factor(parser)?;
                    Ok(Self::Unary {
                        expression: Rc::new(factor),
                        operation: UnaryOps::CAST(t.data_type()),
                    })
                }
                x => Ok(x),
            };
        }

        result
    }

    fn parse_binary(
        parser: &mut Parser,
        operations: &[Vec<Token>],
        index: usize,
    ) -> Result<Self, LexerError> {
        let op = operations.get(index);
        // if we are at the end of the binary operations we parse a factor
        if op.is_none() {
            return Self::parse_factor(parser);
        }
        let op = op.unwrap();
        let mut expression = Self::parse_binary(parser, operations, index + 1)?;

        while let Some(operand) = op.iter().find(|x| parser.peek() == **x) {
            parser.next();
            let first_operand = expression;
            let second_operand = Self::parse_binary(parser, operations, index + 1)?;
            if first_operand.data_type() != second_operand.data_type()
                && !first_operand
                    .data_type()
                    .can_operate(second_operand.data_type())
            {
                return parser.error(
                    "cannot perform binary operation on 2 different data types!".to_string(),
                );
            }

            let first_operand = Box::new(first_operand);
            let second_operand = Box::new(second_operand);

            expression = match *operand {
                Token::ADD => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::ADD,
                },
                Token::SUB => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::SUB,
                },
                Token::MUL => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::MUL,
                },
                Token::DIV => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::DIV,
                },
                Token::MOD => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::MOD,
                },
                Token::AND => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::AND,
                },
                Token::OR => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::OR,
                },
                Token::EQ => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::EQ,
                },
                Token::NE => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::NE,
                },
                Token::LT => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::LT,
                },
                Token::GT => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::GT,
                },
                Token::LE => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::LE,
                },
                Token::GE => Self::BinaryExpression {
                    first: first_operand,
                    second: second_operand,
                    operation: BinaryOps::GE,
                },
                // this should not happen!
                _ => panic!("Unknown operation"),
            }
        }
        Ok(expression)
    }

    fn parse_expressions(parser: &mut Parser) -> Result<Self, LexerError> {
        let operations = [
            vec![Token::OR],
            vec![Token::AND],
            vec![Token::EQ, Token::NE],
            vec![Token::GT, Token::GE, Token::LT, Token::LE],
            vec![Token::ADD, Token::SUB],
            vec![Token::MUL, Token::DIV, Token::MOD],
        ];
        let result = Self::parse_binary(parser, &operations, 0)?;
        match parser.peek() {
            Token::ASSIGN => {
                parser.assignee = Some(Rc::new(result));
                let result = Ok(Self::Assignment(Box::new(parser.parse()?)));
                parser.assignee = None;
                result
            }
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
            Expression::FunctionCall(call) => call.return_type().clone(),
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
            Expression::StructExpresion(x) => x.data_type().clone(),
        }
    }
}
