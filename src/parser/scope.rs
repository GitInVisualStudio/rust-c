use std::rc::Rc;

use super::{function::Function, variable::Variable};

#[derive(Debug)]
pub struct Scope {
    functions: Vec<Vec<Rc<Function>>>,
    variables: Vec<Vec<Rc<Variable>>>,
    stack_offset: usize,
}

pub trait IScope<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn add(&mut self, value: Rc<T>);
}

impl IScope<Variable> for Scope {
    fn get(&self, name: &str) -> Option<&Variable> {
        for vars in &self.variables {
            if let Some(x) = vars.iter().find(|x| x.name() == name) {
                return Some(x);
            }
        }
        None
    }

    fn add(&mut self, value: Rc<Variable>) {
        let vars = self.variables.last_mut();
        match vars {
            Some(x) => {
                self.stack_offset += value.data_type().size();
                x.push(value)
            }
            None => panic!("was not able to add variable into scope!"),
        }
    }
}

impl IScope<Function> for Scope {
    fn get(&self, name: &str) -> Option<&Function> {
        for funcs in &self.functions {
            if let Some(x) = funcs.iter().find(|x| x.name() == name) {
                return Some(x);
            }
        }
        None
    }

    fn add(&mut self, value: Rc<Function>) {
        let funcs = self.functions.last_mut();
        match funcs {
            Some(x) => x.push(value),
            None => panic!("was not able to add function into scope!"),
        }
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            functions: Vec::new(),
            variables: Vec::new(),
            stack_offset: 0,
        }
    }

    pub fn push(&mut self) {
        self.functions.push(Vec::new());
        self.variables.push(Vec::new());
    }

    pub fn pop(&mut self) {
        self.functions.pop();
        self.variables.pop();
        if self.functions.len() == 1 {
            self.stack_offset = 0;
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack_offset
    }

    pub fn add_stack(&mut self, size: usize) {
        self.stack_offset += size
    }
}
