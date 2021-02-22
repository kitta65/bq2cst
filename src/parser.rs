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
    fn parse_code(&mut self) -> Vec<cst::Node> {
        let mut code: Vec<cst::Node> = Vec::new();
        while self.cur_token != None {
            let stmt = self.parse_statement();
            code.push(stmt);
            self.next_token();
        }
        code
    }
    fn parse_statement(&mut self) -> cst::Node {
        // i dont wanna use clone but i dont know how to avoid
        match self
            .cur_token
            .clone()
            .unwrap()
            .literal
            .to_uppercase()
            .as_str()
        {
            "SELECT" => self.parse_select_statement(),
            _ => self.parse_select_statement(),
        }
    }
    fn parse_select_statement(&mut self) -> cst::Node {
        let mut node = cst::Node {
            token: self.cur_token.clone().unwrap(),
            children: HashMap::new(),
        };
        self.next_token(); // select -> [distinct]
        match self
            .cur_token
            .clone()
            .unwrap()
            .literal
            .to_uppercase()
            .as_str()
        {
            "DISTINCT" => {
                node.children.insert(
                    "DISTINCT".to_string(),
                    cst::Children::Node(cst::Node {
                        token: self.cur_token.clone().unwrap(),
                        children: HashMap::new(),
                    }),
                );
                self.next_token(); // distinct -> columns
            }
            _ => (),
        };
        node.children.insert(
            "columns".to_string(),
            cst::Children::NodeVec(self.parse_exprs("FROM".to_string())),
        );
        // for the time being
        self.next_token(); // [,] -> from
        self.next_token(); // from -> ;
        node
    }
    fn parse_exprs(&mut self, until: String) -> Vec<cst::Node> {
        // TODO... precedence
        let mut exprs: Vec<cst::Node> = Vec::new();
        //let token: token::Token;
        //let node: cst::Node;
        while self.cur_token.clone().unwrap().literal.to_uppercase() != until {
            exprs.push(self.parse_expr());
        }
        exprs
    }
    fn parse_expr(&mut self) -> cst::Node {
        let mut left_expr: cst::Node;
        let cur_token = self.cur_token.clone().unwrap();
        if cur_token.is_prefix() {
            left_expr = cst::Node {
                token: cur_token.clone(),
                children: HashMap::new(),
            };
            self.next_token(); // - or ! -> expr
            left_expr
                .children
                .insert("right".to_string(), cst::Children::Node(self.parse_expr()));
        } else {
            left_expr = cst::Node {
                token: cur_token.clone(),
                children: HashMap::new(),
            };
        }
        if self.peek_token.clone().unwrap().literal == ",".to_string() {
            self.next_token(); // expr -> ,
            left_expr.children.insert(
                "comma".to_string(),
                cst::Children::Node(cst::Node {
                    token: self.cur_token.clone().unwrap(),
                    children: HashMap::new(),
                }),
            );
        }
        self.next_token(); // expr -> from, ',' -> expr
        left_expr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use serde_json;
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
    #[test]
    fn test_parse_exprs() {
        let input = "SELECT 'aaa', 123 FROM;".to_string();
        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        let stmt = p.parse_code();
        let tests = vec!["\
self: SELECT
columns:
- self: 'aaa'
  comma:
    self: ,
- self: 123"];
        for i in 0..tests.len() {
            assert_eq!(stmt[i].to_string(0, false), tests[i])
        }
        test_parse_select_statement(&stmt[0], false, vec!["'aaa'".to_string(), "123".to_string()])
    }
    fn test_parse_select_statement(stmt: &cst::Node, distinct: bool, columns: Vec<String>) {
        if distinct {
            assert!(stmt.children.contains_key(&"DISTINCT".to_string()));
        } else {
            assert!(!stmt.children.contains_key(&"DISTINCT".to_string()));
        }
        assert!(stmt.children.contains_key(&"columns".to_string()));
        let nodes = match stmt.children.get(&"columns".to_string()).unwrap() {
            cst::Children::Node(node) => panic!(),
            cst::Children::NodeVec(nodes) => nodes,
        };
        for i in 0..nodes.len() {
            assert_eq!(nodes[i].token.literal, columns[i]);
        }
    }
}
