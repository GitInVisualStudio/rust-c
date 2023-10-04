pub mod data_type;
pub mod variable;
pub mod resolved_expression;
pub mod resolved_function_call;
pub mod resolved_array_expression;
pub mod resolved_struct_expression;
pub mod resolved_assignment;
pub mod resolved_statement;
pub mod resolved_compound;
pub mod resolved_if;
pub mod resolved_for;
pub mod resolved_while;
pub mod resolved_program;
pub mod resolved_function;

pub use data_type::*;
pub use variable::*;
