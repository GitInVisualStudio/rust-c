use crate::{
    parser::ast::{BinaryOps, UnaryOps},
    visitor::Visitable,
};

use super::{
    resolved_array_expression::ResolvedArrayExpression, resolved_assignment::ResolvedAssignment,
    resolved_function_call::ResolvedFunctionCall,
    resolved_struct_expression::ResolvedStructExpression, DataType, Variable,
};

#[derive(Debug)]
pub enum ResolvedExpression<'a> {
    IntLiteral(i32),
    CharLiteral(u8),
    FunctionCall(&'a ResolvedFunctionCall<'a>),
    ArrayExpression(&'a ResolvedArrayExpression<'a>),
    StructExpresion(&'a ResolvedStructExpression<'a>),
    Assignment(&'a ResolvedAssignment<'a>),
    TypeExpression(DataType<'a>),
    SizeOf(usize),
    FieldAccess {
        field_offset: usize,
        data_type: DataType<'a>,
        operand: &'a ResolvedExpression<'a>,
    },
    ArrowAccess {
        field_offset: usize,
        data_type: DataType<'a>,
        operand: &'a ResolvedExpression<'a>,
    },
    Indexing {
        data_type: DataType<'a>,
        index: &'a ResolvedExpression<'a>,
        operand: &'a ResolvedExpression<'a>,
    },
    NamedVariable {
        variable: Variable<'a>,
    },
    Unary {
        expression: &'a ResolvedExpression<'a>,
        operation: UnaryOps<'a>,
        resulting_type: DataType<'a>,
    },
    Cast {
        expression: &'a ResolvedExpression<'a>,
        data_type: DataType<'a>,
    },
    BinaryExpression {
        lhs: &'a ResolvedExpression<'a>,
        rhs: &'a ResolvedExpression<'a>,
        operation: BinaryOps,
        resulting_type: DataType<'a>,
    },
}

impl Visitable for ResolvedExpression<'_> {}

impl<'a> ResolvedExpression<'a> {
    pub fn data_type(&self) -> DataType<'a> {
        match self {
            ResolvedExpression::IntLiteral(_) => DataType::INT,
            ResolvedExpression::CharLiteral(_) => DataType::CHAR,
            ResolvedExpression::FunctionCall(x) => x.return_type,
            ResolvedExpression::ArrayExpression(a) => a.data_type(),
            ResolvedExpression::StructExpresion(s) => s.data_type,
            ResolvedExpression::Assignment(a) => a.data_type(),
            ResolvedExpression::TypeExpression(t) => *t,
            ResolvedExpression::SizeOf(_) => DataType::INT,
            ResolvedExpression::FieldAccess { data_type, .. } => *data_type,
            ResolvedExpression::ArrowAccess { data_type, .. } => *data_type,
            ResolvedExpression::Indexing { data_type, .. } => *data_type,
            ResolvedExpression::NamedVariable { variable } => variable.data_type,
            ResolvedExpression::Unary { resulting_type, .. } => *resulting_type,
            ResolvedExpression::Cast { data_type, .. } => *data_type,
            ResolvedExpression::BinaryExpression { resulting_type, .. } => *resulting_type,
        }
    }
}
