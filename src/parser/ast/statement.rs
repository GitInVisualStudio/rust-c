use crate::{error::Error, lexer::tokens::TokenKind, parser::Parser};

use super::{
    compound_statement::Compound, expression::Expression, for_statement::ForStatement,
    if_statement::IfStatement, type_definition::TypeDefinition, type_expression::TypeExpression,
    while_statement::WhileStatement, ASTNode,
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
        expression: Expression<'a>,
        assignment: Option<Expression<'a>>,
    },
    Conitnue,
    Break,
    Empty,
}

impl ASTNode for Statement<'_> {}

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
                if self.ahead(1) == TokenKind::ASSIGN {
                    Ok(Statement::SingleExpression(self.expression()?))
                } else {
                    let expression = self.expression()?;
                    if self.peek() == TokenKind::SEMIC {
                        Ok(Statement::SingleExpression(expression))
                    } else {
                        let name = self.expect(TokenKind::IDENT)?.string();
                        let mut assignment = None;
                        if self.peek() == TokenKind::ASSIGN {
                            self.next();
                            assignment = Some(self.expression()?);
                        }
                        Ok(Statement::VariableDeclaration {
                            name,
                            expression,
                            assignment,
                        })
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
