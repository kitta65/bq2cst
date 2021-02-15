#[derive(PartialEq, Debug)]
enum TokenType {
    SEMICOLON,
    SHARP,
    EOF,
    ILLEGAL,
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
    fn next_token(&mut self) -> Token {
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
            _ => Token {
                token_type: TokenType::ILLEGAL,
                literal: ch.to_string(),
            },
        };
        self.read_char();
        token
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = ";#".to_string();
        let mut l = Lexer::new(input);
        let expected_tokens: Vec<Token> = vec![
            Token{ token_type: TokenType::SEMICOLON, literal: ";".to_string() },
            Token{ token_type: TokenType::SHARP, literal: "#".to_string() },
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
}
