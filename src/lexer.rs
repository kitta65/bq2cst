#[derive(PartialEq, Debug)]
enum TokenType {
    SEMICOLON,
    SHARP,
    EOF,
    ILLEGAL,
    SELECT,
    IDENT,
    CREATE,
    FROM,
    BRANKLINE,
    INTEGER,
}

#[derive(PartialEq, Debug)]
struct Token {
    token_type: TokenType,
    literal: String,
    line: usize,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    line: usize,
}

impl Lexer {
    fn new(input: String) -> Lexer {
        let mut chars: Vec<char> = Vec::new();
        for i in input.chars() {
            chars.push(i)
        }
        let first_char = chars[0];
        Lexer {
            input: chars,
            position: 0,
            read_position: 1,
            ch: Some(first_char),
            line: 0,
        }
    }
    fn read_char(&mut self) {
        if self.input.len() <= self.read_position {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        while is_letter(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let ch = match self.ch {
            Some(ch) => ch,
            None => {
                return Token {
                    token_type: TokenType::EOF,
                    literal: "".to_string(),
                    line: self.line,
                }
            }
        };
        let token = match ch {
            ';' => Token {
                token_type: TokenType::SEMICOLON,
                literal: ch.to_string(),
                line: self.line,
            },
            '#' => Token {
                token_type: TokenType::SHARP,
                literal: ch.to_string(),
                line: self.line,
            },
            '\n' => Token {
                token_type: TokenType::BRANKLINE,
                literal: ch.to_string(),
                line: self.line,
            },
            _ => {
                if is_letter(&self.ch) {
                    let token_literal = self.read_identifier();
                    return self.lookup_keyword(token_literal); // note: the ownerwhip moves
                } else if ch.is_digit(10) {
                    Token {
                        token_type: TokenType::INTEGER,
                        literal: ch.to_string(),
                        line: self.line,
                    }
                } else {
                    Token {
                        token_type: TokenType::ILLEGAL,
                        literal: ch.to_string(),
                        line: self.line,
                    }
                }
            }
        };
        self.read_char();
        token
    }
    fn skip_whitespace(&mut self) {
        while self.is_whitespace() {
            self.read_char();
        }
    }
    fn is_whitespace(&mut self) -> bool {
        let ch = match self.ch {
            Some(ch) => ch,
            None => return false,
        };
        match ch {
            '\n' => {
                self.line += 1;
                true
            }
            _ => ch.is_whitespace(),
        }
    }
    fn peek_char(&mut self) -> Option<char> {
        if self.input.len() <= self.read_position {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }
    fn lookup_keyword(&self, keyword: String) -> Token {
        let keyword_upper = keyword.to_ascii_uppercase();
        let s = keyword_upper.as_str();
        match s {
            "SELECT" => Token {
                token_type: TokenType::SELECT,
                literal: keyword_upper,
                line: self.line,
            },
            "FROM" => Token {
                token_type: TokenType::FROM,
                literal: keyword_upper,
                line: self.line,
            },
            "CREATE" => Token {
                token_type: TokenType::CREATE,
                literal: keyword_upper,
                line: self.line,
            },
            _ => Token {
                token_type: TokenType::IDENT,
                literal: keyword,
                line: self.line,
            },
        }
    }
}

fn is_letter(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic(),
        None => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = "#standardSQL
            SELECT 1 From;"
            .to_string();
        let mut l = Lexer::new(input);
        let expected_tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::SHARP,
                literal: "#".to_string(),
                line: 0,
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "standardSQL".to_string(),
                line: 0,
            },
            Token {
                token_type: TokenType::SELECT,
                literal: "SELECT".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::INTEGER,
                literal: "1".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::FROM,
                literal: "FROM".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                literal: "".to_string(),
                line: 1,
            },
        ];
        for t in expected_tokens {
            assert_eq!(l.next_token(), t);
        }
    }

    #[test]
    fn test_surrogate() {
        let code = "𩸽";
        for c in code.chars() {
            assert_eq!(c, '𩸽'); // treated as one character
        }
    }

    #[test]
    fn test_chars2string() {
        let chars = vec!['#', ';', 'S', 'E', 'L', 'E', 'C', 'T'];
        let str: String = chars[0..2].into_iter().collect();
        assert_eq!(str, "#;".to_string());
    }

    #[test]
    fn test_is_letter() {
        assert!('a'.is_alphabetic());
        assert!('z'.is_alphabetic());
        assert!('𩸽'.is_alphabetic());
        assert!(!';'.is_alphabetic());
        assert!(';'.is_ascii());
        assert!('z'.is_ascii());
        assert!('0'.is_numeric());
        assert!('0'.is_ascii());
        assert!(!'0'.is_alphabetic());
        assert!(!'9'.is_alphabetic());
    }

    #[test]
    fn test_cr() {
        assert_eq!(
            "
",
            "\n"
        )
    }
}
