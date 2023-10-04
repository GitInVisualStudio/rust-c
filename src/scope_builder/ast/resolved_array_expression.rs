use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, DataType};

#[derive(Debug)]
pub enum ResolvedArrayExpression<'a> {
    StackArray {
        expressions: Vec<&'a ResolvedExpression<'a>>,
        data_type: DataType<'a>,
    },
    StringLiteral {
        string: &'a str,
        data_type: DataType<'a>,
    },
}

impl Visitable for ResolvedArrayExpression<'_> {}

impl<'a> ResolvedArrayExpression<'a> {
    pub fn data_type(&self) -> DataType<'a> {
        match self {
            ResolvedArrayExpression::StackArray { data_type, .. } => *data_type,
            ResolvedArrayExpression::StringLiteral { data_type, .. } => *data_type,
        }
    }
}
