use std::rc::Rc;

use derive_getters::Getters;

use super::ASTNode;
use super::data_type::DataType;
use super::statement_list::StatementList;
use super::type_expression::TypeExpression;
use super::variable::Variable;
use crate::lexer::tokens::Token;
use crate::lexer::LexerError;
use crate::parser::scope::IScope;
use crate::parser::{Parse, Parser};

#[derive(Debug, Getters)]
pub struct Function {
    statements: Option<StatementList>,
    parameter: Vec<Variable>,
    name: String,
    return_type: DataType,
    stack_size: usize,
}

impl ASTNode for Function {}
impl ASTNode for Rc<Function> {}

impl Parse<Rc<Function>> for Parser<'_> {
    fn parse(&mut self) -> Result<Rc<Function>, LexerError> {
        self.scope.push();
        let type_expression: TypeExpression = self.parse()?;
        let name = self.expect(Token::IDENT)?.to_string();

        self.expect(Token::LPAREN)?;
        let mut parameter: Vec<Variable> = Vec::new();
        while self.peek() != Token::RPAREN {
            parameter.push(Function::parse_parameter(self)?);
        }
        self.next();

        if self.peek() == Token::SEMIC {
            self.next();
            let result = Rc::new(Function {
                stack_size: 0,
                statements: None,
                name: name,
                parameter: parameter,
                return_type: type_expression.data_type(),
            });
            result.valid(self)?;
            self.scope.pop();
            self.scope.add(result.clone());
            return Ok(result);
        }

        let statements: StatementList = self.parse()?;

        let result = Rc::new(Function {
            stack_size: *statements.stack_size(),
            statements: Some(statements),
            name: name,
            parameter: parameter,
            return_type: type_expression.data_type(),
        });

        result.valid(self)?;

        self.scope.pop();
        self.scope.add(result.clone());
        Ok(result)
    }
}

impl Function {
    fn valid(&self, parser: &mut Parser) -> Result<bool, LexerError> {
        // check if function already exists
        let contains: Option<&Rc<Function>> = parser.scope.get(&self.name);
        if let Some(x) = contains {
            if x.return_type != self.return_type {
                return parser.error(format!(
                    "Declaration is incompatible with other declaration!"
                ));
            }
            if x.statements.is_some() && self.statements.is_some() {
                return parser.error(format!("Function {} already exists!", &self.name));
            }
            for (other, own) in self.parameter.iter().zip(&x.parameter) {
                if other.data_type() != own.data_type() || x.parameter.len() != self.parameter.len()
                {
                    return parser.error(format!(
                        "Declaration is incompatible with other declaration!"
                    ));
                }
            }
        }

        Ok(true)
    }

    fn parse_parameter(parser: &mut Parser) -> Result<Variable, LexerError> {
        let type_expression: TypeExpression = parser.parse()?;
        let name = parser.expect(Token::IDENT)?.to_string();
        let var = Variable::new(
            &name,
            type_expression.data_type(),
            parser.scope.stack_size(),
        );

        if parser.scope.contains::<Variable>(&name) {
            return parser.error(format!(
                "Parameter with name {} already declared in scope!",
                name
            ));
        }
        parser.scope.add(var.clone());

        if parser.peek() == Token::COMMA {
            parser.next();
        }

        Ok(var)
    }
}
