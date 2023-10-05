use crate::visitor::Visitable;

use super::{DataType, resolved_assignment::ResolvedAssignment};

#[derive(Debug)]
pub enum ResolvedArrayExpression<'a> {
    StackArray {
        expressions: Vec<&'a ResolvedAssignment<'a>>,
        data_type: DataType<'a>,
        stack_offset: usize,
    },
    StringLiteral {
        string: &'a str,
        data_type: DataType<'a>,
        string_label_index: i32
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
