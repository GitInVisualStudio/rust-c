use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{expression::Expression, statement_list::StatementList, ASTNode};

#[derive(Debug, Getters)]
pub struct WhileStatement {
    condition: Expression,
    body: StatementList,
    label_index: usize,
}

impl ASTNode for WhileStatement {}

impl Parse<WhileStatement> for Parser<'_> {
    fn parse(&mut self) -> Result<WhileStatement, LexerError> {
        self.expect(Token::WHILE)?;
        self.expect(Token::LPAREN)?;
        let condition = self.parse()?;
        self.expect(Token::RPAREN)?;
        let label_index = self.next_label_index();
        let body = self.parse()?;
        Ok(WhileStatement {
            condition,
            body,
            label_index,
        })
    }
}