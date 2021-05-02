#[cfg(test)]
mod tests;

use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    type_declaration_depth: usize,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            input: chars,
            position: 0,
            line: 0,
            column: 0,
            type_declaration_depth: 0,
            tokens: Vec::new(),
        }
    }
    pub fn tokenize_code(&mut self) -> &Vec<Token> {
        let mut token = self.next_token();
        while !token.is_none() {
            self.next_token();
            token = self.next_token();
        }
        self.tokens.push(Token::new(usize::MAX, usize::MAX, "")); // EOF token
        &self.tokens
    }
    fn get_curr_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            return Some(self.input[self.position]);
        } else {
            return None; // EOF
        }
    }
    fn read_char(&mut self) {
        if self.position < self.input.len() {
            if self.input[self.position] == '\n' {
                self.column = 0;
                self.line += 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        } else {
            panic!("`read_char()` is called at EOF");
        }
    }
    fn skip_escaped_char(&mut self) {
        self.read_char(); // '\\' -> ?
        match self.get_curr_char() {
            // https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical#literals
            Some('x') => {
                for _ in 0..2 {
                    self.read_char();
                }
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
            Some('1'..='7') => {
                for _ in 0..3 {
                    self.read_char();
                }
            }
            Some(_) => (), // \n, \t, ...
            None => panic!(),
        }
        self.read_char();
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        while is_letter_or_digit(&self.get_curr_char()) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_parameter(&mut self) -> String {
        let first_position = self.position;
        while self.get_curr_char() == Some('@') {
            self.read_char();
        }
        self.read_identifier();
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn construct_token(&mut self, line: usize, column: usize, literal: String) -> &Token {
        let token = Token::new(line, column, literal.as_str());
        self.tokens.push(token);
        &self.tokens.last().unwrap()
    }
    fn next_token(&mut self) -> Option<&Token> {
        self.skip_whitespace();
        let ch = match self.get_curr_char() {
            Some(ch) => ch,
            None => {
                return None; // EOF
            }
        };
        let line = self.line;
        let column = self.column;
        let token = match ch {
            '.' => {
                let next_ch = self.peek_char(1).unwrap();
                if next_ch == '`' || next_ch.is_alphabetic() {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                } else {
                    let literal = self.read_number();
                    self.construct_token(line, column, literal)
                }
            }
            '#' => {
                let literal = self.read_comment();
                self.construct_token(line, column, literal)
            }
            // quotation
            '`' => {
                let literal = self.read_quoted();
                self.construct_token(line, column, literal)
            }
            '"' => {
                if self.peek_char(1) == Some('"') && self.peek_char(2) == Some('"') {
                    let literal = self.read_multiline_string();
                    self.construct_token(line, column, literal)
                } else {
                    let literal = self.read_quoted();
                    self.construct_token(line, column, literal)
                }
            }
            '\'' => {
                if self.peek_char(1) == Some('\'') && self.peek_char(2) == Some('\'') {
                    let literal = self.read_multiline_string();
                    self.construct_token(line, column, literal)
                } else {
                    let literal = self.read_quoted();
                    self.construct_token(line, column, literal)
                }
            }
            '-' => {
                if self.peek_char(1) == Some('-') {
                    let literal = self.read_comment();
                    self.construct_token(line, column, literal)
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '/' => {
                if self.peek_char(1) == Some('*') {
                    let literal = self.read_multiline_comment();
                    self.construct_token(line, column, literal)
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '|' => {
                if self.peek_char(1) == Some('|') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "||".to_string())
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '<' => {
                if self.peek_char(1) == Some('<') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "<<".to_string())
                } else if self.peek_char(1) == Some('=') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "<=".to_string())
                } else if self.peek_char(1) == Some('>') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "<>".to_string())
                } else {
                    if self.tokens.last().unwrap().literal.to_uppercase() == "ARRAY"
                        || self.tokens.last().unwrap().literal.to_uppercase() == "STRUCT"
                    {
                        self.type_declaration_depth += 1;
                    }
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '>' => {
                if 0 < self.type_declaration_depth {
                    self.type_declaration_depth -= 1;
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                } else if self.peek_char(1) == Some('>') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, ">>".to_string())
                } else if self.peek_char(1) == Some('=') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, ">=".to_string())
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '=' => {
                if self.peek_char(1) == Some('>') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "=>".to_string())
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '!' => {
                if self.peek_char(1) == Some('=') {
                    self.read_char();
                    self.read_char();
                    self.construct_token(line, column, "!=".to_string())
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            // parameter
            '@' => {
                let literal = self.read_parameter();
                self.construct_token(line, column, literal)
            }
            // int64 or float64 literal
            '0'..='9' => {
                let literal = self.read_number();
                self.construct_token(line, column, literal)
            }
            // other
            _ => {
                if is_valid_1st_char_of_ident(&self.get_curr_char()) {
                    let literal = self.read_identifier();
                    self.construct_token(line, column, literal)
                } else {
                    self.read_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
        };
        Some(token)
    }
    fn read_multiline_string(&mut self) -> String {
        let first_position = self.position;
        let ch = self.get_curr_char();
        self.read_char(); // first ' -> second '
        while !(self.get_curr_char() == ch && self.peek_char(1) == ch && self.peek_char(2) == ch) {
            if self.get_curr_char() == Some('\\') {
                self.skip_escaped_char();
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
    fn read_quoted(&mut self) -> String {
        let quote = self.get_curr_char();
        let first_position = self.position;
        self.read_char();
        while self.get_curr_char() != quote {
            if self.get_curr_char() == Some('\\') {
                self.skip_escaped_char();
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
        while is_whitespace(&self.get_curr_char()) {
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
        while is_digit_or_period(&self.get_curr_char()) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_comment(&mut self) -> String {
        let first_position = self.position;
        while !is_end_of_line(&self.get_curr_char()) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_multiline_comment(&mut self) -> String {
        let first_position = self.position;
        while !(self.get_curr_char() == Some('*') && self.peek_char(1) == Some('/')) {
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

fn is_valid_1st_char_of_ident(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch == &'_',
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
