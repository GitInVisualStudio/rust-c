use crate::lexer::{tokens::TokenKind, SrcLocation};

#[derive(Debug, Clone)]
pub enum Error<'a> {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        location: SrcLocation<'a>,
    },
    UnableToAssign {
        location: SrcLocation<'a>
    }
}
