use super::{function::Function, variable::Variable};


#[derive(Debug)]
pub struct Scope {
    functions: Vec<Function>,
    variables: Vec<Variable>
}

pub trait IScope<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn add(&mut self, value: T);
}

impl IScope<Variable> for Scope {
    fn get(&self, name: &str) -> Option<&Variable> {
        self.variables.iter().find(|x| x.name() == name)
    }

    fn add(&mut self, value: Variable) {
        self.variables.push(value)
    }
}

impl IScope<Function> for Scope {
    fn get(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|x| x.name() == name)
    }

    fn add(&mut self, value: Function) {
        self.functions.push(value)
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope { functions: Vec::new(), variables: Vec::new() }
    }
}