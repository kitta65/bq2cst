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
}

#[derive(PartialEq, Debug)]
struct Token {
    token_type: TokenType,
    literal: String,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
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
                }
            }
        };
        let token = match ch {
            ';' => Token {
                token_type: TokenType::SEMICOLON,
                literal: ch.to_string(),
            },
            '#' => Token {
                token_type: TokenType::SHARP,
                literal: ch.to_string(),
            },
            _ => {
                if is_letter(&self.ch) {
                    let token_literal = self.read_identifier();
                    return lookup_keyword(token_literal);
                } else {
                    Token {
                        token_type: TokenType::ILLEGAL,
                        literal: ch.to_string(),
                    }
                }
            }
        };
        self.read_char();
        token
    }
    fn skip_whitespace(&mut self) {
        while is_whitespace(&self.ch) {
            self.read_char();
        }
    }
}

fn is_letter(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic(),
        None => false,
    }
}

fn is_whitespace(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_whitespace(),
        None => false,
    }
}

fn lookup_keyword(keyword: String) -> Token {
    let keyword_upper = keyword.to_ascii_uppercase();
    let s = keyword_upper.as_str();
    match s {
        "SELECT" => Token {
            token_type: TokenType::SELECT,
            literal: keyword_upper,
        },
        "FROM" => Token {
            token_type: TokenType::FROM,
            literal: keyword_upper,
        },
        "CREATE" => Token {
            token_type: TokenType::CREATE,
            literal: keyword_upper,
        },
        _ => Token {
            token_type: TokenType::IDENT,
            literal: keyword,
        },
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = ";#SELECT From".to_string();
        let mut l = Lexer::new(input);
        let expected_tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
            },
            Token {
                token_type: TokenType::SHARP,
                literal: "#".to_string(),
            },
            Token {
                token_type: TokenType::SELECT,
                literal: "SELECT".to_string(),
            },
            Token {
                token_type: TokenType::FROM,
                literal: "FROM".to_string(),
            },
            Token {
                token_type: TokenType::EOF,
                literal: "".to_string(),
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
}
