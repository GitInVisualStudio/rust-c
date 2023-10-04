use crate::{
    lexer::{tokens::TokenKind, SrcLocation},
    parser::ast::{Expression, UnaryOps},
    scope_builder::ast::DataType,
};

#[derive(Debug, Clone)]
pub enum Error<'a> {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        location: SrcLocation<'a>,
    },
    UnableToAssign {
        location: SrcLocation<'a>,
    },
    UnknownType {
        type_name: &'a str,
    },
    StructRedefinition {
        name: &'a str,
    },
    StructFieldRedefinition {
        struct_name: &'a str,
        field_name: &'a str,
    },
    StructFieldUnkownSize {
        struct_name: &'a str,
        field_name: &'a str,
    },
    OperandsDifferentDatatypes {
        lhs: &'a Expression<'a>,
        rhs: &'a Expression<'a>,
    },
    DerefOfNonPointer {
        expr: &'a Expression<'a>,
    },
    UnknownVariable {
        name: &'a str,
    },
    AccessNonStruct {
        expression: &'a Expression<'a>,
    },
    UnknownField {
        expression: &'a Expression<'a>,
        name: &'a str,
    },
    CannotAssign {
        from: &'a Expression<'a>,
        to: &'a Expression<'a>,
    },
    CannotAssignVariable {
        assignment: &'a Expression<'a>,
        variable: &'a str,
    },
    ArrayIndexNotANumber {
        index: &'a Expression<'a>,
    },
    EmptyArray {},
    ArrayOfDifferentTypes {},
    UnkownFunction {
        name: &'a str,
    },
    ReturnTypeIncorrect {
        expected: DataType<'a>,
        found: DataType<'a>,
    },
    ReturnWithoutFunction {},
    VariableRedefinition {
        name: &'a str,
    },
    VariableInitWrong {
        expected: DataType<'a>,
        found: DataType<'a>,
        name: &'a str,
    },
    VariableDeclarationOutsideOfFunction {
        name: &'a str,
    },
    VariableOfUnkownSize {
        name: &'a str,
        data_type: DataType<'a>,
    },
    ParameterCountMismatch {
        function: &'a str,
        expected: usize,
        found: usize,
    },
    ParamterTypeMismatch {
        function: &'a str,
        expected: DataType<'a>,
        found: DataType<'a>,
        parameter_name: &'a str,
    },
    UnaryOperandNotNumber {
        expression: &'a Expression<'a>,
        operation: UnaryOps<'a>
    }
}
