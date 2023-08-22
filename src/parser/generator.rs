use std::{io::{BufWriter, Error, Write}, fs::File};

pub struct Generator {
    writer: BufWriter<File>
}

impl Generator {
    pub fn new(file_name: &str) -> Result<Generator, Error> {
        let file = File::create(file_name)?;
        Ok( Generator { writer: BufWriter::new(file) })
    }

    pub fn emit(&mut self, string: String) -> Result<usize, Error> {
        self.writer.write(string.as_bytes())
    }

    pub fn emit_ins(&mut self, instruction: &str, first: &str, second: &str) -> Result<usize, Error> {
        self.emit(format!("\t{}\t{}, {}\n", instruction, first, second))
    }

    pub fn mov(&mut self, value: i32, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tmovl\t${}, %{}\n", value, register))
    }

    pub fn push(&mut self, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tpush\t%{}\n", register))
    }

    pub fn pop(&mut self, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tpop \t%{}\n", register))
    }
}