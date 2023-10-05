use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{
    compound_statement::Compound, expression::Expression, for_statement::ForStatement,
    if_statement::IfStatement, type_definition::TypeDefinition, while_statement::WhileStatement,
    Assignment, TypeExpression,
};

#[derive(Debug)]
pub enum Statement<'a> {
    Return(Option<&'a Expression<'a>>),
    SingleExpression(&'a Expression<'a>),
    Compound(&'a Compound<'a>),
    IfStatement(&'a IfStatement<'a>),
    ForStatement(&'a ForStatement<'a>),
    WhileStatement(&'a WhileStatement<'a>),
    TypeDefinition(&'a TypeDefinition<'a>),
    VariableDeclaration {
        name: &'a str,
        expression: &'a TypeExpression<'a>,
        assignment: Option<&'a Assignment<'a>>,
    },
    Conitnue,
    Break,
    Empty,
}

impl Visitable for Statement<'_> {}

impl<'a> Parser<'a> {
    pub fn statement(&mut self) -> Result<&'a Statement<'a>, Error<'a>> {
        let result = match self.peek() {
            TokenKind::CONTINUE => {
                self.next();
                Statement::Conitnue
            }
            TokenKind::BREAK => {
                self.next();
                Statement::Break
            }
            TokenKind::RETURN => {
                self.expect(TokenKind::RETURN)?;
                if self.peek() == TokenKind::SEMIC {
                    self.expect(TokenKind::SEMIC)?;
                    return Ok(self.alloc(Statement::Return(None)));
                }
                let expression = self.expression()?;
                Statement::Return(Some(expression))
            }
            TokenKind::IF => {
                let statement = self.if_statement()?;
                return Ok(self.alloc(Statement::IfStatement(statement)));
            }
            TokenKind::FOR => {
                let statement = self.for_statement()?;
                return Ok(self.alloc(Statement::ForStatement(statement)));
            }
            TokenKind::WHILE => {
                let statement = self.while_statement()?;
                return Ok(self.alloc(Statement::WhileStatement(statement)));
            }
            TokenKind::INT
            | TokenKind::CHAR
            | TokenKind::LONG
            | TokenKind::VOID
            | TokenKind::STRUCT
            | TokenKind::TYPEOF
            | TokenKind::IDENT => {
                let anchor = self.anchor();

                let mut expression = self.type_expression()?;
                let name = self.expect(TokenKind::IDENT);

                if self.peek() == TokenKind::LBRACE {
                    self.next();
                    expression = self.alloc(TypeExpression::Pointer(expression));
                    self.expect(TokenKind::RBRACE)?;
                }

                match name {
                    Ok(name) => {
                        let mut assignment = None;
                        if self.peek() == TokenKind::ASSIGN {
                            self.next();
                            let result = Assignment::VariableAssignment {
                                name: name.string(),
                                expression: self.expression()?,
                            };
                            assignment = Some(&*self.alloc(result));
                        }
                        Statement::VariableDeclaration {
                            name: name.string(),
                            expression,
                            assignment,
                        }
                    }
                    Err(_) => {
                        self.reset(anchor);
                        Statement::SingleExpression(self.expression()?)
                    }
                }
            }
            TokenKind::TYPEDEF => {
                let def = self.type_def()?;
                Statement::TypeDefinition(def)
            }
            TokenKind::SEMIC => Statement::Empty,
            TokenKind::LCURL => {
                let list = self.compound_statement()?;
                return Ok(self.alloc(Statement::Compound(list)));
            }
            _ => {
                let expr = self.expression()?;
                Statement::SingleExpression(expr)
            }
        };
        self.expect(TokenKind::SEMIC)?;
        Ok(self.alloc(result))
    }
}
