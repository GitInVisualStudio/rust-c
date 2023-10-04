use crate::visitor::Visitable;

use super::resolved_function::ResolvedFunction;


#[derive(Debug)]
pub struct ResolvedProgram<'a> {
    pub(crate) functions: Vec<&'a ResolvedFunction<'a>>
}

impl Visitable for ResolvedProgram<'_> {}
