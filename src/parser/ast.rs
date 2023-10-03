use std::fmt::Debug;

pub mod program;
pub mod type_definition;
pub mod type_expression;
pub mod array_expression;
pub mod struct_expression;
pub mod expression;
pub mod function_call;
pub mod assignment;
pub mod statement;
pub mod compound_statement;
pub mod if_statement;
pub mod for_statement;
pub mod while_statement;
pub mod function;

pub trait Visitor<T, R> {
    fn visit(&mut self, visitor: T) -> R;
}

pub trait ASTNode: Debug {
    fn accept<'a, R>(&'a self, visitor: &mut dyn Visitor<&'a Self, R>) -> R where Self: Sized {
        visitor.visit(&self)
    }
}