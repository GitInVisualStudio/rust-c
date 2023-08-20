#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    VOID,
    INT,
    SEMIC,
    LCURL,
    RCURL,
    LPAREN,
    RPAREN,
    RETURN,
    INTLITERAL,
    IDENT,
    EOF,
    ERR,
}

impl Token {
    pub fn from(value: usize) -> Token {
        match value {
            0 => Token::VOID,
            1 => Token::INT,
            2 => Token::SEMIC,
            3 => Token::LCURL,
            4 => Token::RCURL,
            5 => Token::LPAREN,
            6 => Token::RPAREN,
            7 => Token::RETURN,
            8 => Token::INTLITERAL,
            9 => Token::IDENT,
            10 => Token::EOF,
            _ => Token::ERR
        }
    }
}

pub static TOKEN_PATTERN: &'static [&'static str] = &[
    "void",
    "int",
    ";",
    "\\{",
    "\\}",
    "\\(",
    "\\)",
    "return ",
    "[0-9]+",
    "\\w+"
];