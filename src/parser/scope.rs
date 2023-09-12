use std::rc::Rc;

use crate::ast::{
    data_type::Struct, function::Function, type_definition::TypeDefinition, variable::Variable,
};

#[derive(Debug)]
pub struct Scope {
    functions: Vec<Vec<Rc<Function>>>,
    variables: Vec<Vec<Variable>>,
    typedefs: Vec<TypeDefinition>,
    structs: Vec<Rc<Struct>>,
    stack_offset: usize,
}

pub trait IScope<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn add(&mut self, value: T);
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

        fn add(&mut self, value: $t) {
            self.$var.push(value);
        }
    };
}

impl IScope<Variable> for Scope {
    scope_get!(Variable, variables);

    fn add(&mut self, value: Variable) {
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

impl IScope<Rc<Function>> for Scope {
    scope_get!(Rc<Function>, functions);

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

impl IScope<Rc<Struct>> for Scope {
    scope!(Rc<Struct>, structs);
}

impl Scope {
    pub fn new<'a>() -> Scope {
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
