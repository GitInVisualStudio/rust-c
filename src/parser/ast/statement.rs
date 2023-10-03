use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser, visitor::Visitable};

use super::{
    compound_statement::Compound, expression::Expression, for_statement::ForStatement,
    if_statement::IfStatement, type_definition::TypeDefinition, while_statement::WhileStatement,
    TypeExpression,
};

#[derive(Debug)]
pub enum Statement<'a> {
    Return(Option<Expression<'a>>),
    SingleExpression(Expression<'a>),
    Compound(Compound<'a>),
    IfStatement(IfStatement<'a>),
    ForStatement(ForStatement<'a>),
    WhileStatement(WhileStatement<'a>),
    TypeDefinition(TypeDefinition<'a>),
    VariableDeclaration {
        name: &'a str,
        expression: TypeExpression<'a>,
        assignment: Option<Expression<'a>>,
    },
    Conitnue,
    Break,
    Empty,
}

impl Visitable for Statement<'_> {}

impl<'a> Parser<'a> {
    pub fn statement(&mut self) -> Result<Statement<'a>, Error<'a>> {
        let result = match self.peek() {
            TokenKind::CONTINUE => {
                self.next();
                Ok(Statement::Conitnue)
            }
            TokenKind::BREAK => {
                self.next();
                Ok(Statement::Break)
            }
            TokenKind::RETURN => {
                self.expect(TokenKind::RETURN)?;
                if self.peek() == TokenKind::SEMIC {
                    self.expect(TokenKind::SEMIC)?;
                    return Ok(Statement::Return(None));
                }
                let expression = self.expression()?;
                Ok(Statement::Return(Some(expression)))
            }
            TokenKind::IF => {
                let statement = self.if_statement()?;
                return Ok(Statement::IfStatement(statement));
            }
            TokenKind::FOR => {
                let statement = self.for_statement()?;
                return Ok(Statement::ForStatement(statement));
            }
            TokenKind::WHILE => {
                let statement = self.while_statement()?;
                return Ok(Statement::WhileStatement(statement));
            }
            TokenKind::INT
            | TokenKind::CHAR
            | TokenKind::LONG
            | TokenKind::VOID
            | TokenKind::STRUCT
            | TokenKind::TYPEOF
            | TokenKind::IDENT => {
                let anchor = self.anchor();

                let expression = self.type_expression()?;
                let name = self.expect(TokenKind::IDENT);
                match name {
                    Ok(name) => {
                        let mut assignment = None;
                        if self.peek() == TokenKind::ASSIGN {
                            self.next();
                            assignment = Some(self.expression()?);
                        }
                        Ok(Statement::VariableDeclaration {
                            name: name.string(),
                            expression,
                            assignment,
                        })
                    }
                    Err(_) => {
                        self.reset(anchor);
                        Ok(Statement::SingleExpression(self.expression()?))
                    }
                }
            }
            TokenKind::TYPEDEF => {
                let def = self.type_def()?;
                Ok(Statement::TypeDefinition(def))
            }
            TokenKind::SEMIC => Ok(Statement::Empty),
            TokenKind::LCURL => {
                let list = self.compound_statement()?;
                return Ok(Statement::Compound(list));
            }
            _ => {
                let expr = self.expression()?;
                Ok(Statement::SingleExpression(expr))
            }
        }?;
        self.expect(TokenKind::SEMIC)?;
        Ok(result)
    }
}
