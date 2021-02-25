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
        let node = match self
            .cur_token
            .clone()
            .unwrap()
            .literal
            .to_uppercase()
            .as_str()
        {
            "SELECT" => {
                self.parse_select_statement()
            }
            _ => self.parse_select_statement(),
        };
        self.next_token();
        node
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
            cst::Children::NodeVec(self.parse_exprs(&vec!["from", ";", "limit"])),
        );
        // from
        if self.peek_token_is("FROM") {
            self.next_token(); // expr -> from
            let mut from = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // from -> table
            from.push_node_vec("tables", self.parse_tables(&vec![]));
            node.push_node("from", from);
        }
        // where
        if self.peek_token_is("WHERE") {
            self.next_token(); // expr -> where
            let mut where_ = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // limit -> expr
            where_.push_node(
                "expr",
                self.parse_expr(999, &vec!["group", "having", ";", ","]),
            );
            //self.next_token(); // parse_expr needs next_token()
            node.push_node("where", where_);
        }
        // group by
        if self.peek_token_is("GROUP") {
            self.next_token(); // expr -> group
            let mut groupby = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // group -> by
            groupby.push_node("by", cst::Node::new(self.cur_token.clone().unwrap()));
            self.next_token(); // by -> expr
            groupby.push_node_vec("columns", self.parse_exprs(&vec!["having", "limit", ";"]));
            node.push_node("groupby", groupby);
            if self.peek_token_is("HAVING") {
                self.next_token(); // expr -> having
                let mut having = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // by -> expr
                having.push_node_vec("columns", self.parse_exprs(&vec!["LIMIT", ";"]));
                //self.next_token(); // expr -> limit
                node.push_node("having", having);
            }
        }
        // having
        if self.peek_token_is("HAVING") {
            self.next_token(); // expr -> having
            let mut having = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // by -> expr
            having.push_node_vec("columns", self.parse_exprs(&vec!["GROUP", "limit", ";"]));
            node.push_node("having", having);
            if self.peek_token_is("GROUP") {
                self.next_token(); // expr -> group
                let mut groupby = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // group -> by
                groupby.push_node("by", cst::Node::new(self.cur_token.clone().unwrap()));
                self.next_token(); // by -> expr
                groupby.push_node_vec("columns", self.parse_exprs(&vec!["LIMIT", ";"]));
                //self.next_token(); // expr -> limit
                node.push_node("having", groupby);
            }
        }
        // limit
        if self.peek_token_is("LIMIT") {
            self.next_token(); // expr -> limit
            let mut limit = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // limit -> expr
            limit.push_node("expr", self.parse_expr(999, &vec![";", ","]));
            //self.next_token(); // parse_expr needs next_token()
            node.push_node("limit", limit)
        }
        // ;
        if self.peek_token_is(";") {
            self.next_token(); // expr -> ;
            node.push_node("semicolon", cst::Node::new(self.cur_token.clone().unwrap()))
        }
        // next statement
        //self.next_token(); // ; -> stmt
        node
    }
    fn cur_token_in(&self, literals: &Vec<&str>) -> bool {
        for l in literals {
            if self.cur_token_is(l) {
                return true;
            };
        }
        false
    }
    fn peek_token_in(&self, literals: &Vec<&str>) -> bool {
        for l in literals {
            if self.peek_token_is(l) {
                return true;
            };
        }
        false
    }
    fn parse_tables(&mut self, until: &Vec<&str>) -> Vec<cst::Node> {
        let mut tables: Vec<cst::Node> = Vec::new();
        while !self.cur_token_in(&vec!["where", "group", "having", "limit", ";"])
            && self.cur_token != None
        {
            tables.push(self.parse_table());
            if !self.peek_token_in(&vec!["where", "group", "having", "limit", ";"]) && self.peek_token != None {
                self.next_token();
            } else {
                return tables;
            }
        }
        tables // maybe not needed
    }
    fn parse_table(&mut self) -> cst::Node {
        // join
        let mut join = if self.cur_token_in(&vec![
            "left", "right", "cross", "inner", ",", "full", "join",
        ]) {
            if self.cur_token_in(&vec!["join", ","]) {
                let join = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // join -> table
                join
            } else {
                let mut type_ = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // type -> outer, type -> join
                if self.cur_token_is("outer") {
                    type_.push_node("outer", cst::Node::new(self.cur_token.clone().unwrap()));
                    self.next_token(); // outer -> join
                }
                let mut join = cst::Node::new(self.cur_token.clone().unwrap());
                join.push_node("type", type_);
                self.next_token(); // join -> table,
                join
            }
        } else {
            cst::Node::new_none()
        };
        // table
        let mut table = cst::Node::new(self.cur_token.clone().unwrap());
        // TODO '.' operator
        if self.peek_token_is("as") {
            self.next_token(); // `table` -> AS
            let mut as_ = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // as -> alias
            as_.push_node("alias", cst::Node::new(self.cur_token.clone().unwrap()));
            table.push_node("as", as_);
            //self.next_token(); // alias -> on, clause
        } else if !self.peek_token_in(&vec!["where", "group", "having", "limit", ";", "on"]) {
            self.next_token(); // `table` -> alias
            let mut as_ = cst::Node::new_none();
            as_.push_node("alias", cst::Node::new(self.cur_token.clone().unwrap()));
            table.push_node("as", as_);
            //self.next_token(); // alias -> on, clause
        }
        if join.token != None {
            if self.peek_token_is("on") {
                self.next_token(); // `table` -> on
                let mut on = cst::Node::new(self.cur_token.clone().unwrap());
                self.next_token(); // on -> expr
                on.push_node(
                    "expr",
                    self.parse_expr(
                        999,
                        &vec![
                            "left", "right", "cross", "inner", ",", "full", "join", "where",
                            "group", "having", ";",
                        ],
                    ),
                );
                //self.next_token(); // parse_expr needs next_token()
                join.push_node("on", on);
            } //else self.cur_token_is("using") {}
            table.push_node("join", join);
        }
        // TODO... using()
        table
    }
    fn parse_exprs(&mut self, until: &Vec<&str>) -> Vec<cst::Node> {
        let mut exprs: Vec<cst::Node> = Vec::new();
        //let token: token::Token;
        //let node: cst::Node;
        while !self.cur_token_in(until) && self.cur_token != None {
            exprs.push(self.parse_expr(999, until));
            if !self.peek_token_in(until) && self.peek_token != None {
                self.next_token();
            } else {
                return exprs;
            }
        }
        exprs // maybe not needed
    }
    fn parse_expr(&mut self, precedence: usize, until: &Vec<&str>) -> cst::Node {
        // prefix or literal
        let mut left = cst::Node::new(self.cur_token.clone().unwrap());
        match self
            .cur_token
            .clone()
            .unwrap()
            .literal
            .to_uppercase()
            .as_str()
        {
            "(" => {
                self.next_token(); // ( -> expr
                left.push_node("expr", self.parse_expr(999, until));
                self.next_token(); // expr -> )
                left.push_node("rparen", cst::Node::new(self.cur_token.clone().unwrap()));
            }
            "-" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until);
                left.push_node("right", right);
            }
            "DATE" => {
                if self.peek_token.clone().unwrap().is_string() {
                    self.next_token(); // date -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "TIMESTAMP" => {
                if self.peek_token.clone().unwrap().is_string() {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until);
                    left.push_node("right", right);
                }
            }
            "INTERVAL" => {
                self.next_token(); // interval -> expr
                let right = self.parse_expr(001, &vec!["hour", "month", "year"]);
                self.next_token(); // expr -> hour
                left.push_node("date_part", cst::Node::new(self.cur_token.clone().unwrap()));
                left.push_node("right", right);
            }
            "SELECT" => {
                println!("{:?}", self.cur_token.clone().unwrap());
                left = self.parse_select_statement();
            }
            _ => (),
        };
        while !self.peek_token_in(until) && self.peek_precedence() < precedence {
            // actually, until is not needed
            match self.peek_token.clone().unwrap().literal.to_uppercase().as_str() {
                "+" => {
                    self.next_token(); // expr -> +
                    let precedence = self.cur_precedence();
                    let mut node = cst::Node::new(self.cur_token.clone().unwrap());
                    self.next_token(); // + -> expr
                    node.push_node("left", left);
                    node.push_node("right", self.parse_expr(precedence, until));
                    left = node;
                }
                "*" => {
                    self.next_token(); // expr -> +
                    let precedence = self.cur_precedence();
                    let mut node = cst::Node::new(self.cur_token.clone().unwrap());
                    self.next_token(); // + -> expr
                    node.push_node("left", left);
                    node.push_node("right", self.parse_expr(precedence, until));
                    left = node;
                }
                "IN" => {
                    self.next_token(); // expr -> in
                    //let precedence = self.cur_precedence();
                    let mut node = cst::Node::new(self.cur_token.clone().unwrap());
                    self.next_token(); // in -> (
                    node.push_node("left", left);
                    let mut right = cst::Node::new(self.cur_token.clone().unwrap());
                    self.next_token(); // ( -> expr
                    right.push_node_vec("exprs", self.parse_exprs(&vec![")"]));
                    self.next_token(); // expr -> )
                    right.push_node("rparen", cst::Node::new(self.cur_token.clone().unwrap()));
                    node.push_node("right", right);
                    left = node;
                }
                "(" => {
                    self.next_token(); // expr -> (
                                       //let precedence = self.cur_precedence();
                    let mut node = cst::Node::new(self.cur_token.clone().unwrap());
                    self.next_token(); // ( -> args
                    node.push_node("func", left);
                    node.push_node_vec("args", self.parse_exprs(&vec![")"]));
                    self.next_token(); // expr -> )
                    node.push_node("rparen", cst::Node::new(self.cur_token.clone().unwrap()));
                    // TODO window function
                    left = node;
                }
                _ => panic!(),
            }
        }
        // alias
        if self.peek_token_is("as") && precedence == 999 {
            self.next_token(); // expr -> as
            let mut as_ = cst::Node::new(self.cur_token.clone().unwrap());
            self.next_token(); // as -> alias
            as_.push_node("alias", cst::Node::new(self.cur_token.clone().unwrap()));
            left.push_node("as", as_);
        }
        if !self.peek_token_in(&vec![
            "from", "where", "group", "having", "limit", ";", ",", ")",
        ]) && self.peek_token != None
            && precedence == 999
        {
            self.next_token(); // expr -> alias
            let mut as_ = cst::Node {
                token: None,
                children: HashMap::new(),
            };
            as_.push_node("alias", cst::Node::new(self.cur_token.clone().unwrap()));
            left.push_node("as", as_);
        }
        if self.peek_token_is(",") && precedence == 999 {
            self.next_token(); // expr -> ,
            left.children.insert(
                "comma".to_string(),
                cst::Children::Node(cst::Node {
                    token: Some(self.cur_token.clone().unwrap()),
                    children: HashMap::new(),
                }),
            );
        }
        //self.next_token(); // expr -> from, ',' -> expr
        left
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
    fn cur_precedence(&self) -> usize {
        let token = match self.cur_token.clone() {
            Some(t) => t,
            None => panic!(),
        };
        str2precedence(token.literal.as_str())
    }
    fn peek_precedence(&self) -> usize {
        let token = match self.peek_token.clone() {
            Some(t) => t,
            None => {
                return 999;
            }
        };
        str2precedence(token.literal.as_str())
    }
}

