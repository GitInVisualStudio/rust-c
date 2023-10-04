use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{
    array_expression::ArrayExpression, assignment::Assignment, function_call::FunctionCall,
    struct_expression::StructExpression, type_expression::TypeExpression,
};

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy)]
pub enum UnaryOps<'a> {
    NEG,
    LOGNEG,
    REF,
    DEREF,
    COMPLEMENT,
    Cast(&'a TypeExpression<'a>),
}

#[derive(Debug)]
pub enum Expression<'a> {
    IntLiteral(i32),
    CharLiteral(u8),
    FunctionCall(&'a FunctionCall<'a>),
    ArrayExpression(&'a ArrayExpression<'a>),
    StructExpresion(&'a StructExpression<'a>),
    Assignment(&'a Assignment<'a>),
    TypeExpression(&'a TypeExpression<'a>),
    SizeOf(&'a Expression<'a>),
    FieldAccess {
        name: &'a str,
        operand: &'a Expression<'a>,
    },
    ArrowAccess {
        name: &'a str,
        operand: &'a Expression<'a>,
    },
    Indexing {
        index: &'a Expression<'a>,
        operand: &'a Expression<'a>,
    },
    NamedVariable {
        name: &'a str,
    },
    Unary {
        expression: &'a Expression<'a>,
        operation: UnaryOps<'a>,
    },
    BinaryExpression {
        lhs: &'a Expression<'a>,
        rhs: &'a Expression<'a>,
        operation: BinaryOps,
    },
}

impl<'a> Visitable for Expression<'a> {}

impl<'a> Parser<'a> {
    pub fn expression(&mut self) -> Result<&'a Expression<'a>, Error<'a>> {
        Ok(self.bump.alloc(Expression::parse_expressions(self)?))
    }
}

impl<'a> Expression<'a> {
    fn parse_literal(parser: &mut Parser<'a>) -> Result<Self, Error<'a>> {
        match parser.peek() {
            TokenKind::LCURL => {
                parser.next();
                match parser.peek() {
                    TokenKind::DOT => Ok(Self::StructExpresion(parser.struct_expression()?)),
                    _ => Ok(Self::ArrayExpression(parser.array_expression()?)),
                }
            }
            TokenKind::STRINGLIT => Ok(Self::ArrayExpression(parser.array_expression()?)),
            TokenKind::INTLITERAL => {
                let value: i32 = parser
                    .expect(TokenKind::INTLITERAL)?
                    .string()
                    .trim()
                    .parse()
                    .expect("was not able to parse int literal");
                Ok(Self::IntLiteral(value))
            }
            TokenKind::CHARLITERAL => {
                let string = parser.expect(TokenKind::CHARLITERAL)?.string();
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
            TokenKind::IDENT => {
                // if parser.scope.contains::<Rc<Struct>>(&name) {
                //     Ok(Self::TypeExpression(parser.parse()?))
                // } else if parser.scope.contains::<TypeDefinition>(&name) {
                //     Ok(Self::TypeExpression(parser.parse()?))
                // } else {
                //     let name = parser.expect(TokenKind::IDENT)?.to_string();
                let name = parser.expect(TokenKind::IDENT)?.string();
                match parser.peek() {
                    TokenKind::LPAREN => Ok(Self::FunctionCall(parser.function_call()?)),
                    _ => Ok(Self::NamedVariable { name: name }),
                }
            }
            TokenKind::LPAREN => {
                parser.expect(TokenKind::LPAREN)?;
                let result = Self::parse_expressions(parser);
                parser.expect(TokenKind::RPAREN)?;
                result
            }
            TokenKind::SIZEOF => {
                parser.next();
                parser.expect(TokenKind::LPAREN)?;
                let expression = Self::parse_expressions(parser)?;
                parser.expect(TokenKind::RPAREN)?;
                Ok(Self::SizeOf(parser.bump.alloc(expression)))
            }
            _ => Ok(Self::TypeExpression(
                parser.bump.alloc(parser.type_expression()?),
            )),
        }
    }

    fn parse_indexing(
        mut operand: Expression<'a>,
        parser: &mut Parser<'a>,
    ) -> Result<Self, Error<'a>> {
        while parser.peek() == TokenKind::LBRACE {
            parser.next();
            let expression = parser.expression()?;
            parser.expect(TokenKind::RBRACE)?;
            operand = Self::Indexing {
                index: parser.bump.alloc(expression),
                operand: parser.bump.alloc(operand),
            };
        }
        Ok(operand)
    }

    fn parse_field_access(
        mut operand: Expression<'a>,
        parser: &mut Parser<'a>,
    ) -> Result<Self, Error<'a>> {
        while parser.peek() == TokenKind::DOT {
            parser.next();
            let name = parser.expect(TokenKind::IDENT)?.string();
            operand = Expression::FieldAccess {
                name: name,
                operand: parser.bump.alloc(operand),
            }
        }
        Ok(operand)
    }

