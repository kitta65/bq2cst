#[cfg(test)]
mod tests;

use crate::token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    ch: Option<char>, // TODO delete
    line: usize,
    column: usize,
    previous_token: Option<crate::token::Token>, // TODO delete
    type_declaration_depth: usize,
    tokens: Vec<token::Token>
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let chars: Vec<char> = input.chars().collect();
        let first_char = chars[0];
        Lexer {
            input: chars,
            position: 0,
            ch: Some(first_char),
            line: 0,
            column: 0,
            previous_token: None,
            type_declaration_depth: 0,
            tokens: Vec::new()
        }
    }
    fn tokenize_code(&mut self) -> &Vec<token::Token> {
        let mut token = self.next_token();
        while !token.is_none() {
            self.next_token();
            token = self.next_token();
        }
        &self.tokens
    }
    fn read_char(&mut self) {
        if self.input.len() <= self.position + 1 {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.position + 1]);
            if self.input[self.position] == '\n' {
                self.column = 0;
                self.line += 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }
    fn read_escaped_char(&mut self) {
        self.read_char(); // '\\' -> ?
        match self.ch {
            Some('x') => {
                self.read_char();
                self.read_char();
            }
            Some('u') => {
                for _ in 0..4 {
                    self.read_char();
                }
            }
            Some('U') => {
                for _ in 0..8 {
                    self.read_char();
                }
            }
            Some('0') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('1') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('2') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('3') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('4') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('5') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('6') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some('7') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some(_) => (),
            None => panic!(),
        }
        self.read_char();
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        while is_letter_or_digit(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_parameter(&mut self) -> String {
        let first_position = self.position;
        while self.ch == Some('@') {
            self.read_char();
        }
        self.read_identifier();
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    pub fn next_token(&mut self) -> Option<token::Token> {
        self.skip_whitespace();
        let ch = match self.ch {
            Some(ch) => ch,
            None => {
                return None;
            }
        };
        let token = match ch {
            ',' => token::Token {
                literal: ch.to_string(),
                line: self.line,
                column: self.column,
            },
            ';' => token::Token {
                literal: ch.to_string(),
                line: self.line,
                column: self.column,
            },
            '.' => {
                let next_ch = self.peek_char(1).unwrap();
                if next_ch == '`' || next_ch.is_alphabetic() {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                } else {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_number(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                }
            }
            '#' => {
                let token = Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_comment(),
                });
                self.previous_token = token.clone();
                self.tokens.push(token.clone().unwrap());
                return token;
            }
            // quotation
            '`' => {
                let token = Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_quoted(self.ch),
                });
                self.previous_token = token.clone();
                self.tokens.push(token.clone().unwrap());
                return token;
            }
            '"' => {
                if self.peek_char(1) == Some('"') && self.peek_char(2) == Some('"') {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_string(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_quoted(self.ch),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                }
            }
            '\'' => {
                if self.peek_char(1) == Some('\'') && self.peek_char(2) == Some('\'') {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_string(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_quoted(self.ch),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                }
            }
            // operators
            //'+' => token::Token {
            //    literal: ch.to_string(),
            //    line: self.line,
            //    column: self.column,
            //},
            '-' => {
                if self.peek_char(1) == Some('-') {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_comment(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            //'*' => token::Token {
            //    line: self.line,
            //    column: self.column,
            //    literal: ch.to_string(),
            //},
            '/' => {
                if self.peek_char(1) == Some('*') {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_comment(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '|' => {
                if self.peek_char(1) == Some('|') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "||".to_string(),
                    }
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '<' => {
                if self.peek_char(1) == Some('<') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "<<".to_string(),
                    }
                } else if self.peek_char(1) == Some('=') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "<=".to_string(),
                    }
                } else if self.peek_char(1) == Some('>') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "<>".to_string(),
                    }
                } else {
                    if self.previous_token.clone().unwrap().literal.to_uppercase() == "ARRAY"
                        || self.previous_token.clone().unwrap().literal.to_uppercase() == "STRUCT"
                    {
                        self.type_declaration_depth += 1;
                    }
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '>' => {
                if 0 < self.type_declaration_depth {
                    self.type_declaration_depth -= 1;
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                } else if self.peek_char(1) == Some('>') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: ">>".to_string(),
                    }
                } else if self.peek_char(1) == Some('=') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: ">=".to_string(),
                    }
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '=' => {
                if self.peek_char(1) == Some('>') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "=>".to_string(),
                    }
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '!' => {
                if self.peek_char(1) == Some('=') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "!=".to_string(),
                    }
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            // parameter
            '?' => token::Token {
                line: self.line,
                column: self.column,
                literal: ch.to_string(),
            },
            '@' => {
                let token = Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_parameter(),
                });
                self.previous_token = token.clone();
                self.tokens.push(token.clone().unwrap());
                return token;
            }
            // other
            _ => {
                if ch.is_digit(10) {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_number(),
                    });
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else if is_letter_or_digit(&self.ch) {
                    let token = Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_identifier(),
                    }); // note: the ownerwhip moves
                    self.previous_token = token.clone();
                    self.tokens.push(token.clone().unwrap());
                    return token;
                } else {
                    token::Token {
                        literal: ch.to_string(),
                        line: self.line,
                        column: self.column,
                    }
                }
            }
        };
        self.read_char();
        self.previous_token = Some(token.clone());
        self.tokens.push(token.clone());
        Some(token)
    }
    fn read_multiline_string(&mut self) -> String {
        let first_position = self.position;
        let ch = self.ch;
        self.read_char(); // first ' -> second '
        while !(self.ch == ch && self.peek_char(1) == ch && self.peek_char(2) == ch) {
            if self.ch == Some('\\') {
                self.read_escaped_char();
            } else {
                self.read_char();
            }
        }
        self.read_char(); // first ' -> secont '
        self.read_char(); // second ' -> third '
        self.read_char(); // third ' ->  next_ch
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_quoted(&mut self, quote: Option<char>) -> String {
        let first_position = self.position;
        self.read_char();
        while self.ch != quote {
            if self.ch == Some('\\') {
                self.read_escaped_char();
            } else {
                self.read_char();
            }
        }
        self.read_char();
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn skip_whitespace(&mut self) {
        while is_whitespace(&self.ch) {
            self.read_char();
        }
    }
    fn peek_char(&mut self, offset: usize) -> Option<char> {
        if self.input.len() <= self.position + 1 {
            None
        } else {
            Some(self.input[self.position + offset])
        }
    }
    fn read_number(&mut self) -> String {
        let first_position = self.position;
        while is_digit_or_period(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_comment(&mut self) -> String {
        let first_position = self.position;
        while !is_end_of_line(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_multiline_comment(&mut self) -> String {
        let first_position = self.position;
        while !(self.ch == Some('*') && self.peek_char(1) == Some('/')) {
            self.read_char();
        }
        self.read_char(); // * -> /
        self.read_char(); // / -> next_char
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
}

fn is_letter_or_digit(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch.is_digit(10) || ch == &'_',
        None => false,
    }
}

fn is_digit_or_period(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_digit(10) || ch == &'.' || ch == &'E' || ch == &'e',
        None => false,
    }
}

fn is_end_of_line(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch == &'\n',
        None => true, // EOF is treated as end of line
    }
}

fn is_whitespace(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_whitespace(),
        None => false, // EOF is treated as end of line
    }
}