fn str2precedence(s: &str) -> usize {
    // precedenc
    // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators#arithmetic_operators
    // 001... date, timestamp (literal)
    // 005... ( (call expression)
    // 101... [], .
    // 102... +, - , ~ (unary operator)
    // 103... *, / , ||
    // 104... +, - (binary operator)
    // 105... <<, >>
    // 106... & (bit operator)
    // 107... ^ (bit operator)
    // 108... | (bit operator)
    // 109... =, <, >, (not)like, between, (not)in
    // 110... not
    // 111... and
    // 112... or
    // 999... LOWEST
    match s.to_uppercase().as_str() {
        "(" => 005,
        "-" => 104,
        "+" => 104,
        "*" => 103,
        "/" => 103,
        "||" => 103,
        "IN" => 109,
        _ => 999,
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
            select 2 two;
            select * from data1 as one inner join data2 two ON true;
            select -1, 1+1+1, date '2020-02-24', TIMESTAMP '2020-01-01', interval 9 year, if(true, 'true'), (1+1)*1, ((2)), (select info limit 1) from data where 1 in (1, 2);"
            .to_string();
        let l = lexer::Lexer::new(input);
        let mut p = Parser::new(l);
        let stmt = p.parse_code();
        let tests = vec![
            // simple select
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
            // alias
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
  self: ;",
            // implicit alias
            "\
self: select
columns:
- self: 2
  as:
    self: None
    alias:
      self: two
semicolon:
  self: ;",
            // join
            "\
self: select
columns:
- self: *
from:
  self: from
  tables:
  - self: data1
    as:
      self: as
      alias:
        self: one
  - self: data2
    as:
      self: None
      alias:
        self: two
    join:
      self: join
      on:
        self: ON
        expr:
          self: true
      type:
        self: inner
semicolon:
  self: ;",
            // parse_expr precedence
            "\
self: select
columns:
- self: -
  comma:
    self: ,
  right:
    self: 1
- self: +
  comma:
    self: ,
  left:
    self: +
    left:
      self: 1
    right:
      self: 1
  right:
    self: 1
- self: date
  comma:
    self: ,
  right:
    self: '2020-02-24'
- self: TIMESTAMP
  comma:
    self: ,
  right:
    self: '2020-01-01'
- self: interval
  comma:
    self: ,
  date_part:
    self: year
  right:
    self: 9
- self: (
  args:
  - self: true
    comma:
      self: ,
  - self: 'true'
  comma:
    self: ,
  func:
    self: if
  rparen:
    self: )
- self: *
  comma:
    self: ,
  left:
    self: (
    expr:
      self: +
      left:
        self: 1
      right:
        self: 1
    rparen:
      self: )
  right:
    self: 1
- self: (
  comma:
    self: ,
  expr:
    self: (
    expr:
      self: 2
    rparen:
      self: )
  rparen:
    self: )
- self: (
  expr:
    self: select
    columns:
    - self: info
    limit:
      self: limit
      expr:
        self: 1
  rparen:
    self: )
from:
  self: from
  tables:
  - self: data
semicolon:
  self: ;
where:
  self: where
  expr:
    self: in
    left:
      self: 1
    right:
      self: (
      exprs:
      - self: 1
        comma:
          self: ,
      - self: 2
      rparen:
        self: )",
        ];
        for i in 0..tests.len() {
            println!("{}\n", stmt[i].to_string(0, false));
            assert_eq!(stmt[i].to_string(0, false), tests[i]);
        }
    }
}
