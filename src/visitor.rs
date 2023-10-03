
pub trait Visitor<T, R> {
    fn visit(&mut self, visitor: T) -> R;
}

pub trait Visitable {
    fn accept<'a, R>(&'a self, visitor: &mut dyn Visitor<&'a Self, R>) -> R where Self: Sized {
        visitor.visit(&self)
    }
}