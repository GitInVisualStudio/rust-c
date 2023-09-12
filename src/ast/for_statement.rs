use derive_getters::Getters;

use crate::{
    lexer::{tokens::Token, LexerError},
    parser::{Parse, Parser},
};

use super::{expression::Expression, statement::Statement, statement_list::StatementList, ASTNode};

#[derive(Debug, Getters)]
pub struct ForStatement {
    init: Box<Statement>,
    condition: Expression,
    post: Option<Expression>,
    body: StatementList,
    label_index: usize,
}

impl ASTNode for ForStatement {}

impl Parse<ForStatement> for Parser<'_> {
    fn parse(&mut self) -> Result<ForStatement, LexerError> {
        self.scope.push();
        self.expect(Token::FOR)?;
        self.expect(Token::LPAREN)?;
        let label_index = self.next_label_index();
        let init: Statement = self.parse()?;

        let condition;
        if self.peek() != Token::SEMIC {
            condition = self.parse()?;
        } else {
            condition = Expression::IntLiteral(1);
        }
        self.expect(Token::SEMIC)?;

        let post: Option<Expression>;
        if self.peek() != Token::RPAREN {
            post = Some(self.parse()?);
        } else {
            post = None;
        }
        self.expect(Token::RPAREN)?;

        let body: StatementList = self.parse()?;
        self.scope.pop();

        Ok(ForStatement {
            init: Box::new(init),
            condition: condition,
            post: post,
            body: body,
            label_index: label_index,
        })
    }
}
