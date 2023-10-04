use crate::visitor::Visitable;

use super::{resolved_compound::ResolvedCompound, DataType};

#[derive(Debug)]
pub struct ResolvedFunction<'a> {
    pub(crate) name: &'a str,
    pub(crate) statements: Option<&'a ResolvedCompound<'a>>,
    pub(crate) parameter: Vec<(DataType<'a>, &'a str)>,
    pub(crate) return_type: DataType<'a>,
}

impl Visitable for ResolvedFunction<'_> {}
