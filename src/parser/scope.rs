
#[derive(Debug)]
pub struct Scope {
}

pub trait IScope<T> {
    fn get(&self, name: &str) -> Option<&T>;
    fn add(&mut self, value: T);
}

impl Scope {
    pub fn new() -> Scope {
        Self {}
    }
}