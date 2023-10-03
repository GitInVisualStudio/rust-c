use self::tokens::TokenKind;

pub mod tokens;

#[derive(Copy, Clone, Debug)]
pub struct SrcLocation<'a> {
    pub(crate) src: &'a str,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> SrcLocation<'a> {
    pub fn string(&self) -> &'a str {
        self.src
    }
}

pub type Token<'a> = (TokenKind, SrcLocation<'a>);

pub mod Lexer {

    use super::{
        tokens::{TokenKind, TOKEN_PATTERN},
        SrcLocation, Token,
    };
    use regex::Regex;

    pub fn tokenize<'a>(content: &'a str) -> Vec<Token<'a>> {
        let patterns: Vec<(TokenKind, Regex)> = TOKEN_PATTERN
            .iter()
            .enumerate()
            .map(|(index, pattern)| (TokenKind::from(index), Regex::new(pattern).unwrap()))
            .collect();

        let mut tokens = Vec::new();
        let mut index = 0;
        let mut last_index;

        let mut column = 0;
        let mut line_breaks = 1;

        loop {
            last_index = index;
            let next_token = next_token(&mut index, content, &patterns);
            let token_string = &content[last_index..index];

            for i in token_string.bytes() {
                column += 1;
                if i == b'\n' {
                    line_breaks += 1;
                    column = 0;
                }
            }

            let token = (
                next_token,
                SrcLocation {
                    src: token_string.trim(),
                    line: line_breaks,
                    column: column,
                },
            );
            tokens.push(token);

            if next_token == TokenKind::EOF {
                break;
            }
        }
        tokens
    }

    fn next_token(
        index: &mut usize,
        content: &str,
        patterns: &Vec<(TokenKind, Regex)>,
    ) -> TokenKind {
        while *index < content.len() && content.as_bytes()[*index].is_ascii_whitespace() {
            *index += 1;
        }
        if *index == content.len() {
            return TokenKind::EOF;
        }

        let result = patterns
            .iter()
            .map(|(token, regex)| (token, regex.find_at(&content, *index)))
            .filter_map(|(token, x)| match x {
                Some(m) if m.start() == *index && m.len() > 0 => Some((token, m.end())),
                Some(_) => None,
                None => None,
            })
            .next();
        if let Some((token, end)) = result {
            *index = end;
            return token.clone();
        }

        panic!("was not able to tokinize {content}");
    }
}
