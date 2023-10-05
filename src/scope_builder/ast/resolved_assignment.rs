use crate::visitor::Visitable;

use super::{resolved_expression::ResolvedExpression, DataType, Variable};

#[derive(Debug)]
pub enum ResolvedAssignment<'a> {
    StackAssignment {
        variable: Variable<'a>,
        expression: &'a ResolvedExpression<'a>,
    },
    PtrAssignment {
        data_type: DataType<'a>,
        value: &'a ResolvedExpression<'a>,
        address: &'a ResolvedExpression<'a>,
    },
    ArrayAssignment {
        data_type: DataType<'a>,
        index: &'a ResolvedExpression<'a>,
        value: &'a ResolvedExpression<'a>,
        address: &'a ResolvedExpression<'a>,
    },
    FieldAssignment {
        field_offset: usize,
        data_type: DataType<'a>,
        value: &'a ResolvedExpression<'a>,
        address: &'a ResolvedExpression<'a>,
    },
}

impl Visitable for ResolvedAssignment<'_> {}

impl<'a> ResolvedAssignment<'a> {
    pub fn data_type(&self) -> DataType<'a> {
        *match self {
            ResolvedAssignment::StackAssignment { variable, .. } => &variable.data_type,
            ResolvedAssignment::PtrAssignment { data_type, .. } => data_type,
            ResolvedAssignment::ArrayAssignment { data_type, .. } => data_type,
            ResolvedAssignment::FieldAssignment { data_type, .. } => data_type,
        }
    }
}
