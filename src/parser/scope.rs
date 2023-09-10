use std::rc::Rc;

use super::{
    data_type::Struct, function::Function, type_definition::TypeDefinition, variable::Variable,
};

#[derive(Debug)]
pub struct Scope {
    functions: Vec<Vec<Rc<Function>>>,
    variables: Vec<Vec<Rc<Variable>>>,
    typedefs: Vec<Rc<TypeDefinition>>,
    structs: Vec<Rc<Struct>>,
    stack_offset: usize,
}

pub trait IScope<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn get_rc(&self, name: &str) -> Option<Rc<T>>;
    fn add(&mut self, value: Rc<T>);
}

macro_rules! scope_get {
    ($t: ty, $var: ident) => {
        fn get(&self, name: &str) -> Option<&$t> {
            for vars in &self.$var {
                if let Some(x) = vars.iter().find(|x| x.name() == name) {
                    return Some(x);
                }
            }
            None
        }
        fn get_rc(&self, name: &str) -> Option<Rc<$t>> {
            for vars in &self.$var {
                if let Some(x) = vars.iter().find(|x| x.name() == name) {
                    return Some(x.clone());
                }
            }
            None
        }
    };
}

macro_rules! scope {
    ($t: ty, $var: ident) => {
        fn get(&self, name: &str) -> Option<&$t> {
            if let Some(x) = self.$var.iter().find(|x| x.name() == name) {
                return Some(x);
            }
            None
        }

        fn get_rc(&self, name: &str) -> Option<Rc<$t>> {
            if let Some(x) = self.$var.iter().find(|x| x.name() == name) {
                return Some(x.clone());
            }
            None
        }

        fn add(&mut self, value: Rc<$t>) {
            self.$var.push(value);
        }
    };
}

impl IScope<Variable> for Scope {
    scope_get!(Variable, variables);

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
    scope_get!(Function, functions);

    fn add(&mut self, value: Rc<Function>) {
        let funcs = self.functions.last_mut();
        match funcs {
            Some(x) => x.push(value),
            None => panic!("was not able to add function into scope!"),
        }
    }
}

impl IScope<TypeDefinition> for Scope {
    scope!(TypeDefinition, typedefs);
}

impl IScope<Struct> for Scope {
    scope!(Struct, structs);
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            structs: Vec::new(),
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

    pub fn get_structs(&self) -> &Vec<Rc<Struct>> {
        &self.structs
    }

    pub fn contains<T>(&self, name: &str) -> bool
    where
        Scope: IScope<T>,
    {
        let contains: Option<&T> = self.get(name);
        contains.is_some()
    }
}
