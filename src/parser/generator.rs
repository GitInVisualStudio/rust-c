use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    sync::atomic::AtomicIsize,
};

pub static CLAUSE_COUNT: AtomicIsize = AtomicIsize::new(0);
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

    pub fn emit(&mut self, string: String) -> Result<usize, Error> {
        self.writer.write(string.as_bytes())
    }

    pub fn emit_ins(
        &mut self,
        instruction: &str,
        first: &str,
        second: &str,
    ) -> Result<usize, Error> {
        self.emit(format!("\t{}\t{}, {}\n", instruction, first, second))
    }

    pub fn mov(&mut self, value: i32, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tmov \t${}, %{}\n", value, register))
    }

    pub fn push(&mut self, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tpush\t%{}\n", register))
    }

    pub fn pop(&mut self, register: &str) -> Result<usize, Error> {
        self.emit(format!("\tpop \t%{}\n", register))
    }

    pub fn push_stack(&mut self, size: usize) -> Result<usize, Error> {
        self.emit(format!(
            "\tpush\t%rbp\n\tmovq\t%rsp, %rbp\n\tsub \t${}, %rsp\n",
            size
        ))
    }

    pub fn pop_stack(&mut self) -> Result<usize, Error> {
        self.emit("\tmov\t\t%rbp, %rsp\n\tpop\t\t%rbp\n".to_string())
    }

    pub fn emit_cmp(&mut self, comparator: &str) -> Result<usize, Error> {
        self.emit(format!(
            "\tcmp \t%eax, %ecx\n\tmov \t$0, %eax\n\t{}\t%al\n",
            comparator
        ))
    }

    pub fn emit_label(&mut self, label: &str) -> Result<usize, Error> {
        self.emit(format!("{}:\n", label))
    }

    pub fn generate_clause_names() -> (String, String) {
        let clause_count = CLAUSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (
            format!("_clause{}", clause_count),
            format!("_end{}", clause_count),
        )
    }

    pub fn generate_label_names() -> (String, String, String) {
        let clause_count = CLAUSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (
            format!("_clause{}", clause_count),
            format!("_expression{}", clause_count),
            format!("_end{}", clause_count),
        )
    }
}
