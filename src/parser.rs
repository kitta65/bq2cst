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
            //self.next_token();
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
            "SELECT" => {
                println!("found select!");
                self.parse_select_statement()
            }
            _ => self.parse_select_statement(),
        }
    }
    fn parse_select_statement(&mut self) -> cst::Node {
        let mut node = cst::Node {
            token: Some(self.cur_token.clone().unwrap()),
            children: HashMap::new(),
        };
        self.next_token(); // select -> [distinct]

        // distinct
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
                        token: Some(self.cur_token.clone().unwrap()),
                        children: HashMap::new(),
                    }),
                );
                self.next_token(); // distinct -> columns
            }
            _ => (),
        };
        // columns
        node.children.insert(
            "columns".to_string(),
            cst::Children::NodeVec(self.parse_exprs(&vec!["from"])),
        );
        // from
        if self.cur_token_is("FROM") {
            let mut from = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // from -> table
            from.push_node_vec(
                "tables",
                self.parse_exprs(&vec!["where", "group", "having", "limit", ";"]), // TODO... define parse_tables
            );
            node.push_node("from", from);
        }
        // where
        if self.cur_token_is("WHERE") {
            let mut where_ = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // limit -> expr
            where_.push_node("expr", self.parse_expr());
            node.push_node("where", where_)
        }
        // group by
        if self.cur_token_is("GROUP") {
            let mut groupby = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // group -> by
            groupby.push_node("by", cst::Node::new(self.cur_token.clone().unwrap()));
            self.next_token(); // by -> expr
            groupby.push_node_vec("columns", self.parse_exprs(&vec!["having", "limit", ";"]));
            node.push_node("groupby", groupby);
            if self.cur_token_is("HAVING") {
                let mut having = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // by -> expr
                having.push_node_vec("columns", self.parse_exprs(&vec!["LIMIT", ";"]));
                node.push_node("having", having);
            }
        }
        // having
        if self.cur_token_is("HAVING") {
            let mut having = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // by -> expr
            having.push_node_vec("columns", self.parse_exprs(&vec!["GROUP", "limit", ";"]));
            node.push_node("having", having);
            if self.cur_token_is("GROUP") {
                let mut groupby = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // group -> by
                groupby.push_node("by", cst::Node::new(self.cur_token.clone().unwrap()));
                self.next_token(); // by -> expr
                groupby.push_node_vec("columns", self.parse_exprs(&vec!["LIMIT", ";"]));
                node.push_node("having", groupby);
            }
        }
        // limit
        if self.cur_token_is("LIMIT") {
            let mut limit = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // limit -> expr
            limit.push_node("expr", self.parse_expr());
            node.push_node("limit", limit)
        }
        // ;
        if self.cur_token_is(";") {
            node.push_node("semicolon", cst::Node::new(self.cur_token.clone().unwrap()))
        }
        // next statement
        self.next_token(); // ; -> stmt
        node
    }
    fn parse_exprs(&mut self, until: &Vec<&str>) -> Vec<cst::Node> {
        // TODO... precedence
        let mut exprs: Vec<cst::Node> = Vec::new();
        //let token: token::Token;
        //let node: cst::Node;
        while !self.cur_token_in(until) && self.cur_token != None {
            exprs.push(self.parse_expr());
        }
        exprs
    }
    fn cur_token_in(&self, literals: &Vec<&str>) -> bool {
        for l in literals {
            if self.cur_token_is(l) {
                return true;
            };
        }
        false
    }
    //fn peek_token_in(&self, literals: &Vec<&str>) -> bool {
    //    for l in literals {
    //        if self.peek_token_is(l) {
    //            return true;
    //        };
    //    }
    //    false
    //}
    fn parse_expr(&mut self) -> cst::Node {
        let mut left_expr: cst::Node;
        let cur_token = self.cur_token.clone().unwrap();
        if cur_token.is_prefix() {
            left_expr = cst::Node {
                token: Some(cur_token.clone()),
                children: HashMap::new(),
            };
            self.next_token(); // - or ! -> expr
            left_expr
                .children
                .insert("right".to_string(), cst::Children::Node(self.parse_expr()));
        } else {
            left_expr = cst::Node {
                token: Some(cur_token.clone()),
                children: HashMap::new(),
            };
        }
        if self.peek_token_is("as") {
            self.next_token(); // expr -> as
            let mut as_ = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // as -> alias
            as_.push_node("alias", cst::Node::new(self.cur_token.clone().unwrap()));
            left_expr.push_node("as", as_);
        }
        if self.peek_token_is(",") {
            self.next_token(); // expr -> ,
            left_expr.children.insert(
                "comma".to_string(),
                cst::Children::Node(cst::Node {
                    token: Some(self.cur_token.clone().unwrap()),
                    children: HashMap::new(),
                }),
            );
        //} else if !self.peek_token_in(&vec!["from", "where", "group", "having", ";"]) && self.peek_token != None {
        //    self.next_token(); // expr -> alias
        //    let mut as_ = cst::Node::new()
        //    left_expr.push_node()
        }
        // TODO... as
        self.next_token(); // expr -> from, ',' -> expr
        left_expr
    }
    fn peek_token_is(&self, s: &str) -> bool {
        match self.peek_token.clone() {
            Some(t) => t.literal.to_uppercase() == s.to_uppercase(),
            None => false,
        }
    }
    fn cur_token_is(&self, s: &str) -> bool {
        match self.cur_token.clone() {
            Some(t) => t.literal.to_uppercase() == s.to_uppercase(),
            None => false,
        }
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
        let input = "\
            SELECT 'aaa', 123 FROM data where true group by 1 HAVING true limit 100;
            select 1 as num from data;
            select 2 as two"
            .to_string();
        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        let stmt = p.parse_code();
        let tests = vec![
            "\
self: SELECT
columns:
- self: 'aaa'
  comma:
    self: ,
- self: 123
from:
  self: FROM
  tables:
  - self: data
groupby:
  self: group
  by:
    self: by
  columns:
  - self: 1
having:
  self: HAVING
  columns:
  - self: true
limit:
  self: limit
  expr:
    self: 100
semicolon:
  self: ;
where:
  self: where
  expr:
    self: true",
            "\
self: select
columns:
- self: 1
  as:
    self: as
    alias:
      self: num
from:
  self: from
  tables:
  - self: data
semicolon:
  self: ;","\
self: select
columns:
- self: 2
  as:
    self: as
    alias:
      self: two"
        ];
        for i in 0..tests.len() {
            assert_eq!(stmt[i].to_string(0, false), tests[i])
        }
        //test_parse_select_statement(&stmt[0], false, vec!["'aaa'".to_string(), "123".to_string()])
    }
}
