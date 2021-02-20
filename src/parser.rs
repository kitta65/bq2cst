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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let input = "select * from data;".to_string();
        let l = lexer::Lexer::new(input);
        let p = Parser::new(l);
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
    }
}
