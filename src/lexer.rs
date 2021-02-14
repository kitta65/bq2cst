enum TokenType {
    SEMICOLON,
    SHARP,
}

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
    fn next_token(&self) {
        let token = match self.ch {
            TokenType.SEMICOLON => TOKEN { token_type: TokenType.SEMICOLON, literal: self.ch},
            TokenType.SHARP => TOKEN { token_type: TokenType.SHARP, literal: self.ch },
        };
        self.read_char();
        token
    }
}

#[test]
fn test_lexer() {
    let input = ";#".to_string();
    let l = Lexer::new(input);
    let tokens = l.next_token();
    assert_eq!(tokens[0].literal, "SELECT".to_string());
}

#[test]
fn test_surrogate() {
    let code = "𩸽";
    for c in code.chars() {
        assert_eq!(c, '𩸽'); // treated as one character
    }
}
