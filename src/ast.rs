use std::fmt::Debug;

pub mod array_expression;
pub mod assignment;
pub mod data_type;
pub mod expression;
pub mod for_statement;
pub mod function;
pub mod function_call;
pub mod if_statement;
pub mod program;
pub mod statement;
pub mod statement_list;
pub mod struct_expression;
pub mod type_definition;
pub mod type_expression;
pub mod variable;
pub mod while_statement;

pub trait Visitor<T, R> {
    fn visit(&mut self, visitor: T) -> R;
}

pub trait ASTNode: Debug {
    fn accept<'a, R>(&'a self, visitor: &mut dyn Visitor<&'a Self, R>) -> R where Self: Sized {
        visitor.visit(&self)
    }
}