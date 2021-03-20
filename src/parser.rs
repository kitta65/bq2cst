use crate::cst;
use crate::lexer;
use crate::token;
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<token::Token>,
    position: usize,
    leading_comment_indices: Vec<usize>,
    following_comment_indices: Vec<usize>,
}

impl Parser {
    pub fn new(mut l: lexer::Lexer) -> Parser {
        let mut tokens = Vec::new();
        let mut token = l.next_token();
        while !token.is_none() {
            tokens.push(token.unwrap());
            token = l.next_token();
        }
        let mut p = Parser {
            tokens,
            position: 0,
            leading_comment_indices: Vec::new(),
            following_comment_indices: Vec::new(),
        };
        while p.tokens[p.position].is_comment() {
            p.leading_comment_indices.push(p.position);
            p.position += 1;
        }
        let mut following_comment_idx = p.position + 1;
        while p.tokens[following_comment_idx].is_comment() {
            p.following_comment_indices.push(following_comment_idx);
            following_comment_idx += 1;
        }
        p
    }
    fn get_offset_index(&self, offset: usize) -> usize {
        if offset == 0 {
            return self.position;
        }
        let mut cnt = 0;
        let mut idx = self.position + 1;
        while cnt < offset && idx < self.tokens.len() {
            while self.tokens[idx].is_comment() {
                idx += 1;
            }
            cnt += 1;
            if cnt < offset {
                idx += 1;
            }
        }
        idx
    }
    fn next_token(&mut self) {
        // leading comments
        self.leading_comment_indices = Vec::new();
        let idx = self.get_offset_index(1);
        let from_idx = match self.following_comment_indices.iter().rev().next() {
            Some(n) => *n,
            None => self.position,
        };
        for i in from_idx + 1..idx {
            self.leading_comment_indices.push(i);
        }
        self.position = idx;
        // following comments
        self.following_comment_indices = Vec::new();
        let mut following_comment_idx = self.position + 1;
        while following_comment_idx < self.tokens.len()
            && self.tokens[following_comment_idx].is_comment()
            && self.get_token(0).line == self.tokens[following_comment_idx].line
        {
            self.following_comment_indices.push(following_comment_idx);
            following_comment_idx += 1;
        }
    }
    fn get_token(&self, offset: usize) -> token::Token {
        let idx = self.get_offset_index(offset);
        if idx <= self.tokens.len() - 1 {
            return self.tokens[idx].clone();
        }
        token::Token::new(usize::MAX, usize::MAX, "") // eof token
    }
    fn is_eof(&self, offset: usize) -> bool {
        let idx = self.get_offset_index(offset);
        self.tokens.len() <= idx
    }
    pub fn parse_code(&mut self) -> Vec<cst::Node> {
        let mut code: Vec<cst::Node> = Vec::new();
        while !self.is_eof(0) {
            let stmt = self.parse_statement();
            //println!("{}\n", stmt.to_string(0, false));
            code.push(stmt);
            self.next_token();
        }
        code
    }
    fn construct_node(&self) -> cst::Node {
        let mut node = cst::Node::new(self.get_token(0).clone());
        node.push_node("self", cst::Node::new(self.get_token(0).clone()));
        // leading comments
        let mut leading_comment_nodes = Vec::new();
        for idx in &self.leading_comment_indices {
            leading_comment_nodes.push(cst::Node::new(self.tokens[*idx].clone()))
        }
        if 0 < leading_comment_nodes.len() {
            node.push_node_vec("leading_comments", leading_comment_nodes);
        }
        // following comments
        let mut following_comment_nodes = Vec::new();
        for idx in &self.following_comment_indices {
            following_comment_nodes.push(cst::Node::new(self.tokens[*idx].clone()))
        }
        if 0 < following_comment_nodes.len() {
            node.push_node_vec("following_comments", following_comment_nodes);
        }
        node
    }
    fn parse_statement(&mut self) -> cst::Node {
        let node = match self.get_token(0).literal.to_uppercase().as_str() {
            "WITH" => self.parse_select_statement(true),
            "SELECT" => self.parse_select_statement(true),
            "CREATE" => {
                let mut target = self.get_token(1).literal.to_uppercase();
                if target == "OR" {
                    target = self.get_token(3).literal.to_uppercase();
                }
                match target.as_str() {
                    "FUNCTION" => self.parse_create_function_statement(),
                    "TEMP" => self.parse_create_function_statement(),
                    "TEMPORARY" => self.parse_create_function_statement(),
                    "TABLE" => panic!(),
                    _ => panic!(),
                }
            }
            "(" => self.parse_select_statement(true),
            _ => {
                println!("{:?}", self.get_token(0));
                panic!();
            }
        };
        node
    }
    fn parse_create_function_statement(&mut self) -> cst::Node {
        let mut node = self.construct_node();
        if self.get_token(1).literal.to_uppercase().as_str() == "OR" {
            let mut or_replace = Vec::new();
            self.next_token(); // create -> or
            or_replace.push(self.construct_node());
            self.next_token(); // or -> replace
            or_replace.push(self.construct_node());
            node.push_node_vec("or_replace", or_replace);
        }
        if self.peek_token_in(&vec!["temporary", "temp"]) {
            self.next_token(); // -> temp
            node.push_node("temp", self.construct_node());
        }
        self.next_token(); // -> function
        node.push_node("what", self.construct_node());
        if self.peek_token_in(&vec!["if"]) {
            let mut if_not_exists = Vec::new();
            self.next_token(); // function -> if
            if_not_exists.push(self.construct_node());
            self.next_token(); // if -> not
            if_not_exists.push(self.construct_node());
            self.next_token(); // not -> exists
            if_not_exists.push(self.construct_node());
            node.push_node_vec("if_not_exists", if_not_exists);
        }
        self.next_token(); // -> ident
        node.push_node("ident", self.parse_identifier());
        self.next_token(); // ident -> (
        let mut group = self.construct_node();
        let mut args = Vec::new();
        while !self.peek_token_is(")") {
            self.next_token(); // ( -> arg, ',' -> arg
            let mut arg = self.construct_node();
            self.next_token(); // arg -> type
            arg.push_node("type", self.parse_type());
            if self.peek_token_is(",") {
                self.next_token(); // type -> ,
                arg.push_node("comma", self.construct_node());
            }
            args.push(arg);
        }
        if args.len() > 0 {
            group.push_node_vec("args", args);
        }
        self.next_token(); // type -> )
        group.push_node("rparen", self.construct_node());
        node.push_node("group", group);
        if self.peek_token_is("return") {
            self.next_token(); // ) -> return
            let mut return_ = self.construct_node();
            self.next_token(); // return -> type
            return_.push_node("type", self.parse_type());
            node.push_node("return", return_);
        }
        if self.peek_token_is("as") {
            self.next_token(); // -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> (
            let mut group = self.construct_node();
            self.next_token(); // ( -> expr
            group.push_node("expr", self.parse_expr(999, &vec![")"], false));
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node());
            as_.push_node("group", group);
            node.push_node("as", as_);
        } else {
            if self.peek_token_in(&vec!["deterministic", "not"]) {
                self.next_token(); // type -> determinism
                let mut determinism = self.construct_node();
                if self.get_token(0).literal.to_uppercase().as_str() == "NOT" {
                    self.next_token(); // not -> deterministic
                    determinism.push_node("right", self.construct_node());
                }
                node.push_node("determinism", determinism);
            }
            self.next_token(); // determinism -> language, type -> language
            let mut language = self.construct_node();
            self.next_token(); // language -> js
            language.push_node("language", self.construct_node());
            node.push_node("language", language);
            if self.peek_token_is("options") {
                self.next_token(); // js -> options
                let mut options = self.construct_node();
                self.next_token(); // options -> (
                let mut group = self.construct_node();
                if !self.peek_token_is(")") {
                    self.next_token(); // ( -> expr
                    group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                }
                self.next_token(); // expr -> )
                group.push_node("rparen", self.construct_node());
                options.push_node("group", group);
                node.push_node("options", options);
            }
            self.next_token(); // js -> as, ) -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> javascript_code
            println!("{:?}", self.get_token(0));
            as_.push_node("expr", self.construct_node());
            node.push_node("as", as_);
        }
        if self.peek_token_is(";") {
            self.next_token(); // ) -> ;
            node.push_node("semicolon", self.construct_node());
        }
        node
    }
    fn parse_identifier(&mut self) -> cst::Node {
        let mut left = self.construct_node();
        while self.peek_token_is(".") {
            self.next_token(); // ident -> .
            let mut operator = self.construct_node();
            operator.push_node("left", left);
            self.construct_node(); // . -> ident
            operator.push_node("right", self.construct_node());
            left = operator;
        }
        left
    }
    fn parse_select_statement(&mut self, root: bool) -> cst::Node {
        if self.get_token(0).literal.as_str() == "(" {
            let mut node = self.construct_node();
            self.next_token(); // ( -> select
            node.push_node("stmt", self.parse_select_statement(true));
            self.next_token(); // stmt -> )
            node.push_node("rparen", self.construct_node());
            while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
                self.next_token(); // stmt -> union
                let mut operator = self.construct_node();
                self.next_token(); // union -> distinct
                operator.push_node("distinct", self.construct_node());
                operator.push_node("left", node);
                self.next_token(); // distinct -> stmt
                operator.push_node("right", self.parse_select_statement(false));
                node = operator;
            }
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node())
            }
            return node;
        }
        if self.get_token(0).literal.to_uppercase().as_str() == "WITH" {
            let mut with = self.construct_node();
            let mut queries = Vec::new();
            while self.get_token(1).literal.to_uppercase().as_str() != "SELECT" {
                self.next_token(); // with -> ident, ) -> ident
                let mut query = self.construct_node();
                self.next_token(); // ident -> as
                query.push_node("as", self.construct_node());
                self.next_token(); // as -> (
                query.push_node("stmt", self.parse_select_statement(true));
                if self.get_token(1).literal.as_str() == "," {
                    self.next_token(); // ) -> ,
                    query.push_node("comma", self.construct_node());
                }
                queries.push(query);
            }
            with.push_node_vec("queries", queries);
            self.next_token(); // ) -> select
            let mut node = self.parse_select_statement(true);
            node.push_node("with", with);
            return node;
        }
        let mut node = self.construct_node(); // select

        // as struct, as value
        if self.get_token(1).literal.to_uppercase().as_str() == "AS" {
            self.next_token(); // select -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> struct, value
            as_.push_node("struct_value", self.construct_node());
            node.push_node("as", as_);
        }

        // distinct
        if self.peek_token_in(&vec!["all", "distinct"]) {
            self.next_token(); // select -> all, distinct
            node.push_node("distinct", self.construct_node());
        }
        self.next_token(); // -> expr

        // columns
        node.children.insert(
            "exprs".to_string(),
            cst::Children::NodeVec(self.parse_exprs(
                &vec!["from", ";", "limit", ")", "union", "intersect", "except"],
                true,
            )),
        );
        // from
        if self.peek_token_is("FROM") {
            self.next_token(); // expr -> from
            let mut from = self.construct_node();
            self.next_token(); // from -> table
            from.push_node("expr", self.parse_table(true));
            node.push_node("from", from);
        }
        // where
        if self.peek_token_is("WHERE") {
            self.next_token(); // expr -> where
            let mut where_ = self.construct_node();
            self.next_token(); // limit -> expr
            where_.push_node(
                "expr",
                self.parse_expr(999, &vec!["group", "having", ";", ","], false),
            );
            //self.next_token(); // parse_expr needs next_token()
            node.push_node("where", where_);
        }
        // group by
        if self.peek_token_is("GROUP") {
            self.next_token(); // expr -> group
            let mut groupby = self.construct_node();
            self.next_token(); // group -> by
            groupby.push_node("by", self.construct_node());
            self.next_token(); // by -> expr
            groupby.push_node_vec(
                "exprs",
                self.parse_exprs(&vec!["having", "limit", ";", "order"], false),
            );
            node.push_node("groupby", groupby);
        }
        // having
        if self.peek_token_is("HAVING") {
            self.next_token(); // expr -> having
            let mut having = self.construct_node();
            self.next_token(); // by -> expr
            having.push_node(
                "expr",
                self.parse_expr(999, &vec!["LIMIT", ";", "order"], false),
            );
            //self.next_token(); // expr -> limit
            node.push_node("having", having);
        }
        // oeder by
        if self.peek_token_is("order") {
            self.next_token(); // expr -> order
            let mut order = self.construct_node();
            self.next_token(); // order -> by
            order.push_node("by", self.construct_node());
            self.next_token(); // by -> expr
            order.push_node_vec("exprs", self.parse_exprs(&vec!["limit", ",", ";"], false));
            node.push_node("orderby", order);
        }
        // limit
        if self.peek_token_is("LIMIT") {
            self.next_token(); // expr -> limit
            let mut limit = self.construct_node();
            self.next_token(); // limit -> expr
            limit.push_node(
                "expr",
                self.parse_expr(999, &vec![";", ",", "offset"], false),
            );
            if self.get_token(1).literal.to_uppercase().as_str() == "OFFSET" {
                self.next_token(); // expr -> offset
                let mut offset = self.construct_node();
                self.next_token(); // offset -> expr
                offset.push_node(
                    "expr",
                    self.parse_expr(999, &vec!["union", "intersect", "except", ";"], false),
                );
                limit.push_node("offset", offset);
            }
            node.push_node("limit", limit);
        }
        // union
        while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
            self.next_token(); // stmt -> union
            let mut operator = self.construct_node();
            self.next_token(); // union -> distinct
            operator.push_node("distinct", self.construct_node());
            operator.push_node("left", node);
            self.next_token(); // distinct -> stmt
            operator.push_node("right", self.parse_select_statement(false));
            node = operator;
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node())
            }
        }
        // ;
        if self.peek_token_is(";") && root {
            self.next_token(); // expr -> ;
            node.push_node("semicolon", self.construct_node())
        }
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
    fn parse_table(&mut self, root: bool) -> cst::Node {
        match self.get_token(0).literal.to_uppercase().as_str() {
            "(" => {
                let mut group = self.construct_node();
                self.next_token(); // ( -> table
                group.push_node("expr", self.parse_table(true));
                self.next_token(); // table -> )
                group.push_node("rparen", self.construct_node());
                group = self.parse_alias(group);
                return group;
            }
            _ => (),
        }
        let mut left = self.parse_expr(
            999,
            &vec![
                "where", "group", "having", "limit", ";", "on", ",", "left", "right", "cross",
                "inner", "join",
            ],
            true,
        );
        if self.get_token(1).literal.to_uppercase().as_str() == "FOR" {
            self.next_token(); // table -> for
            let mut for_ = self.construct_node();
            self.next_token(); // for -> system_time
            let mut system_time_as_of = Vec::new();
            system_time_as_of.push(self.construct_node());
            self.next_token(); // system_time -> as
            system_time_as_of.push(self.construct_node());
            self.next_token(); // as -> of
            system_time_as_of.push(self.construct_node());
            for_.push_node_vec("system_time_as_of", system_time_as_of);
            self.next_token(); // of -> timestamp
            for_.push_node(
                "expr",
                self.parse_expr(
                    999,
                    &vec![
                        "on", "left", "right", "cross", "inner", ",", "full", "join", "where",
                        "group", "having", ";",
                    ],
                    false,
                ),
            );
            left.push_node("for_system_time_as_of", for_);
        }
        if self.get_token(1).literal.to_uppercase().as_str() == "WITH" {
            self.next_token(); // unnest() -> with
            let mut with = self.construct_node();
            self.next_token(); // with -> offset
            with.push_node(
                "offset",
                self.parse_expr(
                    999,
                    &vec![
                        "on", "left", "right", "cross", "inner", ",", "full", "join", "where",
                        "group", "having", ";",
                    ],
                    true,
                ),
            );
            left.push_node("with", with);
        }
        while self.peek_token_in(&vec![
            "left", "right", "cross", "inner", "full", "join", ",",
        ]) && root
        {
            self.next_token(); // table -> left, right, inner, cross, full, join, ","
            let mut join = if self.cur_token_in(&vec!["join", ","]) {
                let join = self.construct_node();
                join
            } else {
                let mut type_ = self.construct_node();
                self.next_token(); // type -> outer, type -> join
                if self.cur_token_is("outer") {
                    type_.push_node("outer", self.construct_node());
                    self.next_token(); // outer -> join
                }
                let mut join = self.construct_node();
                join.push_node("type", type_);
                join
            };
            self.next_token(); // -> table
            let right = self.parse_table(false);
            if self.peek_token_is("on") {
                self.next_token(); // `table` -> on
                let mut on = self.construct_node();
                self.next_token(); // on -> expr
                on.push_node(
                    "expr",
                    self.parse_expr(
                        999,
                        &vec![
                            "left", "right", "cross", "inner", ",", "full", "join", "where",
                            "group", "having", ";",
                        ],
                        false,
                    ),
                );
                join.push_node("on", on);
            }
            join.push_node("left", left);
            join.push_node("right", right);
            left = join;
        }
        left
    }
    fn parse_exprs(&mut self, until: &Vec<&str>, alias: bool) -> Vec<cst::Node> {
        let mut exprs: Vec<cst::Node> = Vec::new();
        // first expr
        let mut expr = self.parse_expr(999, until, alias);
        if self.peek_token_is(",") {
            self.next_token(); // expr -> ,
            expr.push_node("comma", self.construct_node());
        }
        exprs.push(expr);
        // second expr and later
        while !self.peek_token_in(until) && !self.is_eof(1) {
            self.next_token();
            let mut expr = self.parse_expr(999, until, alias);
            if self.peek_token_is(",") {
                self.next_token(); // expr -> ,
                expr.push_node("comma", self.construct_node());
            }
            exprs.push(expr);
        }
        exprs
    }
    fn parse_expr(&mut self, precedence: usize, until: &Vec<&str>, alias: bool) -> cst::Node {
        // prefix or literal
        let mut left = self.construct_node();
        match self.get_token(0).literal.to_uppercase().as_str() {
            "*" => {
                match self.get_token(1).literal.to_uppercase().as_str() {
                    "REPLACE" => {
                        self.next_token(); // * -> replace
                        let mut replace = self.construct_node();
                        self.next_token(); // replace -> (
                        let mut group = self.construct_node();
                        let mut exprs = Vec::new();
                        while self.get_token(1).literal.as_str() != ")" {
                            self.next_token(); // ( -> expr, ident -> expr
                            let expr = self.parse_expr(999, &vec![")"], true);
                            exprs.push(expr);
                        }
                        self.next_token(); // ident -> )
                        group.push_node("rparen", self.construct_node());
                        group.push_node_vec("exprs", exprs);
                        replace.push_node("group", group);
                        left.push_node("replace", replace);
                    }
                    "EXCEPT" => {
                        self.next_token(); // * -> except
                        let mut except = self.construct_node();
                        self.next_token(); // except -> (
                        let mut group = self.construct_node();
                        self.next_token(); // ( -> exprs
                        group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                        self.next_token(); // exprs -> )
                        group.push_node("rparen", self.construct_node());
                        except.push_node("group", group);
                        left.push_node("except", except);
                    }
                    _ => (),
                }
            }
            "(" => {
                self.next_token(); // ( -> expr
                let exprs = self.parse_exprs(&vec![")"], false);
                if exprs.len() == 1 {
                    left.push_node("expr", exprs[0].clone());
                } else {
                    left.push_node_vec("exprs", exprs);
                }
                //left.push_node("expr", self.parse_expr(999, until));
                self.next_token(); // expr -> )
                left.push_node("rparen", self.construct_node());
            }
            "-" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until, false);
                left.push_node("right", right);
            }
            "[" => {
                self.next_token(); // [ -> exprs
                left.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                self.next_token(); // exprs -> ]
                left.push_node("rparen", self.construct_node());
            }
            "DATE" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // date -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "TIMESTAMP" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "NUMERIC" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "BIGNUMERIC" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "DECIMAL" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "BIGDECIMAL" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // timestamp -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "INTERVAL" => {
                self.next_token(); // interval -> expr
                let right = self.parse_expr(001, &vec!["hour", "month", "year"], false);
                self.next_token(); // expr -> hour
                left.push_node("date_part", self.construct_node());
                left.push_node("right", right);
            }
            "R" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "B" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "BR" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "RB" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "SELECT" => {
                left = self.parse_select_statement(true);
            }
            "NOT" => {
                self.next_token(); // not -> boolean
                let right = self.parse_expr(110, until, false);
                left.push_node("right", right);
            }
            "ARRAY" => {
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // ARRAY -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type
                    type_.push_node("type", self.parse_type());
                    self.next_token(); // type -> >
                    type_.push_node("rparen", self.construct_node());
                    left.push_node("type_declaration", type_);
                }
                self.next_token(); // ARRAY -> [, > -> [
                let mut right = self.construct_node();
                self.next_token(); // [ -> exprs
                right.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                self.next_token(); // exprs -> ]
                right.push_node("rparen", self.construct_node());
                left.push_node("right", right);
            }
            "STRUCT" => {
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // struct -> <
                    let mut type_ = self.construct_node();
                    let mut type_declarations = Vec::new();
                    self.next_token(); // < -> ident or type
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node();
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = cst::Node::new_none();
                        }
                        type_declaration.push_node("type", self.parse_type());
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration.push_node("comma", self.construct_node());
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node());
                    type_.push_node_vec("declarations", type_declarations);
                    left.push_node("type_declaration", type_);
                }
                self.next_token(); // struct -> (, > -> (
                let mut right = self.construct_node();
                self.next_token(); // ( -> exprs
                right.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                self.next_token(); // exprs -> )
                right.push_node("rparen", self.construct_node());
                left.push_node("right", right);
            }
            "CASE" => {
                self.next_token(); // case -> expr, case -> when
                if !self.cur_token_is("WHEN") {
                    left.push_node("expr", self.parse_expr(999, &vec!["WHEN"], false));
                    self.next_token(); // expr -> when
                }
                let mut arms = Vec::new();
                while !self.cur_token_is("ELSE") {
                    let mut arm = self.construct_node();
                    self.next_token(); // when -> expr
                    arm.push_node("expr", self.parse_expr(999, &vec!["then"], false));
                    self.next_token(); // expr -> then
                    arm.push_node("then", self.construct_node());
                    self.next_token(); // then -> result_expr
                    arm.push_node("result", self.parse_expr(999, &vec!["else", "when"], false));
                    self.next_token(); // result_expr -> else, result_expr -> when
                    arms.push(arm)
                }
                let mut arm = self.construct_node();
                self.next_token(); // else -> result_expr
                arm.push_node("result", self.construct_node());
                arms.push(arm);
                left.push_node_vec("arms", arms);
                self.next_token(); // result_expr -> end
                left.push_node("end", self.construct_node());
            }
            _ => (),
        };
        // infix
        while !self.peek_token_in(until) && self.get_precedence(1) < precedence {
            // actually, until is not needed
            match self.get_token(1).literal.to_uppercase().as_str() {
                "BETWEEN" => {
                    self.next_token(); // expr -> between
                    let precedence = self.get_precedence(0);
                    let mut between = self.construct_node();
                    between.push_node("left", left);
                    left = between;
                    self.next_token(); // between -> expr1
                    let mut exprs = Vec::new();
                    exprs.push(self.parse_expr(precedence, until, false));
                    self.next_token(); // expr1 -> and
                    left.push_node("and", self.construct_node());
                    self.next_token(); // and -> expr2
                    exprs.push(self.parse_expr(precedence, until, false));
                    left.push_node_vec("right", exprs);
                }
                "LIKE" => {
                    self.next_token(); // expr -> like
                    left = self.parse_binary_operator(left, until);
                }
                "." => {
                    self.next_token(); // expr -> .
                    left = self.parse_binary_operator(left, until);
                }
                "+" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                "*" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                ">" => {
                    self.next_token(); // expr -> >
                    left = self.parse_binary_operator(left, until);
                }
                ">=" => {
                    self.next_token(); // expr -> >=
                    left = self.parse_binary_operator(left, until);
                }
                "<" => {
                    self.next_token(); // expr -> <
                    left = self.parse_binary_operator(left, until);
                }
                "<=" => {
                    self.next_token(); // expr -> <=
                    left = self.parse_binary_operator(left, until);
                }
                "=" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "AND" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "OR" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "IN" => {
                    self.next_token(); // expr -> in
                    left = self.parse_in_operator(left);
                }
                "[" => {
                    self.next_token(); // expr -> [
                    let mut node = self.construct_node();
                    node.push_node("left", left);
                    //let precedence = self.get_precedence(0);
                    self.next_token(); // [ -> index_expr
                    node.push_node("right", self.parse_expr(999, &vec!["]"], false));
                    self.next_token(); // index_expr -> ]
                    node.push_node("rparen", self.construct_node());
                    left = node;
                }
                "(" => {
                    self.next_token(); // expr -> (
                    let mut node = self.construct_node();
                    self.next_token(); // ( -> args
                    node.push_node("func", left);
                    if !self.cur_token_is(")") {
                        node.push_node_vec("args", self.parse_exprs(&vec![")"], false));
                        self.next_token(); // expr -> )
                    }
                    node.push_node("rparen", self.construct_node());
                    if self.peek_token_is("over") {
                        self.next_token(); // ) -> over
                        let mut over = self.construct_node();
                        if self.peek_token_is("(") {
                            self.next_token(); // over -> (
                            let mut window = self.construct_node();
                            if self.get_token(1).is_identifier() {
                                self.next_token(); // ( -> identifier
                                window.push_node("name", self.construct_node());
                            }
                            if self.peek_token_is("partition") {
                                self.next_token(); // ( -> partition, order, frame
                                let mut partition = self.construct_node();
                                self.next_token(); // partition -> by
                                partition.push_node("by", self.construct_node());
                                self.next_token(); // by -> exprs
                                partition.push_node_vec(
                                    "exprs",
                                    self.parse_exprs(&vec!["order", ")"], false),
                                );
                                window.push_node("partitionby", partition);
                            }
                            if self.peek_token_is("order") {
                                self.next_token(); // ( -> order, expr -> order
                                let mut order = self.construct_node();
                                self.next_token(); // order -> by
                                order.push_node("by", self.construct_node());
                                self.next_token(); // by -> exprs
                                order.push_node_vec(
                                    "exprs",
                                    self.parse_exprs(&vec!["rows", "range", ")"], false),
                                );
                                window.push_node("orderby", order);
                            }
                            if self.peek_token_in(&vec!["range", "rows"]) {
                                self.next_token(); // ( -> rows, expr -> rows
                                let mut frame = self.construct_node();
                                if self.peek_token_is("between") {
                                    // frame between
                                    self.next_token(); // rows -> between
                                    frame.push_node("between", self.construct_node());
                                    self.next_token(); // between -> expr
                                    let mut start = self.parse_expr(999, &vec!["preceding"], false);
                                    self.next_token(); // expr -> preceding
                                    start.push_node("preceding", self.construct_node());
                                    frame.push_node("start", start);
                                    self.next_token(); // preceding -> and
                                    frame.push_node("and", self.construct_node());
                                    self.next_token(); // and -> expr
                                    let mut end = self.parse_expr(999, &vec![")"], false);
                                    self.next_token(); // expr -> following
                                    end.push_node("following", self.construct_node());
                                    frame.push_node("end", end);
                                } else {
                                    // frame start
                                    if !self.peek_token_is(")") {
                                        self.next_token(); // rows -> expr
                                        let mut start =
                                            self.parse_expr(999, &vec!["preceding"], false);
                                        self.next_token(); // expr -> preceding, row
                                        start.push_node("preceding", self.construct_node());
                                        frame.push_node("start", start);
                                    }
                                }
                                window.push_node("frame", frame)
                            }
                            self.next_token(); // -> )
                            window.push_node("rparen", self.construct_node());
                            over.push_node("window", window);
                            node.push_node("over", over);
                        } else {
                            self.next_token(); // over -> identifier
                            over.push_node("window", self.construct_node());
                            node.push_node("over", over);
                        }
                    }
                    left = node;
                }
                "NOT" => {
                    self.next_token(); // expr -> not
                    let not = self.construct_node();
                    self.next_token(); // not -> in, like
                    if self.cur_token_is("in") {
                        left = self.parse_in_operator(left);
                        left.push_node("not", not);
                    } else if self.cur_token_is("like") {
                        left = self.parse_binary_operator(left, until);
                        left.push_node("not", not);
                    } else {
                        panic!();
                    }
                }
                _ => panic!(),
            }
        }
        // alias
        if self.peek_token_is("as") && precedence == 999 {
            self.next_token(); // expr -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> alias
            as_.push_node("alias", self.construct_node());
            left.push_node("as", as_);
        }
        if self.get_token(1).is_identifier() && !self.is_eof(1) && precedence == 999 && alias {
            self.next_token(); // expr -> alias
            let mut as_ = cst::Node {
                token: None,
                children: HashMap::new(),
            };
            as_.push_node("alias", self.construct_node());
            left.push_node("as", as_);
        }
        if self.peek_token_in(&vec!["asc", "desc"]) {
            self.next_token(); // expr -> asc
            let order = self.construct_node();
            left.push_node("order", order);
        }
        if self.peek_token_in(&vec!["nulls"]) {
            self.next_token(); // asc -> nulls, expr -> nulls
            let mut nulls = self.construct_node();
            self.next_token(); // nulls -> first, last
            nulls.push_node("first", self.construct_node());
            left.push_node("nulls", nulls);
        }
        left
    }
    fn parse_alias(&mut self, node: cst::Node) -> cst::Node {
        let mut node = node.clone();
        if self.peek_token_is("as") {
            self.next_token(); // expr -> as
            let mut as_ = self.construct_node();
            self.next_token(); // as -> alias
            as_.push_node("alias", self.construct_node());
            node.push_node("as", as_);
        } else if self.get_token(1).is_identifier() && !self.is_eof(1) {
            self.next_token(); // expr -> alias
            let mut as_ = cst::Node {
                token: None,
                children: HashMap::new(),
            };
            as_.push_node("alias", self.construct_node());
            node.push_node("as", as_);
        }
        node
    }
    fn parse_binary_operator(&mut self, left: cst::Node, until: &Vec<&str>) -> cst::Node {
        let precedence = self.get_precedence(0);
        let mut node = self.construct_node();
        self.next_token(); // binary_operator -> expr
        node.push_node("left", left);
        node.push_node("right", self.parse_expr(precedence, until, false));
        node
    }
    fn parse_in_operator(&mut self, mut left: cst::Node) -> cst::Node {
        let mut node = self.construct_node();
        self.next_token(); // in -> (
        node.push_node("left", left);
        let mut right = self.construct_node();
        self.next_token(); // ( -> expr
        right.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
        self.next_token(); // expr -> )
        right.push_node("rparen", self.construct_node());
        node.push_node("right", right);
        node
    }
    fn peek_token_is(&self, s: &str) -> bool {
        self.get_token(1).literal.to_uppercase() == s.to_uppercase()
    }
    fn cur_token_is(&self, s: &str) -> bool {
        self.get_token(0).literal.to_uppercase() == s.to_uppercase()
    }
    fn parse_type(&mut self) -> cst::Node {
        let res = match self.get_token(0).literal.to_uppercase().as_str() {
            "ARRAY" => {
                let mut res = self.construct_node();
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type_expr
                    type_.push_node("type", self.parse_type());
                    self.next_token(); // type_expr -> >
                    type_.push_node("rparen", self.construct_node());
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "STRUCT" => {
                let mut res = self.construct_node();
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node();
                    self.next_token(); // < -> type or ident
                    let mut type_declarations = Vec::new();
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node();
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = cst::Node::new_none();
                        }
                        type_declaration.push_node("type", self.parse_type());
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration.push_node("comma", self.construct_node());
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node());
                    type_.push_node_vec("declarations", type_declarations);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "ANY" => {
                let mut res = self.construct_node();
                self.next_token(); // ANY -> TYPE
                res.push_node("type", self.construct_node());
                res
            }
            _ => self.construct_node(),
        };
        res
    }
    fn get_precedence(&self, offset: usize) -> usize {
        // precedenc
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators
        // 001... date, timestamp, r'', b'' (literal)
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
        match self.get_token(offset).literal.to_uppercase().as_str() {
            "(" => 005,
            "." => 101,
            "[" => 101,
            "*" => 103,
            "/" => 103,
            "||" => 103,
            "-" => 104,
            "+" => 104,
            ">" => 109,
            ">=" => 109,
            "<" => 109,
            "<=" => 109,
            "IN" => 109,
            "LIKE" => 109,
            "BETWEEN" => 109,
            "=" => 109,
            "NOT" => {
                match self.get_token(offset + 1).literal.to_uppercase().as_str() {
                    "IN" => {
                        return 109;
                    }
                    "LIKE" => {
                        return 109;
                    }
                    _ => (),
                }
                110
            }
            "AND" => 111,
            "OR" => 112,
            _ => 999,
        }
    }
}

#[cfg(test)]
mod tests;
