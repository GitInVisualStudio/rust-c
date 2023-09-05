use std::rc::Rc;

use crate::{lexer::tokens::Token, parser::generator::register::Reg};

use super::{
    expression::Expression, generator::Generator, statement::Statement,
    statement_list::StatementList, ASTNode,
};

#[derive(Debug)]
pub struct ForStatement {
    init: Rc<Statement>,
    condition: Rc<Expression>,
    post: Option<Rc<Expression>>,
    body: Rc<StatementList>,
    label_index: usize,
}

impl ASTNode for ForStatement {
    fn parse(
        lexer: &mut crate::lexer::Lexer,
        scope: &mut super::scope::Scope,
    ) -> Result<Rc<Self>, crate::lexer::LexerError>
    where
        Self: Sized,
    {
        scope.push();

        lexer.expect_tokens(&[Token::FOR, Token::LPAREN])?;
        let label_index = Generator::next_label_index();
        let init = Statement::parse(lexer, scope)?;

        let condition;
        if lexer.peek() != Token::SEMIC {
            condition = Expression::parse(lexer, scope)?;
        } else {
            condition = Rc::new(Expression::IntLiteral(1));
        }
        lexer.expect(Token::SEMIC)?;

        let post;
        if lexer.peek() != Token::RPAREN {
            post = Some(Expression::parse(lexer, scope)?);
        } else {
            post = None;
        }
        lexer.expect(Token::RPAREN)?;

        let body = StatementList::parse(lexer, scope)?;
        scope.pop();

        Ok(Rc::new(ForStatement {
            init: init,
            condition: condition,
            post: post,
            body: body,
            label_index: label_index,
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        let (body, end, post) = Generator::generate_label_names(self.label_index);
        self.init.generate(gen)?;

        gen.emit_label(&body)?;

        self.condition.generate(gen)?;
        gen.cmp(Reg::IMMEDIATE(0), Reg::current())?;
        // jump to end of for if the condition is not met anymore
        gen.je(&end)?;

        self.body.generate(gen)?;

        gen.emit_label(&post)?;
        if let Some(post) = &self.post {
            post.generate(gen)?;
        }

        gen.jmp(&body)?;
        gen.emit_label(&end)
    }
}
