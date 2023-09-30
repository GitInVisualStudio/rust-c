use crate::lexer::{tokens::Token, Lexer, LexerError};

pub struct Preprocessor<'a> {
    lexer: Lexer<'a>,
    input: &'a str,
    macros: Vec<Macro>,
    expanded: bool,
}

#[derive(Debug)]
pub enum Macro {
    Literal {
        name: String,
        replacement: String,
    },
    Function {
        name: String,
        parameter: Vec<String>,
        replacement: String,
    },
}

impl Macro {
    pub fn name(&self) -> &String {
        match self {
            Macro::Literal {
                name,
                replacement: _,
            } => name,
            Macro::Function {
                name,
                parameter: _,
                replacement: _,
            } => name,
        }
    }
}

impl Preprocessor<'_> {
    pub fn new<'a>(content: &'a str) -> Preprocessor<'a> {
        Preprocessor {
            lexer: Lexer::new(content),
            input: content,
            macros: Vec::new(),
            expanded: false,
        }
    }

    pub fn expand(&mut self) -> Result<String, LexerError> {
        let mut output = String::new();

        loop {
            let token = self.lexer.next();

            if token == Token::HASHTAG {
                self.parse_macro(&mut output)?;
                continue;
            }

            if token == Token::EOF {
                break;
            }

            self.expand_macros(&mut output)?;
        }

        // expand recursivly until we are finished!
        if self.expanded {
            let mut child = Preprocessor::new(&mut output);
            return child.expand();
        }

        //TODO: remove the defintions
        let mut lexer = Lexer::new(&output);
        let mut output = String::new();
        while lexer.peek() != Token::EOF {
            while lexer.peek() == Token::HASHTAG {
                lexer.consume_line();
            }
            lexer.next();
            output += lexer.last_string();
            output += " ";
        }

        output = output.replace("\\\n", " ");

        Ok(output)
    }

    fn parse_define(&mut self, output: &mut String) -> Result<Macro, LexerError> {
        output.push_str(&"\n#");
        output.push_str(self.lexer.expect(Token::DEFINE)?);
        output.push(' ');

        let name = self.lexer.expect(Token::IDENT)?.to_string();

        output.push_str(&name);
        let mut parameter = None;

        if self.lexer.peek() == Token::LPAREN {
            self.lexer.next();
            output.push('(');
            let mut parameter_list = Vec::new();
            loop {
                let name = self.lexer.expect(Token::IDENT)?.to_string();
                output.push_str(&name);
                parameter_list.push(name);

                if self.lexer.peek() == Token::RPAREN {
                    break;
                }

                self.lexer.expect(Token::COMMA)?;
                output.push(',');
            }
            self.lexer.next();
            output.push(')');
            parameter = Some(parameter_list);
        }

        let start_index = self.lexer.current_index();
        self.lexer.consume_line();
        let end_index = self.lexer.current_index();

        let replacement = self.input[start_index..end_index].to_string();
        output.push_str(&replacement);
        output.push('\n');

        Ok(match parameter {
            Some(parameter) => Macro::Function {
                name: name,
                parameter: parameter,
                replacement: replacement,
            },
            None => Macro::Literal {
                name: name,
                replacement: replacement,
            },
        })
    }

    fn parse_macro(&mut self, output: &mut String) -> Result<(), LexerError> {
        match self.lexer.peek() {
            Token::DEFINE => {
                let parsed_macro = self.parse_define(output)?;
                self.macros.push(parsed_macro);
                return Ok(());
            }
            Token::INCLUDE => {
                self.expanded = true;
                self.lexer.expect(Token::INCLUDE)?;
                let name = self.lexer.expect(Token::STRINGLIT)?;
                let file_name = name[1..name.len() - 1].to_string();

                let file = std::fs::read_to_string(&file_name);
                return match file {
                    Ok(x) => {
                        output.push_str(&x);
                        return Ok(());
                    }
                    Err(_) => self.lexer.error(format!("Cannot open file: {file_name}")),
                };
            }
            x => panic!("Can't parse macro! Found token: {:?}", x),
        }
    }

    fn expand_macros(&mut self, output: &mut String) -> Result<bool, LexerError> {
        let name = self.lexer.last_string();
        if let Some(m) = self.macros.iter().find(|x| x.name() == name) {
            self.expanded = true;
            match m {
                Macro::Literal {
                    name: _,
                    replacement,
                } => output.push_str(&replacement),
                Macro::Function {
                    name: _,
                    parameter,
                    replacement,
                } => {
                    self.lexer.expect(Token::LPAREN)?;

                    let mut arguments = Vec::new();
                    loop {
                        let mut parathese = 0;
                        let start_index = self.lexer.current_index();

                        loop {
                            if self.lexer.peek() == Token::LPAREN {
                                parathese += 1;
                            }
                            if (self.lexer.peek() == Token::RPAREN
                                || self.lexer.peek() == Token::COMMA)
                                && parathese == 0
                            {
                                break;
                            }
                            if self.lexer.peek() == Token::RPAREN {
                                parathese -= 1;
                            }
                            if self.lexer.peek() == Token::EOF {
                                panic!("Cannot parse MACRO argument!");
                            }
                            self.lexer.next();
                        }

                        let end_index = self.lexer.current_index();
                        let argument = self.input[start_index..end_index].to_string();
                        arguments.push(argument);

                        if self.lexer.peek() == Token::RPAREN {
                            break;
                        }
                        self.lexer.expect(Token::COMMA)?;
                    }

                    self.lexer.expect(Token::RPAREN)?;

                    let mut replacement_lexer = Lexer::new(&replacement);

                    loop {
                        match replacement_lexer.next() {
                            Token::IDENT => {
                                let argument_name = replacement_lexer.last_string();
                                let index = parameter
                                    .iter()
                                    .enumerate()
                                    .find(|(_, name)| argument_name == *name);
                                match index {
                                    Some((index, _)) => output.push_str(&arguments[index]),
                                    None => output.push_str(replacement_lexer.last_string()),
                                }
                            }
                            Token::EOF => {
                                break;
                            }
                            _ => output.push_str(replacement_lexer.last_string()),
                        }
                    }
                }
            }
            return Ok(true);
        }

        output.push_str(" ");
        output.push_str(self.lexer.last_string());
        return Ok(false);
    }
}