    fn parse_arrow_access(
        mut operand: Expression<'a>,
        parser: &mut Parser<'a>,
    ) -> Result<Self, Error<'a>> {
        while parser.peek() == TokenKind::ARROW {
            parser.next();
            let name = parser.expect(TokenKind::IDENT)?.string();
            operand = Expression::ArrowAccess {
                name: name,
                operand: parser.bump.alloc(operand),
            }
        }
        Ok(operand)
    }

    fn parse_postfix(parser: &mut Parser<'a>) -> Result<Self, Error<'a>> {
        let mut result = Self::parse_literal(parser)?;
        while parser.peek() == TokenKind::LBRACE
            || parser.peek() == TokenKind::DOT
            || parser.peek() == TokenKind::ARROW
        {
            result = match parser.peek() {
                TokenKind::LBRACE => Self::parse_indexing(result, parser),
                TokenKind::DOT => Self::parse_field_access(result, parser),
                TokenKind::ARROW => Self::parse_arrow_access(result, parser),
                _ => Ok(result),
            }?;
        }
        Ok(result)
    }

    fn parse_unary(parser: &mut Parser<'a>) -> Result<Self, Error<'a>> {
        let (token, location) = parser.next();
        let e = Self::parse_factor(parser)?;
        let op = match token {
            TokenKind::SUB => UnaryOps::NEG,
            TokenKind::LOGNEG => UnaryOps::LOGNEG,
            TokenKind::REF => UnaryOps::REF,
            TokenKind::MUL => UnaryOps::DEREF,
            TokenKind::COMPLEMENT => UnaryOps::COMPLEMENT,
            _ => panic!(
                "Cannot parse binary operation! This should never happen! {:?}",
                location
            ),
        };
        Ok(Self::Unary {
            expression: parser.bump.alloc(e),
            operation: op,
        })
    }

    fn parse_factor(parser: &mut Parser<'a>) -> Result<Self, Error<'a>> {
        let can_be_cast = parser.peek() == TokenKind::LPAREN;

        let result = match parser.peek() {
            TokenKind::SUB
            | TokenKind::LOGNEG
            | TokenKind::MUL
            | TokenKind::REF
            | TokenKind::COMPLEMENT => Self::parse_unary(parser),
            // only literal left to parse
            _ => Self::parse_postfix(parser),
        };

        if can_be_cast {
            return match result? {
                Expression::TypeExpression(t) => {
                    let factor = Self::parse_factor(parser)?;
                    Ok(Self::Unary {
                        expression: parser.bump.alloc(factor),
                        operation: UnaryOps::Cast(parser.bump.alloc(t)),
                    })
                }
                x => Ok(x),
            };
        }

        result
    }

    fn parse_binary(
        parser: &mut Parser<'a>,
        operations: &[Vec<TokenKind>],
        index: usize,
    ) -> Result<Self, Error<'a>> {
        let op = operations.get(index);
        // if we are at the end of the binary operations we parse a factor
        if op.is_none() {
            return Self::parse_factor(parser);
        }
        let op = op.unwrap();
        let mut expression = Self::parse_binary(parser, operations, index + 1)?;

        while let Some(operand) = op.iter().find(|x| parser.peek() == **x) {
            parser.next();
            let lhs = expression;
            let rhs = Self::parse_binary(parser, operations, index + 1)?;

            let lhs = parser.bump.alloc(lhs);
            let rhs = parser.bump.alloc(rhs);

            expression = match *operand {
                TokenKind::ADD => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::ADD,
                },
                TokenKind::SUB => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::SUB,
                },
                TokenKind::MUL => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::MUL,
                },
                TokenKind::DIV => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::DIV,
                },
                TokenKind::MOD => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::MOD,
                },
                TokenKind::AND => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::AND,
                },
                TokenKind::OR => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::OR,
                },
                TokenKind::EQ => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::EQ,
                },
                TokenKind::NE => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::NE,
                },
                TokenKind::LT => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::LT,
                },
                TokenKind::GT => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::GT,
                },
                TokenKind::LE => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::LE,
                },
                TokenKind::GE => Self::BinaryExpression {
                    lhs,
                    rhs,
                    operation: BinaryOps::GE,
                },
                _ => panic!("should never happen!"),
            }
        }
        Ok(expression)
    }

    pub fn parse_expressions(parser: &mut Parser<'a>) -> Result<Self, Error<'a>> {
        let operations = [
            vec![TokenKind::OR],
            vec![TokenKind::AND],
            vec![TokenKind::EQ, TokenKind::NE],
            vec![TokenKind::GT, TokenKind::GE, TokenKind::LT, TokenKind::LE],
            vec![TokenKind::ADD, TokenKind::SUB],
            vec![TokenKind::MUL, TokenKind::DIV, TokenKind::MOD],
        ];
        let result = Self::parse_binary(parser, &operations, 0)?;
        match parser.peek() {
            TokenKind::ASSIGN => {
                parser.assignee = Some(parser.bump.alloc(result));
                let assignment = parser.assignment()?;
                let result = Ok(Self::Assignment(parser.bump.alloc(assignment)));
                parser.assignee = None;
                result
            }
            _ => Ok(result),
        }
    }
}
