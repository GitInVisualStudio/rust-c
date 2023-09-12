use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{expression::Expression, statement_list::StatementList, ASTNode};

#[derive(Debug)]
pub enum ElsePart {
    IfStatement(IfStatement),
    StatementList(StatementList),
    None
}

#[derive(Debug, Getters)]
pub struct IfStatement {
    statements: StatementList,
    condition: Expression,
    else_part: Box<ElsePart>,
}

impl ASTNode for IfStatement {}

impl Parse<IfStatement> for Parser<'_> {
    fn parse(&mut self) -> Result<IfStatement, LexerError> {
        self.expect(Token::IF)?;
        self.expect(Token::LPAREN)?;
        let condition: Expression = self.parse()?;
        self.expect(Token::RPAREN)?;
        let statements: StatementList = self.parse()?;
        let mut else_part: ElsePart = ElsePart::None;
        if self.peek() == Token::ELSE {
            self.next();
            else_part = match self.peek() {
                Token::IF => ElsePart::IfStatement(self.parse()?),
                _ => ElsePart::StatementList(self.parse()?)
            };
        }
        Ok(IfStatement {
            statements: statements,
            condition: condition,
            else_part: Box::new(else_part),
        })
    }
}
