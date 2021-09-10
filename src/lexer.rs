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
    // ----- pub -----
    pub fn new(input: String) -> Lexer {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            input: chars,
            position: 0,
            line: 1,
            column: 1,
            type_declaration_depth: 0,
            tokens: Vec::new(),
        }
    }
    pub fn tokenize_code(&mut self) -> &Vec<Token> {
        let mut token = self.next_token();
        while !token.is_none() {
            token = self.next_token();
        }
        self.tokens.push(Token::eof());
        &self.tokens
    }
    // ----- core -----
    fn construct_token(&mut self, line: usize, column: usize, literal: String) -> &Token {
        let token = Token::new(line, column, literal);
        self.tokens.push(token);
        &self.tokens.last().unwrap()
    }
    fn get_char(&self, offset: usize) -> Option<char> {
        if self.position + offset < self.input.len() {
            return Some(self.input[self.position + offset]);
        } else {
            return None; // EOF
        }
    }
    fn next_char(&mut self) {
        if self.position < self.input.len() {
            if self.input[self.position] == '\n' {
                self.column = 1;
                self.line += 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        } else {
            panic!("`next_char()` is called at EOF");
        }
    }
    fn next_token(&mut self) -> Option<&Token> {
        self.skip_whitespace();
        let ch = match self.get_char(0) {
            Some(ch) => ch,
            None => {
                return None; // EOF
            }
        };
        let line = self.line;
        let column = self.column;
        let token = match ch {
            '.' => match self.get_char(1) {
                Some('0'..='9') => {
                    let literal = self.read_number();
                    self.construct_token(line, column, literal)
                }
                _ => {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            },
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
                if self.get_char(1) == Some('"') && self.get_char(2) == Some('"') {
                    let literal = self.read_multiline_string();
                    self.construct_token(line, column, literal)
                } else {
                    let literal = self.read_quoted();
                    self.construct_token(line, column, literal)
                }
            }
            '\'' => {
                if self.get_char(1) == Some('\'') && self.get_char(2) == Some('\'') {
                    let literal = self.read_multiline_string();
                    self.construct_token(line, column, literal)
                } else {
                    let literal = self.read_quoted();
                    self.construct_token(line, column, literal)
                }
            }
            '-' => {
                if self.get_char(1) == Some('-') {
                    let literal = self.read_comment();
                    self.construct_token(line, column, literal)
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '/' => {
                if self.get_char(1) == Some('*') {
                    let literal = self.read_multiline_comment();
                    self.construct_token(line, column, literal)
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '|' => {
                if self.get_char(1) == Some('|') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "||".to_string())
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '<' => {
                if self.get_char(1) == Some('<') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "<<".to_string())
                } else if self.get_char(1) == Some('=') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "<=".to_string())
                } else if self.get_char(1) == Some('>') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "<>".to_string())
                } else {
                    if self.tokens.last().unwrap().literal.to_uppercase() == "ARRAY"
                        || self.tokens.last().unwrap().literal.to_uppercase() == "STRUCT"
                    {
                        self.type_declaration_depth += 1;
                    }
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '>' => {
                if 0 < self.type_declaration_depth {
                    self.type_declaration_depth -= 1;
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                } else if self.get_char(1) == Some('>') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, ">>".to_string())
                } else if self.get_char(1) == Some('=') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, ">=".to_string())
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '=' => {
                if self.get_char(1) == Some('>') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "=>".to_string())
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
            '!' => {
                if self.get_char(1) == Some('=') {
                    self.next_char();
                    self.next_char();
                    self.construct_token(line, column, "!=".to_string())
                } else {
                    self.next_char();
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
                if is_valid_1st_char_of_ident(&Some(ch)) {
                    let literal = self.read_identifier();
                    self.construct_token(line, column, literal)
                } else {
                    self.next_char();
                    self.construct_token(line, column, ch.to_string())
                }
            }
        };
        Some(token)
    }
    // ----- read -----
    fn read_comment(&mut self) -> String {
        let first_position = self.position;
        while !is_end_of_line(&self.get_char(0)) {
            self.next_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect::<String>()
            .trim_end()
            .to_string()
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        let first_char = self.get_char(0);
        if !is_valid_1st_char_of_ident(&first_char) {
            panic!(
                "Calling `read_identifier()` is not allowed here: {:?}",
                first_char
            );
        }
        self.next_char();
        while is_valid_char_of_ident(&self.get_char(0)) {
            self.next_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_multiline_comment(&mut self) -> String {
        let first_position = self.position;
        while !(self.get_char(0) == Some('*') && self.get_char(1) == Some('/')) {
            self.next_char();
        }
        self.next_char(); // * -> /
        self.next_char(); // / -> next_char
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_multiline_string(&mut self) -> String {
        // NOTE '''abc''' is OK. ''''abc'''' should throw an error.
        let first_position = self.position;
        let ch = self.get_char(0);
        self.next_char(); // first ' -> second '
        while !(self.get_char(0) == ch && self.get_char(1) == ch && self.get_char(2) == ch) {
            if self.get_char(0) == Some('\\') {
                self.skip_escaped_char();
            } else {
                self.next_char();
            }
        }
        self.next_char(); // first ' -> secont '
        self.next_char(); // second ' -> third '
        self.next_char(); // third ' ->  next_ch
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_number(&mut self) -> String {
        let first_position = self.position;
        while is_digit(&self.get_char(0)) {
            self.next_char();
        } // 9 -> .
        if self.get_char(0) == Some('.') {
            self.next_char();
            while is_digit(&self.get_char(0)) {
                self.next_char();
            }
        }
        if let Some('E') | Some('e') = self.get_char(0) {
            self.next_char(); // e -> 9, +, -
            if let Some('+') | Some('-') = self.get_char(0) {
                self.next_char(); // +, - -> 9
            }
            while is_digit(&self.get_char(0)) {
                self.next_char();
            }
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_parameter(&mut self) -> String {
        let first_position = self.position;
        while self.get_char(0) == Some('@') {
            self.next_char();
        }
        if self.get_char(0) == Some('`') {
            self.read_quoted();
        } else {
            self.read_identifier();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_quoted(&mut self) -> String {
        let quote = self.get_char(0);
        let first_position = self.position;
        self.next_char();
        while self.get_char(0) != quote {
            if self.get_char(0) == Some('\\') {
                self.skip_escaped_char();
            } else {
                self.next_char();
            }
        }
        self.next_char(); // ' -> next_ch
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    // ----- skip -----
    fn skip_escaped_char(&mut self) {
        self.next_char(); // '\\' -> ?
        match self.get_char(0) {
            // https://cloud.google.com/bigquery/docs/reference/standard-sql/lexical#literals
            Some('x') => {
                for _ in 0..2 {
                    self.next_char();
                }
            }
            Some('u') => {
                for _ in 0..4 {
                    self.next_char();
                }
            }
            Some('U') => {
                for _ in 0..8 {
                    self.next_char();
                }
            }
            Some('0') => {
                for _ in 0..3 {
                    self.next_char();
                }
            }
            Some('1'..='7') => {
                for _ in 0..3 {
                    self.next_char();
                }
            }
            Some(_) => (), // \n, \t, ...
            None => panic!("reached EOF in `skip_escaped_char()`"),
        }
        self.next_char();
    }
    fn skip_whitespace(&mut self) {
        while is_whitespace(&self.get_char(0)) {
            self.next_char();
        }
    }
}

fn is_digit(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_digit(10),
        None => false,
    }
}

fn is_end_of_line(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch == &'\n',
        None => true, // EOF is treated as end of line
    }
}

fn is_valid_1st_char_of_ident(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch == &'_',
        None => false,
    }
}

fn is_valid_char_of_ident(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch.is_digit(10) || ch == &'_',
        None => false,
    }
}

fn is_whitespace(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_whitespace(), // specified in the Unicode Character Database
        None => false,                  // EOF is treated as end of line
    }
}
