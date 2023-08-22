
#[derive(Debug)]
pub enum DataType {
    INT,
}

#[derive(Debug)]
pub struct Variable {
    name: String,
    data_type: DataType
}

impl Variable {
    pub fn new(name: &str, data_type: DataType) -> Variable {
        Variable { name: name.to_owned(), data_type: data_type }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}