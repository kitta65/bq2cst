use crate::cst;
use crate::lexer;
use crate::token;
use std::collections::HashMap;

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
        // i dont wanna use clone but i dont know how to avoid
        self.cur_token = match self.peek_token.clone() {
            Some(token) => Some(token),
            None => None,
        };
        self.peek_token = self.lexer.next_token();
    }
    fn parse_code(&mut self, code: String) -> Vec<cst::Node> {
        let mut code: Vec<cst::Node> = Vec::new();
        while self.cur_token != None {
            let stmt = self.parse_statement();
            code.push(stmt);
            self.next_token();
        }
        Vec::new()
    }
    fn parse_statement(&mut self) -> cst::Node {
        cst::Node {
            token: token::Token {
                line: 0,
                column: 0,
                literal: "dummy".to_string(),
            },
            children: HashMap::new(),
        }
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
        assert_eq!(
            p.cur_token,
            Some(token::Token {
                line: 0,
                column: 0,
                literal: "select".to_string(),
            })
        );
        assert_eq!(
            p.peek_token,
            Some(token::Token {
                line: 0,
                column: 7,
                literal: "*".to_string(),
            })
        );
        p.next_token();
        assert_eq!(
            p.cur_token,
            Some(token::Token {
                line: 0,
                column: 7,
                literal: "*".to_string(),
            })
        );
        assert_eq!(
            p.peek_token,
            Some(token::Token {
                line: 0,
                column: 8,
                literal: ";".to_string(),
            })
        );
        p.next_token();
        assert_eq!(
            p.cur_token,
            Some(token::Token {
                line: 0,
                column: 8,
                literal: ";".to_string(),
            })
        );
        assert_eq!(p.peek_token, None);
        p.next_token();
        assert_eq!(p.cur_token, None);
        assert_eq!(p.peek_token, None);
    }
}
