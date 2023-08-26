use std::rc::Rc;

use crate::lexer::tokens::Token;

use super::{
    expression::Expression, generator::Generator, statement::Statement,
    statement_list::StatementList, ASTNode,
};

#[derive(Debug)]
pub struct ForStatement {
    init: Rc<Statement>,
    condition: Rc<Expression>,
    post: Rc<Expression>,
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

        let init = Statement::parse(lexer, scope)?;
        let condition = Expression::parse(lexer, scope)?;
        lexer.expect(Token::SEMIC)?;

        let post = Expression::parse(lexer, scope)?;
        lexer.expect(Token::RPAREN)?;

        let body = StatementList::parse(lexer, scope)?;
        scope.pop();

        Ok(Rc::new(ForStatement {
            init: init,
            condition: condition,
            post: post,
            body: body,
            label_index: Generator::next_label_index(),
        }))
    }

    fn generate(&self, gen: &mut super::generator::Generator) -> Result<usize, std::io::Error> {
        let (body, end, post) = Generator::generate_label_names(self.label_index);
        self.init.generate(gen)?;

        gen.emit_label(&body)?;

        self.condition.generate(gen)?;
        gen.emit_ins("cmp ", "$0", "%eax")?;
        // jump to end of for if the condition is not met anymore
        gen.emit(format!("\tje\t\t{}\n", end))?;

        self.body.generate(gen)?;

        gen.emit_label(&post)?;
        self.post.generate(gen)?;

        gen.emit(format!("\tjmp\t\t{}\n", body))?;
        gen.emit_label(&end)?;
        Ok(0)
    }
}
