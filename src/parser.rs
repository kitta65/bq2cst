use crate::lexer;
use crate::token;

struct Parser {
    lexer: lexer::Lexer,
    cur_token: Option<token::Token>,
    peek_token: Option<token::Token>,
    leading_comments: Vec<token::Token>,
}

impl Parser {
    fn new(mut l: lexer::Lexer) -> Parser {
        let first_token = l.next_token();
        let second_token = l.next_token();
        Parser {
            lexer: l,
            cur_token: first_token,
            peek_token: second_token,
            leading_comments: Vec::new(),
        }
    }
    fn next_token(&mut self) {
        self.cur_token = match self.peek_token.clone() {
            Some(token) => Some(token),
            None => None,
        };
        self.peek_token = self.lexer.next_token();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = "select *;".to_string();
        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        assert_eq!(p.cur_token, Some(token::Token {
            line: 0,
            column: 0,
            literal: "select".to_string(),
        }));
        assert_eq!(p.peek_token, Some(token::Token {
            line: 0,
            column: 7,
            literal: "*".to_string(),
        }));
        p.next_token();
        assert_eq!(p.cur_token, Some(token::Token {
            line: 0,
            column: 7,
            literal: "*".to_string(),
        }));
        assert_eq!(p.peek_token, Some(token::Token {
            line: 0,
            column: 8,
            literal: ";".to_string(),
        }));
        p.next_token();
        assert_eq!(p.cur_token, Some(token::Token {
            line: 0,
            column: 8,
            literal: ";".to_string(),
        }));
        assert_eq!(p.peek_token, None);
    }
}
