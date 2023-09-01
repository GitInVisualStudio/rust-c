pub mod register;

use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    sync::atomic::AtomicUsize,
};

use self::register::Reg;

pub static CLAUSE_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static LABEL_COUNT: AtomicUsize = AtomicUsize::new(0);
pub struct Generator {
    writer: BufWriter<File>,
}

impl Generator {
    pub fn new(file_name: &str) -> Result<Generator, Error> {
        let file = File::create(file_name)?;
        Ok(Generator {
            writer: BufWriter::new(file),
        })
    }

    pub fn emit(&mut self, string: &str) -> Result<usize, Error> {
        self.writer.write(string.as_bytes())
    }

    pub fn emit_ins(&mut self, ins: &str, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit(&format!("\t{}\t{}, {}\n", ins, from, to))
    }

    pub fn emit_sins(&mut self, ins: &str, reg: Reg) -> Result<usize, Error> {
        self.emit(&format!("\t{}\t{}\n", ins, reg))
    }

    pub fn mov(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("mov ", from, to)
    }

    pub fn add(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("add ", from, to)
    }

    pub fn sub(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("sub ", from, to)
    }

    pub fn mul(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("imul", from, to)
    }

    pub fn cmp(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("cmp ", from, to)
    }

    pub fn gen_cmp(&mut self, ins: &str, from: Reg, to: Reg) -> Result<usize, Error> {
        self.cmp(from, to)?;
        self.mov(Reg::IMMEDIATE(0), to)?;
        let prev = Reg::set_size(1);
        let result = self.emit_sins(ins, to);
        Reg::set_size(prev);
        result
    }

    pub fn lea(&mut self, from: Reg, to: Reg) -> Result<usize, Error> {
        self.emit_ins("lea ", from, to)
    }

    pub fn push_stack(&mut self, size: usize) -> Result<usize, Error> {
        self.emit(&format!(
            "\tpush\t%rbp\n\tmov \t%rsp, %rbp\n\tsub \t${}, %rsp\n",
            (size / 16 + 1) * 16
        ))
    }

    pub fn pop_stack(&mut self) -> Result<usize, Error> {
        self.emit("\tleave\n")
    }

    pub fn emit_label(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("{}:\n", label))
    }

    pub fn call(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(&format!("\tcall \t{}\n", label))
    }

    pub fn emit_string(&mut self, label: usize, string: &str) -> Result<usize, Error> {
        self.emit(&format!(
            "    .section   .rodata
.LC{}:
    .string	{}
    .text
",
            label, string
        ))
    }

    pub fn generate_clause_names() -> (String, String) {
        let clause_count = CLAUSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (
            format!("_clause{}", clause_count),
            format!("_end{}", clause_count),
        )
    }

    pub fn generate_label_names(label_index: usize) -> (String, String, String) {
        (
            format!("_label{}", label_index),
            format!("_labelend{}", label_index),
            format!("_expression{}", label_index),
        )
    }

    pub fn next_label_index() -> usize {
        LABEL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    pub fn label_index() -> usize {
        LABEL_COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }
}
