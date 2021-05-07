#[cfg(test)]
mod tests;

use crate::cst::Node;
use crate::cst::NodeType;
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser {
    position: usize,
    leading_comment_indices: Vec<usize>,
    trailing_comment_indices: Vec<usize>,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(code: String) -> Parser {
        let mut l = Lexer::new(code);
        l.tokenize_code();
        let mut p = Parser {
            position: 0,
            leading_comment_indices: Vec::new(),
            trailing_comment_indices: Vec::new(),
            tokens: l.tokens,
        };
        while p.tokens[p.position].is_comment() {
            p.leading_comment_indices.push(p.position);
            p.position += 1;
        }
        let mut trailing_comment_idx = p.position + 1;
        while p.tokens[trailing_comment_idx].is_comment() {
            p.trailing_comment_indices.push(trailing_comment_idx);
            trailing_comment_idx += 1;
        }
        p
    }
    fn get_offset_index(&self, offset: usize) -> Option<usize> {
        if offset == 0 {
            return Some(self.position);
        }
        let mut cnt = 0;
        let mut idx = self.position + 1;
        loop {
            if idx < self.tokens.len() {
                if !self.tokens[idx].is_comment() {
                    cnt += 1;
                    if offset <= cnt {
                        break;
                    }
                }
                idx += 1;
            } else {
                return None;
            }
        }
        Some(idx)
    }
    fn next_token(&mut self) {
        // leading comments
        self.leading_comment_indices = Vec::new();
        let next_token_idx = match self.get_offset_index(1) {
            Some(i) => i,
            None => panic!(
                "Next token was not found. Current token is: {:?}",
                self.get_token(0)
            ),
        };
        let from_idx = match self.trailing_comment_indices.last() {
            Some(n) => *n + 1,
            None => self.position + 1,
        };
        for i in from_idx..next_token_idx {
            self.leading_comment_indices.push(i);
        }
        self.position = next_token_idx;
        // trailing comments
        self.trailing_comment_indices = Vec::new();
        let next_token_idx = match self.get_offset_index(1) {
            Some(i) => i,
            None => return (), // already reached EOF
        };
        let mut trailing_comment_idx = self.position + 1;
        while trailing_comment_idx < next_token_idx
            && self.get_token(0).line == self.tokens[trailing_comment_idx].line
        {
            self.trailing_comment_indices.push(trailing_comment_idx);
            trailing_comment_idx += 1;
        }
    }
    fn get_token(&self, offset: usize) -> &Token {
        let idx = match self.get_offset_index(offset) {
            Some(i) => i,
            None => panic!(
                "{}st token was not found. Current token is: {:?}",
                offset,
                self.get_token(0)
            ),
        };
        return &self.tokens[idx];
    }
    fn is_eof(&self, offset: usize) -> bool {
        let idx = match self.get_offset_index(offset) {
            Some(i) => i,
            None => return true,
        };
        self.tokens.len() - 1 <= idx
    }
    fn construct_node(&self, node_type: NodeType) -> Node {
        let curr_token = self.get_token(0);
        let mut node = match node_type {
            NodeType::EOF => Node::empty(node_type),
            NodeType::Unknown => {
                let mut node = Node::new(curr_token.clone(), node_type);
                if curr_token.is_identifier() {
                    node.node_type = NodeType::Identifier;
                } else if curr_token.is_numeric() {
                    node.node_type = NodeType::NumericLiteral;
                } else if curr_token.is_string() {
                    node.node_type = NodeType::StringLiteral;
                } else if curr_token.is_boolean() {
                    node.node_type = NodeType::BooleanLiteral;
                } else if curr_token.literal.to_uppercase() == "NULL" {
                    node.node_type = NodeType::NullLiteral;
                }
                node
            }
            _ => Node::new(self.get_token(0).clone(), node_type),
        };
        // leading_comments
        let mut leading_comment_nodes = Vec::new();
        for idx in &self.leading_comment_indices {
            leading_comment_nodes.push(Node::new(self.tokens[*idx].clone(), NodeType::Comment))
        }
        if 0 < leading_comment_nodes.len() {
            node.push_node_vec("leading_comments", leading_comment_nodes);
        }
        // trailing comments
        let mut trailing_comment_nodes = Vec::new();
        for idx in &self.trailing_comment_indices {
            trailing_comment_nodes.push(Node::new(self.tokens[*idx].clone(), NodeType::Comment))
        }
        if 0 < trailing_comment_nodes.len() {
            node.push_node_vec("trailing_comments", trailing_comment_nodes);
        }
        node
    }
    // ----- common -----
    pub fn parse_code(&mut self) -> Vec<Node> {
        let mut stmts: Vec<Node> = Vec::new();
        while !self.is_eof(0) {
            let stmt = self.parse_statement();
            stmts.push(stmt);
            self.next_token();
        }
        stmts.push(self.construct_node(NodeType::EOF));
        stmts
    }
    fn parse_statement(&mut self) -> Node {
        let node = match self.get_token(0).literal.to_uppercase().as_str() {
            // SELECT
            "WITH" | "SELECT" | "(" => self.parse_select_statement(true),
            // DML
            "INSERT" => self.parse_insert_statement(true),
            "DELETE" => self.parse_delete_statement(),
            "TRUNCATE" => self.parse_truncate_statement(),
            "UPDATE" => self.parse_update_statement(true),
            "MERGE" => self.parse_merge_statement(true),
            // DDL
            "CREATE" => {
                let mut offset = 1;
                let mut target = self.get_token(offset).literal.to_uppercase();
                loop {
                    match target.as_str() {
                        "TEMP" => {
                            offset += 1;
                            target = self.get_token(offset).literal.to_uppercase();
                        }
                        "TEMPORARY" => {
                            offset += 1;
                            target = self.get_token(offset).literal.to_uppercase();
                        }
                        "OR" => {
                            offset += 2;
                            target = self.get_token(offset).literal.to_uppercase();
                        }
                        _ => break,
                    }
                }
                // TODO separete functions
                match target.as_str() {
                    "SCHEMA" | "TABLE" | "VIEW" | "MATERIALIZED" | "EXTERNAL" => {
                        self.parse_create_table_statement()
                    }
                    "FUNCTION" => self.parse_create_function_statement(),
                    "PROCEDURE" => self.parse_create_procedure_statement(),
                    _ => panic!("Cannot decide what to create."),
                }
            }
            "ALTER" => self.parse_alter_statement(),
            "DROP" => self.parse_drop_statement(),
            // DEBUG
            "ASSERT" => panic!("not implementd!"),
            // other
            "EXPORT" => panic!("not implementd!"),
            // script
            "DECLARE" => self.parse_declare_statement(),
            "SET" => self.parse_set_statement(),
            "EXECUTE" => self.parse_execute_statement(),
            "IF" => self.parse_if_statement(),
            "BEGIN" => self.parse_begin_statement(true),
            "LOOP" => self.parse_loop_statement(),
            "WHILE" => self.parse_while_statement(),
            "BREAK" | "LEAVE" | "CONTINUE" | "ITERATE" | "RETURN" => {
                self.parse_single_token_statement()
            }
            "RAISE" => self.parse_raise_statement(),
            "CALL" => self.parse_call_statement(),
            _ => panic!(
                "Calling `parse_staement()` is not allowed here: {:?}",
                self.get_token(0)
            ),
        };
        node
    }
    fn parse_keyword_with_statements(&mut self, until: &Vec<&str>) -> Node {
        let mut node = self.construct_node(NodeType::KeywordWithStatements);
        let mut stmts = Vec::new();
        while !self.get_token(1).in_(until) {
            self.next_token(); // -> stmt
            stmts.push(self.parse_statement());
        }
        if 0 < stmts.len() {
            node.push_node_vec("stmts", stmts);
        }
        node
    }
    // ----- SELECT statement -----
    fn parse_select_statement(&mut self, root: bool) -> Node {
        if self.get_token(0).literal.to_uppercase() == "(" {
            let mut node = self.construct_node(NodeType::GroupedStatement);
            self.next_token(); // ( -> SELECT
            node.push_node("stmt", self.parse_select_statement(true));
            self.next_token(); // stmt -> )
            node.push_node("rparen", self.construct_node(NodeType::Symbol));
            while self.get_token(1).in_(&vec!["union", "intersect", "except"]) && root {
                self.next_token(); // stmt -> UNION
                let mut operator = self.construct_node(NodeType::SetOperator);
                self.next_token(); // UNION -> DISTINCT
                operator.push_node("distinct_or_all", self.construct_node(NodeType::Keyword));
                operator.push_node("left", node);
                self.next_token(); // DISTINCT -> stmt
                operator.push_node("right", self.parse_select_statement(false));
                node = operator;
            }
            if self.get_token(1).is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node(NodeType::Symbol))
            }
            return node;
        }
        if self.get_token(0).literal.to_uppercase() == "WITH" {
            let mut with = self.construct_node(NodeType::WithClause);
            let mut queries = Vec::new();
            while self.get_token(1).literal.to_uppercase() != "SELECT" {
                self.next_token(); // WITH -> ident, ) -> ident
                let mut query = self.construct_node(NodeType::WithQuery);
                self.next_token(); // ident -> AS
                query.push_node("as", self.construct_node(NodeType::Keyword));
                self.next_token(); // AS -> (
                query.push_node("stmt", self.parse_select_statement(true));
                if self.get_token(1).literal.as_str() == "," {
                    self.next_token(); // ) -> ,
                    query.push_node("comma", self.construct_node(NodeType::Symbol));
                }
                queries.push(query);
            }
            with.push_node_vec("queries", queries);
            self.next_token(); // ) -> SELECT
            let mut node = self.parse_select_statement(true);
            node.push_node("with", with);
            return node;
        }
        // SELECT
        let mut node = self.construct_node(NodeType::SelectStatement);

        // AS STRUCT, VALUE
        if self.get_token(1).literal.to_uppercase() == "AS" {
            self.next_token(); // SELECT -> AS
            let as_ = self.construct_node(NodeType::Keyword);
            self.next_token(); // AS -> STRUCT, VALUE
            node.push_node_vec(
                "as_struct_or_value",
                vec![as_, self.construct_node(NodeType::Keyword)],
            );
        }

        // DISTINCT
        if self.get_token(1).in_(&vec!["ALL", "DISTINCT"]) {
            self.next_token(); // select -> all, distinct
            node.push_node("distinct_or_all", self.construct_node(NodeType::Keyword));
        }
        self.next_token(); // -> expr

        // exprs
        node.push_node_vec(
            "exprs",
            self.parse_exprs(
                &vec![
                    "from",
                    ";",
                    "limit",
                    ")",
                    "union",
                    "intersect",
                    "except",
                    "where",
                ],
                true,
            ),
        );
        // FROM
        if self.get_token(1).is("FROM") {
            self.next_token(); // expr -> FROM
            let mut from = self.construct_node(NodeType::KeywordWithExpr);
            self.next_token(); // FROM -> table
            from.push_node("expr", self.parse_table(true));
            node.push_node("from", from);
        }
        // WHERE
        if self.get_token(1).is("WHERE") {
            self.next_token(); // expr -> WHERE
            let mut where_ = self.construct_node(NodeType::KeywordWithExpr);
            self.next_token(); // WHERE -> expr
            where_.push_node(
                "expr",
                self.parse_expr(
                    999,
                    &vec!["group", "having", ";", "order", ",", "window"],
                    false,
                ),
            );
            node.push_node("where", where_);
        }
        // GROUP BY
        if self.get_token(1).is("GROUP") {
            self.next_token(); // expr -> GROUP
            let mut groupby = self.construct_node(NodeType::XXXByExprs);
            self.next_token(); // GROUP -> BY
            groupby.push_node("by", self.construct_node(NodeType::Keyword));
            self.next_token(); // BY -> expr
            groupby.push_node_vec(
                "exprs",
                self.parse_exprs(&vec!["having", "limit", ";", "order", ")", "window"], false),
            );
            node.push_node("groupby", groupby);
        }
        // HAVING
        if self.get_token(1).is("HAVING") {
            self.next_token(); // expr -> HAVING
            let mut having = self.construct_node(NodeType::KeywordWithExpr);
            self.next_token(); // HAVING -> expr
            having.push_node(
                "expr",
                self.parse_expr(999, &vec!["LIMIT", ";", "order", ")", "window"], false),
            );
            node.push_node("having", having);
        }
        // WINDOW
        if self.get_token(1).is("WINDOW") {
            self.next_token(); // -> WINDOW
            let mut window = self.construct_node(NodeType::WindowClause);
            let mut window_exprs = Vec::new();
            while self.get_token(1).is_identifier() {
                self.next_token(); // -> ident
                let mut window_expr = self.construct_node(NodeType::WindowExpr);
                self.next_token(); // ident -> AS
                window_expr.push_node("as", self.construct_node(NodeType::Keyword));
                self.next_token(); // AS -> (, AS -> named_window
                window_expr.push_node("window", self.parse_window_expr());
                if self.get_token(1).is(",") {
                    self.next_token(); // -> ,
                    window_expr.push_node("comma", self.construct_node(NodeType::Symbol));
                }
                window_exprs.push(window_expr);
            }
            window.push_node_vec("window_exprs", window_exprs);
            node.push_node("window", window);
        }
        // ORDER BY
        if self.get_token(1).is("ORDER") {
            self.next_token(); // expr -> ORDER
            let mut order = self.construct_node(NodeType::XXXByExprs);
            self.next_token(); // ORDER -> BY
            order.push_node("by", self.construct_node(NodeType::Keyword));
            self.next_token(); // BY -> expr
            order.push_node_vec(
                "exprs",
                self.parse_exprs(&vec!["limit", ",", ";", ")"], false),
            );
            node.push_node("orderby", order);
        }
        // LIMIT
        if self.get_token(1).is("LIMIT") {
            self.next_token(); // expr -> LIMIT
            let mut limit = self.construct_node(NodeType::LimitClause);
            self.next_token(); // LIMIT -> expr
            limit.push_node(
                "expr",
                self.parse_expr(999, &vec![";", ",", "offset", ")"], false), // NOTE offset is not reserved
            );
            if self.get_token(1).literal.to_uppercase() == "OFFSET" {
                self.next_token(); // expr -> OFFSET
                let mut offset = self.construct_node(NodeType::KeywordWithExpr);
                self.next_token(); // OFFSET -> expr
                offset.push_node(
                    "expr",
                    self.parse_expr(999, &vec!["union", "intersect", "except", ";", ")"], false),
                );
                limit.push_node("offset", offset);
            }
            node.push_node("limit", limit);
        }
        // UNION
        while self.get_token(1).in_(&vec!["UNION", "INTERSECT", "EXCEPT"]) && root {
            self.next_token(); // stmt -> UNION
            let mut operator = self.construct_node(NodeType::SetOperator);
            self.next_token(); // UNION -> DISTINCT
            operator.push_node("distinct_or_all", self.construct_node(NodeType::Keyword));
            operator.push_node("left", node);
            self.next_token(); // DISTINCT -> stmt
            operator.push_node("right", self.parse_select_statement(false));
            node = operator;
            if self.get_token(1).is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node(NodeType::Symbol))
            }
        }
        // ;
        if self.get_token(1).is(";") && root {
            self.next_token(); // expr -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol))
        }
        node
    }
    // ----- DML -----
    fn parse_insert_statement(&mut self, root: bool) -> Node {
        let mut insert = self.construct_node(NodeType::InsertStatement);
        if self.get_token(1).is("INTO") {
            self.next_token(); // INSERT -> INTO
            insert.push_node("into", self.construct_node(NodeType::Keyword));
        }
        if !self.get_token(1).in_(&vec!["(", "VALUE", "ROW"]) {
            self.next_token(); // INSERT -> identifier
            insert.push_node("target_name", self.parse_identifier());
        }
        if self.get_token(1).is("(") {
            self.next_token(); // identifier -> (
            let mut group = self.construct_node(NodeType::GroupedExprs);
            self.next_token(); // ( -> columns
            group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            self.next_token(); // columns -> )
            group.push_node("rparen", self.construct_node(NodeType::Symbol));
            insert.push_node("columns", group);
        }
        if self.get_token(1).is("values") {
            self.next_token(); // ) -> values
            let mut values = self.construct_node(NodeType::KeywordWithExprs);
            let mut lparens = Vec::new();
            while self.get_token(1).is("(") {
                self.next_token(); // VALUES -> (, ',' -> (
                let mut lparen = self.construct_node(NodeType::GroupedExprs);
                self.next_token(); // -> expr
                lparen.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                self.next_token(); // expr -> )
                lparen.push_node("rparen", self.construct_node(NodeType::Symbol));
                if self.get_token(1).is(",") {
                    self.next_token(); // ) -> ,
                    lparen.push_node("comma", self.construct_node(NodeType::Symbol));
                }
                lparens.push(lparen);
            }
            values.push_node_vec("exprs", lparens);
            insert.push_node("input", values);
        } else if self.get_token(1).is("row") {
            self.next_token(); // -> ROW
            insert.push_node("input", self.construct_node(NodeType::Keyword));
        } else {
            self.next_token(); // ) -> SELECT
            insert.push_node("input", self.parse_select_statement(false));
        }
        if self.get_token(1).is(";") && root {
            self.next_token(); // -> ;
            insert.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        insert
    }
    fn parse_delete_statement(&mut self) -> Node {
        let mut delete = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("from") {
            self.next_token(); // delete -> from
            delete.push_node("from", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> target_name
        let mut target_name = self.parse_identifier();
        if !self.get_token(1).is("where") {
            self.next_token(); // -> AS, ident
            if self.get_token(0).is("AS") {
                target_name.push_node("as", self.construct_node(NodeType::Keyword));
                self.next_token(); // AS -> ident
            }
            target_name.push_node("alias", self.construct_node(NodeType::Identifier));
        }
        delete.push_node("target_name", target_name);
        self.next_token(); // target_name -> where, alias -> where
        let mut where_ = self.construct_node(NodeType::Unknown);
        self.next_token(); // where -> expr
        where_.push_node("expr", self.parse_expr(999, &vec![";"], false));
        delete.push_node("where", where_);
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            delete.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        delete
    }
    fn parse_truncate_statement(&mut self) -> Node {
        let mut truncate = self.construct_node(NodeType::Unknown);
        self.next_token(); // truncate -> table
        truncate.push_node("table", self.construct_node(NodeType::Unknown));
        self.next_token(); // table -> ident
        truncate.push_node("target_name", self.parse_identifier());
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            truncate.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        truncate
    }
    fn parse_update_statement(&mut self, root: bool) -> Node {
        let mut update = self.construct_node(NodeType::Unknown);
        if !self.get_token(1).is("set") {
            self.next_token(); // -> target_name
            update.push_node("target_name", self.parse_table(true));
        }
        self.next_token(); // -> set
        let mut set = self.construct_node(NodeType::Unknown);
        self.next_token(); // set -> exprs
        set.push_node_vec(
            "exprs",
            self.parse_exprs(&vec!["from", "where", "when", ";"], false),
        );
        if self.get_token(1).is("from") {
            self.next_token(); // exprs -> from
            let mut from = self.construct_node(NodeType::Unknown);
            self.next_token(); // from -> target_name
            from.push_node("expr", self.parse_table(true));
            update.push_node("from", from);
        }
        update.push_node("set", set);
        if self.get_token(1).is("where") {
            self.next_token(); // exprs -> where
            let mut where_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // where -> expr
            where_.push_node("expr", self.parse_expr(999, &vec![";"], false));
            update.push_node("where", where_);
        }
        if self.get_token(1).is(";") && root {
            self.next_token(); // -> ;
            update.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        update
    }
    fn parse_merge_statement(&mut self, root: bool) -> Node {
        let mut merge = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("into") {
            self.next_token(); // merge -> into
            merge.push_node("into", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> target_table
        merge.push_node("target_name", self.parse_table(true));
        self.next_token(); // -> using
        let mut using = self.construct_node(NodeType::Unknown);
        self.next_token(); // using -> expr
        using.push_node("expr", self.parse_expr(999, &vec!["on"], true));
        merge.push_node("using", using);
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        self.next_token(); // -> on
        let mut on = self.construct_node(NodeType::Unknown);
        self.next_token(); // on -> expr
        on.push_node("expr", self.parse_expr(999, &vec!["when"], false));
        merge.push_node("on", on);
        let mut whens = Vec::new();
        while self.get_token(1).is("when") {
            self.next_token(); // -> when
            let mut when = self.construct_node(NodeType::Unknown);
            if self.get_token(1).is("not") {
                self.next_token(); // when -> not
                when.push_node("not", self.construct_node(NodeType::Unknown));
            }
            self.next_token(); // -> matched
            when.push_node("matched", self.construct_node(NodeType::Unknown));
            if self.get_token(1).is("by") {
                self.next_token(); // -> by
                let by = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> target, source
                let target = self.construct_node(NodeType::Unknown);
                when.push_node_vec("by_target", vec![by, target]);
            }
            if self.get_token(1).is("and") {
                self.next_token(); // -> and
                let mut and = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> expr
                let cond = self.parse_expr(999, &vec!["then"], false);
                and.push_node("expr", cond);
                when.push_node("and", and);
            }
            self.next_token(); // -> then
            when.push_node("then", self.construct_node(NodeType::Unknown));
            self.next_token(); // then -> stmt
            let stmt = match self.get_token(0).literal.to_uppercase().as_str() {
                "DELETE" => self.construct_node(NodeType::Unknown),
                "UPDATE" => self.parse_update_statement(false),
                "INSERT" => self.parse_insert_statement(false),
                _ => panic!(),
            };
            when.push_node("stmt", stmt);
            whens.push(when);
        }
        merge.push_node_vec("whens", whens);
        if self.get_token(1).is(";") && root {
            self.next_token(); // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        merge
    }
    // ----- DDL -----
    fn parse_create_table_statement(&mut self) -> Node {
        let mut create = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("or") {
            self.next_token(); // -> or
            let or_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> replace
            let replace = self.construct_node(NodeType::Unknown);
            create.push_node_vec("or_replace", vec![or_, replace]);
        }
        if self.get_token(1).is("materialized") {
            self.next_token();
            create.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        if self.get_token(1).is("external") {
            self.next_token();
            create.push_node("external", self.construct_node(NodeType::Unknown));
        }
        if self.get_token(1).in_(&vec!["temp", "temporary"]) {
            self.next_token(); // -> temporary
            create.push_node("temp", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table
        create.push_node("what", self.construct_node(NodeType::Unknown));
        if self.get_token(1).is("if") {
            self.next_token(); // -> if
            let if_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> not
            let not = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> exists
            let exists = self.construct_node(NodeType::Unknown);
            create.push_node_vec("if_not_exists", vec![if_, not, exists]);
        }
        self.next_token(); // -> ident
        create.push_node("ident", self.parse_identifier());
        if self.get_token(1).is("(") {
            self.next_token(); // -> (
            let mut group = self.construct_node(NodeType::Unknown);
            let mut column_definitions = Vec::new();
            while !self.get_token(1).is(")") {
                self.next_token(); // -> column_identifier
                let mut column = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> type
                column.push_node("type", self.parse_type(true));
                if self.get_token(1).is(",") {
                    self.next_token(); // -> ,
                    column.push_node("comma", self.construct_node(NodeType::Unknown));
                }
                column_definitions.push(column);
            }
            group.push_node_vec("column_definitions", column_definitions);
            self.next_token(); // -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            create.push_node("column_schema_group", group);
        }
        if self.get_token(1).is("partition") {
            self.next_token(); // -> partition
            let mut partitionby = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> by
            partitionby.push_node("by", self.construct_node(NodeType::Unknown));
            self.next_token(); // -> expr
            partitionby.push_node(
                "expr",
                self.parse_expr(999, &vec!["cluster", "options", "as"], false),
            );
            create.push_node("partitionby", partitionby);
        }
        if self.get_token(1).is("cluster") {
            self.next_token(); // -> cluster
            let mut clusterby = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> by
            clusterby.push_node("by", self.construct_node(NodeType::Unknown));
            self.next_token(); // -> expr
            clusterby.push_node_vec("exprs", self.parse_exprs(&vec!["options", "as"], false)); // NOTE options is not reservved
            create.push_node("clusterby", clusterby);
        }
        if self.get_token(1).is("with") {
            self.next_token(); // -> with
            let mut with = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> partition
            let partition = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> columns
            let columns = self.construct_node(NodeType::Unknown);
            with.push_node_vec("partition_columns", vec![partition, columns]);
            if self.get_token(1).is("(") {
                self.next_token(); // -> "("
                let mut group = self.construct_node(NodeType::Unknown);
                let mut column_definitions = Vec::new();
                while !self.get_token(1).is(")") {
                    self.next_token(); // -> column_identifier
                    let mut column = self.construct_node(NodeType::Unknown);
                    self.next_token(); // -> type
                    column.push_node("type", self.parse_type(true));
                    if self.get_token(1).is(",") {
                        self.next_token(); // -> ,
                        column.push_node("comma", self.construct_node(NodeType::Unknown));
                    }
                    column_definitions.push(column);
                }
                if 0 < column_definitions.len() {
                    group.push_node_vec("column_definitions", column_definitions);
                }
                self.next_token(); // -> )
                group.push_node("rparen", self.construct_node(NodeType::Unknown));
                with.push_node("column_schema_group", group);
            }
            create.push_node("with_partition_columns", with);
        }
        if self.get_token(1).is("options") {
            self.next_token(); // options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.get_token(1).is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            create.push_node("options", options);
        }
        if self.get_token(1).is("as") {
            self.next_token(); // -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> stmt
            as_.push_node("stmt", self.parse_select_statement(false));
            create.push_node("as", as_);
        }
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        create
    }
    fn parse_create_function_statement(&mut self) -> Node {
        let mut node = self.construct_node(NodeType::Unknown);
        if self.get_token(1).literal.to_uppercase() == "OR" {
            let mut or_replace = Vec::new();
            self.next_token(); // create -> or
            or_replace.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // or -> replace
            or_replace.push(self.construct_node(NodeType::Unknown));
            node.push_node_vec("or_replace", or_replace);
        }
        if self.get_token(1).in_(&vec!["temporary", "temp"]) {
            self.next_token(); // -> temp
            node.push_node("temp", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> function
        node.push_node("what", self.construct_node(NodeType::Unknown));
        if self.get_token(1).in_(&vec!["if"]) {
            let mut if_not_exists = Vec::new();
            self.next_token(); // function -> if
            if_not_exists.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // if -> not
            if_not_exists.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // not -> exists
            if_not_exists.push(self.construct_node(NodeType::Unknown));
            node.push_node_vec("if_not_exists", if_not_exists);
        }
        self.next_token(); // -> ident
        node.push_node("ident", self.parse_identifier());
        self.next_token(); // ident -> (
        let mut group = self.construct_node(NodeType::Unknown);
        let mut args = Vec::new();
        while !self.get_token(1).is(")") {
            self.next_token(); // ( -> arg, ',' -> arg
            let mut arg = self.construct_node(NodeType::Unknown);
            self.next_token(); // arg -> type
            arg.push_node("type", self.parse_type(false));
            if self.get_token(1).is(",") {
                self.next_token(); // type -> ,
                arg.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            args.push(arg);
        }
        if args.len() > 0 {
            group.push_node_vec("args", args);
        }
        self.next_token(); // type -> )
        group.push_node("rparen", self.construct_node(NodeType::Unknown));
        node.push_node("group", group);
        if self.get_token(1).is("returns") {
            self.next_token(); // ) -> return
            let mut return_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // return -> type
            return_.push_node("type", self.parse_type(false));
            node.push_node("returns", return_);
        }
        if self.get_token(1).is("as") {
            self.next_token(); // -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> (
            let mut group = self.construct_node(NodeType::Unknown);
            self.next_token(); // ( -> expr
            group.push_node("expr", self.parse_expr(999, &vec![")"], false));
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            as_.push_node("group", group);
            node.push_node("as", as_);
        } else {
            if self.get_token(1).in_(&vec!["deterministic", "not"]) {
                self.next_token(); // type -> determinism
                let mut determinism = self.construct_node(NodeType::Unknown);
                if self.get_token(0).literal.to_uppercase() == "NOT" {
                    self.next_token(); // not -> deterministic
                    determinism.push_node("right", self.construct_node(NodeType::Unknown));
                }
                node.push_node("determinism", determinism);
            }
            self.next_token(); // determinism -> language, type -> language
            let mut language = self.construct_node(NodeType::Unknown);
            self.next_token(); // language -> js
            language.push_node("language", self.construct_node(NodeType::Unknown));
            node.push_node("language", language);
            if self.get_token(1).is("options") {
                self.next_token(); // js -> options
                let mut options = self.construct_node(NodeType::Unknown);
                self.next_token(); // options -> (
                let mut group = self.construct_node(NodeType::Unknown);
                if !self.get_token(1).is(")") {
                    self.next_token(); // ( -> expr
                    group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                }
                self.next_token(); // expr -> )
                group.push_node("rparen", self.construct_node(NodeType::Unknown));
                options.push_node("group", group);
                node.push_node("options", options);
            }
            self.next_token(); // js -> as, ) -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> javascript_code
            as_.push_node("expr", self.construct_node(NodeType::Unknown));
            node.push_node("as", as_);
        }
        if self.get_token(1).is(";") {
            self.next_token(); // ) -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        node
    }
    fn parse_create_procedure_statement(&mut self) -> Node {
        let mut create = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("or") {
            self.next_token(); // -> or
            let or_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> replace
            let replace = self.construct_node(NodeType::Unknown);
            create.push_node_vec("or_replace", vec![or_, replace]);
        }
        self.next_token(); // -> procedure
        create.push_node("what", self.construct_node(NodeType::Unknown));
        if self.get_token(1).is("if") {
            self.next_token(); // -> if
            let if_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> not
            let not = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> exists
            let exists = self.construct_node(NodeType::Unknown);
            create.push_node_vec("if_not_exists", vec![if_, not, exists]);
        }
        self.next_token(); // -> ident
        create.push_node("ident", self.parse_identifier());
        self.next_token(); // ident -> (
        let mut group = self.construct_node(NodeType::Unknown);
        let mut args = Vec::new();
        while !self.get_token(1).is(")") {
            self.next_token(); // ) -> arg, in
            let mut arg = self.construct_node(NodeType::Unknown);
            match self.get_token(2).literal.to_uppercase().as_str() {
                "TYPE" | "," | "<" | ")" => (),
                _ => {
                    self.next_token(); // -> ident
                    let mut ident = self.construct_node(NodeType::Unknown);
                    ident.push_node("in_out", arg);
                    arg = ident;
                }
            }
            self.next_token(); // arg -> type
            arg.push_node("type", self.parse_type(false));
            if self.get_token(1).is(",") {
                self.next_token(); // type -> ,
                arg.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            args.push(arg);
        }
        if args.len() > 0 {
            group.push_node_vec("args", args);
        }
        self.next_token(); // -> )
        group.push_node("rparen", self.construct_node(NodeType::Unknown));
        create.push_node("group", group);
        if self.get_token(1).is("options") {
            self.next_token(); // js -> options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.get_token(1).is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            create.push_node("options", options);
        }
        self.next_token(); // -> begin
        create.push_node("stmt", self.parse_begin_statement(false));
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        create
    }
    fn parse_alter_statement(&mut self) -> Node {
        let mut alter = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("materialized") {
            self.next_token(); // -> materialized
            alter.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table, view
        alter.push_node("what", self.construct_node(NodeType::Unknown));
        self.next_token(); // -> ident
        alter.push_node("ident", self.parse_identifier());
        if self.get_token(1).is("set") {
            self.next_token(); // -> set
            alter.push_node("set", self.construct_node(NodeType::Unknown));
            self.next_token(); // js -> options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.get_token(1).is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            alter.push_node("options", options);
        }
        let mut add_columns = Vec::new();
        while self.get_token(1).is("add") {
            self.next_token(); // -> add
            let mut add_column = self.construct_node(NodeType::Unknown);
            self.next_token();
            add_column.push_node("column", self.construct_node(NodeType::Unknown));
            if self.get_token(1).is("if") {
                self.next_token(); // -> if
                let if_ = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> not
                let not = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> exists
                let exists = self.construct_node(NodeType::Unknown);
                add_column.push_node_vec("if_not_exists", vec![if_, not, exists]);
            }
            self.next_token(); // -> column_name
            let mut column = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> schema
            column.push_node("type", self.parse_type(true));
            add_column.push_node("column_definition", column);
            if self.get_token(1).is(",") {
                self.next_token(); // -> ,
                add_column.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            add_columns.push(add_column);
        }
        if 0 < add_columns.len() {
            alter.push_node_vec("add_columns", add_columns);
        }
        let mut drop_columns = Vec::new();
        // TODO check when it becomes GA
        while self.get_token(1).is("drop") {
            self.next_token(); // -> drop
            let mut drop_column = self.construct_node(NodeType::Unknown);
            self.next_token();
            drop_column.push_node("column", self.construct_node(NodeType::Unknown));
            if self.get_token(1).is("if") {
                self.next_token(); // -> if
                let if_ = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> exists
                let exists = self.construct_node(NodeType::Unknown);
                drop_column.push_node_vec("if_exists", vec![if_, exists]);
            }
            self.next_token(); // -> column_name
            drop_column.push_node("column_name", self.construct_node(NodeType::Unknown));
            if self.get_token(1).is(",") {
                self.next_token(); // -> ,
                drop_column.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            drop_columns.push(drop_column);
        }
        if 0 < drop_columns.len() {
            alter.push_node_vec("drop_columns", drop_columns);
        }
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        alter
    }
    fn parse_drop_statement(&mut self) -> Node {
        let mut drop = self.construct_node(NodeType::Unknown);
        if self.get_token(1).is("external") {
            self.next_token(); // -> external
            drop.push_node("external", self.construct_node(NodeType::Unknown));
        }
        if self.get_token(1).is("materialized") {
            self.next_token(); // -> materialized
            drop.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table, view, function, procedure
        drop.push_node("what", self.construct_node(NodeType::Unknown));
        if self.get_token(1).is("if") {
            self.next_token(); // -> if
            let if_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> exists
            let exists = self.construct_node(NodeType::Unknown);
            drop.push_node_vec("if_exists", vec![if_, exists]);
        }
        self.next_token(); // -> ident
        drop.push_node("ident", self.parse_identifier());
        if self.get_token(1).in_(&vec!["cascade", "restrict"]) {
            self.next_token(); // -> cascade, restrict
            drop.push_node(
                "cascade_or_restrict",
                self.construct_node(NodeType::Unknown),
            );
        }
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            drop.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        drop
    }
    // ----- script -----
    fn parse_declare_statement(&mut self) -> Node {
        let mut declare = self.construct_node(NodeType::DeclareStatement);
        let mut idents = Vec::new();
        loop {
            self.next_token(); // -> ident
            if self.get_token(1).is(",") {
                let mut ident = self.parse_identifier();
                self.next_token(); // ident -> comma
                ident.push_node("comma", self.construct_node(NodeType::Symbol));
                idents.push(ident);
            } else {
                idents.push(self.parse_identifier());
                break;
            }
        }
        declare.push_node_vec("idents", idents);
        if !self.get_token(1).is("DEFAULT") {
            self.next_token(); // ident -> variable_type
            declare.push_node("variable_type", self.parse_type(false));
        }
        if self.get_token(1).is("DEFAULT") {
            self.next_token(); // -> DEFAULT
            let mut default = self.construct_node(NodeType::KeywordWithExpr);
            self.next_token(); // DEFAULT -> expr
            default.push_node("expr", self.parse_expr(999, &vec![";"], false));
            declare.push_node("default", default);
        }
        if self.get_token(1).is(";") {
            self.next_token();
            declare.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        declare
    }
    fn parse_set_statement(&mut self) -> Node {
        let mut set = self.construct_node(NodeType::SetStatement);
        self.next_token(); // set -> expr
        set.push_node("expr", self.parse_expr(999, &vec![";"], false));
        if self.get_token(1).is(";") {
            self.next_token();
            set.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        set
    }
    fn parse_execute_statement(&mut self) -> Node {
        let mut execute = self.construct_node(NodeType::ExecuteStatement);
        self.next_token(); // EXECUTE -> IMMEDIATE
        execute.push_node("immediate", self.construct_node(NodeType::Keyword));
        self.next_token(); // IMMEDIATE -> sql_expr
        execute.push_node(
            "sql_expr",
            self.parse_expr(999, &vec!["into", "using", ";"], false),
        );
        if self.get_token(1).is("INTO") {
            self.next_token(); // sql_expr -> INTO
            let mut into = self.construct_node(NodeType::KeywordWithExprs);
            let mut idents = Vec::new();
            loop {
                self.next_token(); // -> ident
                if self.get_token(1).is(",") {
                    let mut ident = self.parse_identifier();
                    self.next_token(); // ident -> ,
                    ident.push_node("comma", self.construct_node(NodeType::Symbol));
                    idents.push(ident);
                } else {
                    idents.push(self.parse_identifier());
                    break;
                }
            }
            into.push_node_vec("idents", idents);
            execute.push_node("into", into);
        }
        if self.get_token(1).is("USING") {
            self.next_token(); // -> using
            let mut using = self.construct_node(NodeType::KeywordWithExprs);
            self.next_token(); // using -> exprs
            using.push_node_vec("exprs", self.parse_exprs(&vec![";"], true));
            execute.push_node("using", using);
        }
        if self.get_token(1).is(";") {
            self.next_token();
            execute.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        execute
    }
    fn parse_begin_statement(&mut self, root: bool) -> Node {
        let mut begin = self.construct_node(NodeType::BeginStatement);
        let mut stmts = Vec::new();
        while !self.get_token(1).in_(&vec!["END", "EXCEPTION"]) {
            self.next_token(); // -> stmt
            stmts.push(self.parse_statement());
        }
        if 0 < stmts.len() {
            begin.push_node_vec("stmts", stmts);
        }
        if self.get_token(1).is("exception") {
            self.next_token(); // ; -> EXCEPTION
            let exception = self.construct_node(NodeType::Keyword);
            self.next_token(); // EXCEPTION -> WHEN
            let when = self.construct_node(NodeType::Keyword);
            self.next_token(); // WHEN -> ERROR
            let error = self.construct_node(NodeType::Keyword);
            begin.push_node_vec("exception_when_error", vec![exception, when, error]);
            self.next_token(); // ERROR -> THEN
            begin.push_node("then", self.parse_keyword_with_statements(&vec!["END"]));
        }
        self.next_token(); // -> end
        begin.push_node("end", self.construct_node(NodeType::Keyword));
        if self.get_token(1).is(";") && root {
            self.next_token(); // -> ;
            begin.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        begin
    }
    fn parse_if_statement(&mut self) -> Node {
        let mut if_ = self.construct_node(NodeType::IfStatement);
        self.next_token(); // -> condition
        if_.push_node("condition", self.parse_expr(999, &vec!["then"], false));

        self.next_token(); // -> THEN
        if_.push_node("then", self.parse_keyword_with_statements(&vec!["ELSEIF", "ELSE", "END"]));

        let mut elseifs = Vec::new();
        while self.get_token(1).is("ELSEIF") {
            self.next_token(); // -> ELSEIF
            let mut elseif = self.construct_node(NodeType::Keyword);
            self.next_token(); // -> condition
            elseif.push_node("condition", self.parse_expr(999, &vec!["then"], false));
            self.next_token(); // -> THEN
            elseif.push_node("then", self.parse_keyword_with_statements(&vec!["ELSEIF", "ELSE", "END"]));
            elseifs.push(elseif);
        }
        if 0 < elseifs.len() {
            if_.push_node_vec("elseifs", elseifs);
        }

        if self.get_token(1).is("ELSE") {
            self.next_token(); // -> ELSE
            if_.push_node("else", self.parse_keyword_with_statements(&vec!["END"]));
        }
        self.next_token(); // -> END
        let end = self.construct_node(NodeType::Keyword);
        self.next_token(); // -> IF
        if_.push_node_vec("end_if", vec![end, self.construct_node(NodeType::Keyword)]);
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            if_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        if_
    }
    fn parse_loop_statement(&mut self) -> Node {
        let mut loop_ = self.parse_keyword_with_statements(&vec!["END"]);
        loop_.node_type = NodeType::LoopStatement;
        self.next_token(); // -> END
        let end = self.construct_node(NodeType::Keyword);
        self.next_token(); // -> LOOP
        loop_.push_node_vec(
            "end_loop",
            vec![end, self.construct_node(NodeType::Keyword)],
        );
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            loop_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        loop_
    }
    fn parse_while_statement(&mut self) -> Node {
        let mut while_ = self.construct_node(NodeType::WhileStatement);
        self.next_token(); // -> condition
        while_.push_node("condition", self.parse_expr(999, &vec!["do"], false)); // NOTE do is not reserved
        self.next_token(); // -> DO
        while_.push_node("do", self.parse_keyword_with_statements(&vec!["END"]));
        self.next_token(); // -> END
        let end = self.construct_node(NodeType::Keyword);
        self.next_token(); // -> WHILE
        while_.push_node_vec(
            "end_while",
            vec![end, self.construct_node(NodeType::Keyword)],
        );
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            while_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        while_
    }
    fn parse_single_token_statement(&mut self) -> Node {
        let mut node = self.construct_node(NodeType::SingleTokenStatement);
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        node
    }
    fn parse_raise_statement(&mut self) -> Node {
        let mut raise = self.construct_node(NodeType::RaiseStatement);
        if self.get_token(1).is("using") {
            self.next_token(); // -> USING
            let mut using = self.construct_node(NodeType::KeywordWithExpr);
            self.next_token(); // -> MESSAGE
            // NOTE node_type of MESSAGE is not Keyword but Identifier
            using.push_node("expr", self.parse_expr(999, &vec![";"], false));
            raise.push_node("using", using);
        }
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            raise.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        raise
    }
    fn parse_call_statement(&mut self) -> Node {
        let mut call = self.construct_node(NodeType::CallStatement);
        self.next_token(); // -> procedure_name
        // NOTE node_type of procedure is CallingFunction
        let procedure = self.parse_expr(999, &vec![";"], false);
        call.push_node("procedure", procedure);
        if self.get_token(1).is(";") {
            self.next_token(); // -> ;
            call.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        call
    }
    fn parse_identifier(&mut self) -> Node {
        let mut left = self.construct_node(NodeType::Identifier);
        while self.get_token(1).is(".") {
            self.next_token(); // ident -> .
            let mut operator = self.construct_node(NodeType::Identifier);
            operator.push_node("left", left);
            self.next_token(); // . -> ident
            operator.push_node("right", self.construct_node(NodeType::Identifier));
            left = operator;
        }
        left
    }
    fn parse_table(&mut self, root: bool) -> Node {
        let mut left: Node;
        match self.get_token(0).literal.to_uppercase().as_str() {
            "(" => {
                let mut group = self.construct_node(NodeType::GroupedStatement);
                self.next_token(); // ( -> table
                if self.get_token(0).is("SELECT") {
                    group.push_node("stmt", self.parse_select_statement(true));
                } else {
                    group.node_type = NodeType::GroupedExpr;
                    group.push_node("expr", self.parse_table(true));
                }
                self.next_token(); // table -> )
                group.push_node("rparen", self.construct_node(NodeType::Symbol));
                if self.get_token(1).is("AS") {
                    self.next_token(); // -> AS
                    group.push_node("as", self.construct_node(NodeType::Keyword));
                }
                if self.get_token(1).is_identifier() {
                    self.next_token(); // -> ident
                    group.push_node("alias", self.construct_node(NodeType::Identifier));
                }
                left = group;
            }
            _ => {
                left = self.parse_expr(
                    999,
                    &vec![
                        "where", "group", "having", "limit", ";", "on", ",", "left", "right",
                        "cross", "inner", "join", ")",
                    ],
                    true,
                );
            }
        }
        if self.get_token(1).literal.to_uppercase() == "FOR" {
            self.next_token(); // TABLE -> FOR
            let mut for_ = self.construct_node(NodeType::ForSystemTimeAsOfClause);
            self.next_token(); // FOR -> SYSTEM_TIME
            let mut system_time_as_of = Vec::new();
            system_time_as_of.push(self.construct_node(NodeType::Keyword));
            self.next_token(); // SYSTEM_TIME -> AS
            system_time_as_of.push(self.construct_node(NodeType::Keyword));
            self.next_token(); // AS -> OF
            system_time_as_of.push(self.construct_node(NodeType::Keyword));
            for_.push_node_vec("system_time_as_of", system_time_as_of);
            self.next_token(); // OF -> timestamp
            for_.push_node(
                "expr",
                self.parse_expr(
                    999,
                    &vec![
                        "on", "left", "right", "cross", "inner", ",", "full", "join", "where",
                        "group", "having", ";", ")",
                    ],
                    false,
                ),
            );
            left.push_node("for_system_time_as_of", for_);
        }
        if self.get_token(1).is("tablesample") {
            // TODO check when it becomes GA
            self.next_token(); // -> TABLESAMPLE
            let mut tablesample = self.construct_node(NodeType::TableSampleCaluse);
            self.next_token(); // -> SYSTEM
            tablesample.push_node("system", self.construct_node(NodeType::Keyword));
            self.next_token(); // -> (
            let mut group = self.construct_node(NodeType::TableSampleRatio);
            self.next_token(); // -> expr
            group.push_node("expr", self.parse_expr(999, &vec!["percent"], false)); // NOTE percent is not reserved
            self.next_token(); // -> PERCENT
            group.push_node("percent", self.construct_node(NodeType::Keyword));
            self.next_token(); // -> )
            group.push_node("rparen", self.construct_node(NodeType::Symbol));
            tablesample.push_node("group", group);
            left.push_node("tablesample", tablesample);
        }
        if self.get_token(1).literal.to_uppercase() == "WITH" {
            self.next_token(); // UNNEST() -> WITH
            let with = self.construct_node(NodeType::Keyword);
            self.next_token(); // WITH -> OFFSET
            let offset = self.construct_node(NodeType::Keyword);
            if self.get_token(1).is("AS") {
                self.next_token(); // OFFSET -> AS
                left.push_node("offset_as", self.construct_node(NodeType::Keyword));
                self.next_token(); // AS -> alias
                left.push_node("offset_alias", self.construct_node(NodeType::Identifier));
            } else if self.get_token(1).is_identifier() {
                self.next_token(); // expr -> alias
                left.push_node("offset_alias", self.construct_node(NodeType::Identifier));
            }
            left.push_node_vec("with_offset", vec![with, offset]);
        }
        while self.get_token(1).in_(&vec![
            "left", "right", "cross", "inner", "full", "join", ",",
        ]) && root
        {
            self.next_token(); // table -> LEFT, RIGHT, INNER, CROSS, FULL, JOIN, ","
            let mut join = if self.get_token(0).in_(&vec!["join", ","]) {
                let join = self.construct_node(NodeType::JoinOperator);
                join
            } else {
                let mut type_ = self.construct_node(NodeType::Keyword);
                self.next_token(); // join_type -> OUTER, JOIN
                if self.get_token(0).is("OUTER") {
                    type_.push_node("outer", self.construct_node(NodeType::Keyword));
                    self.next_token(); // OUTER -> JOIN
                }
                let mut join = self.construct_node(NodeType::JoinOperator);
                join.push_node("join_type", type_);
                join
            };
            self.next_token(); // -> table
            let right = self.parse_table(false);
            if self.get_token(1).is("on") {
                self.next_token(); // `table` -> ON
                let mut on = self.construct_node(NodeType::OnClause);
                self.next_token(); // ON -> expr
                on.push_node(
                    "expr",
                    self.parse_expr(
                        999,
                        &vec![
                            "left", "right", "cross", "inner", ",", "full", "join", "where",
                            "group", "having", ";", ")",
                        ],
                        false,
                    ),
                );
                join.push_node("on", on);
            } else if self.get_token(1).is("using") {
                self.next_token(); // -> USING
                join.push_node(
                    "using",
                    self.parse_expr(
                        999,
                        &vec![
                            "left", "right", "cross", "inner", ",", "full", "join", "where",
                            "group", "having", ";", ")",
                        ],
                        false,
                    ),
                )
            }
            join.push_node("left", left);
            join.push_node("right", right);
            left = join;
        }
        left
    }
    fn parse_exprs(&mut self, until: &Vec<&str>, alias: bool) -> Vec<Node> {
        let mut exprs: Vec<Node> = Vec::new();
        // first expr
        let mut expr = self.parse_expr(999, until, alias);
        if self.get_token(1).is(",") {
            self.next_token(); // expr -> ,
            expr.push_node("comma", self.construct_node(NodeType::Symbol));
        }
        exprs.push(expr);
        // second expr and later
        while !self.get_token(1).in_(until) && !self.is_eof(1) {
            self.next_token();
            let mut expr = self.parse_expr(999, until, alias);
            if self.get_token(1).is(",") {
                self.next_token(); // expr -> ,
                expr.push_node("comma", self.construct_node(NodeType::Symbol));
                exprs.push(expr);
            } else {
                exprs.push(expr);
                break;
            }
        }
        exprs
    }
    fn parse_expr(&mut self, precedence: usize, until: &Vec<&str>, alias: bool) -> Node {
        // prefix or literal
        let mut left = self.construct_node(NodeType::Unknown);
        match self.get_token(0).literal.to_uppercase().as_str() {
            "*" => {
                left.node_type = NodeType::Symbol;
                match self.get_token(1).literal.to_uppercase().as_str() {
                    "REPLACE" => {
                        self.next_token(); // * -> REPLACE
                        let mut replace = self.construct_node(NodeType::KeywordWithGroupedExprs);
                        self.next_token(); // REPLACE -> (
                        let mut group = self.construct_node(NodeType::GroupedExprs);
                        let mut exprs = Vec::new();
                        while self.get_token(1).literal.as_str() != ")" {
                            self.next_token(); // ( -> expr, ident -> expr
                            let expr = self.parse_expr(999, &vec![")"], true);
                            exprs.push(expr);
                        }
                        self.next_token(); // ident -> )
                        group.push_node("rparen", self.construct_node(NodeType::Symbol));
                        group.push_node_vec("exprs", exprs);
                        replace.push_node("group", group);
                        left.push_node("replace", replace);
                    }
                    "EXCEPT" => {
                        self.next_token(); // * -> except
                        let mut except = self.construct_node(NodeType::KeywordWithGroupedExprs);
                        self.next_token(); // except -> (
                        let mut group = self.construct_node(NodeType::GroupedExprs);
                        self.next_token(); // ( -> exprs
                        group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                        self.next_token(); // exprs -> )
                        group.push_node("rparen", self.construct_node(NodeType::Symbol));
                        except.push_node("group", group);
                        left.push_node("except", except);
                    }
                    _ => (),
                }
            }
            // STRUCT
            "(" => {
                self.next_token(); // ( -> expr
                let mut exprs;
                if self.get_token(0).in_(&vec!["WITH", "SELECT"]) {
                    left.node_type = NodeType::GroupedStatement;
                    exprs = vec![self.parse_select_statement(true)];
                    left.push_node("stmt", exprs.pop().unwrap());
                } else {
                    exprs = self.parse_exprs(&vec![")"], true); // parse alias in the case of struct
                    if exprs.len() == 1 {
                        left.node_type = NodeType::GroupedExpr;
                        left.push_node("expr", exprs.pop().unwrap());
                    } else {
                        left.node_type = NodeType::StructLiteral;
                        left.push_node_vec("exprs", exprs);
                    }
                }
                self.next_token(); // expr -> )
                left.push_node("rparen", self.construct_node(NodeType::Symbol));
            }
            "STRUCT" => {
                let type_ = self.parse_type(false);
                self.next_token(); // STRUCT -> (, > -> (
                let mut struct_literal = self.parse_expr(999, &vec![], false);
                struct_literal.node_type = NodeType::StructLiteral; // in the case of `(1)`
                struct_literal.push_node("type", type_);
                left = struct_literal;
            }
            // ARRAY
            "[" => {
                left.node_type = NodeType::ArrayLiteral;
                self.next_token(); // [ -> exprs
                left.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                self.next_token(); // exprs -> ]
                left.push_node("rparen", self.construct_node(NodeType::Symbol));
            }
            "ARRAY" => {
                // when used as literal
                if !self.get_token(1).is("(") {
                    let type_ = self.parse_type(false);
                    self.next_token(); // > -> [
                    let mut arr = self.construct_node(NodeType::ArrayLiteral);
                    self.next_token(); // [ -> exprs
                    arr.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                    self.next_token(); // exprs -> ]
                    arr.push_node("rparen", self.construct_node(NodeType::Symbol));
                    arr.push_node("type", type_);
                    left = arr;
                }
            }
            "-" | "+" | "~" => {
                left.node_type = NodeType::UnaryOperator;
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until, false);
                left.push_node("right", right);
            }
            "DATE" | "TIME" | "DATETIME" | "TIMESTAMP" | "NUMERIC" | "BIGNUMERIC" | "DECIMAL"
            | "BIGDECIMAL" => {
                if self.get_token(1).is_string()
                    || self.get_token(1).in_(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    left.node_type = NodeType::UnaryOperator;
                    self.next_token(); // -> expr
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "INTERVAL" => {
                left.node_type = NodeType::Keyword;
                self.next_token(); // INTERVAL -> expr
                let right = self.parse_expr(999, &vec!["hour", "day", "month", "year"], false); // NOTE hour, month, year is not reserved
                self.next_token(); // expr -> HOUR
                left.push_node("date_part", self.construct_node(NodeType::Keyword));
                left.push_node("right", right);
            }
            "B" | "R" | "BR" | "RB" => {
                if self.get_token(1).is_string() {
                    self.next_token(); // R -> 'string'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                    left.node_type = NodeType::UnaryOperator;
                }
            }
            "SELECT" => {
                left = self.parse_select_statement(true);
            }
            "NOT" => {
                self.next_token(); // NOT -> boolean
                let right = self.parse_expr(110, until, false);
                left.push_node("right", right);
                left.node_type = NodeType::UnaryOperator;
            }
            "CASE" => {
                left.node_type = NodeType::CaseExpr;
                self.next_token(); // CASE -> expr, CASE -> when
                if !self.get_token(0).is("WHEN") {
                    left.push_node("expr", self.parse_expr(999, &vec!["WHEN"], false));
                    self.next_token(); // expr -> WHEN
                }
                let mut arms = Vec::new();
                while !self.get_token(0).is("ELSE") {
                    let mut arm = self.construct_node(NodeType::CaseArm);
                    self.next_token(); // WHEN -> expr
                    arm.push_node("expr", self.parse_expr(999, &vec!["then"], false));
                    self.next_token(); // expr ->THEN
                    arm.push_node("then", self.construct_node(NodeType::Keyword));
                    self.next_token(); // THEN -> result_expr
                    arm.push_node("result", self.parse_expr(999, &vec!["else", "when"], false));
                    self.next_token(); // result_expr -> ELSE, result_expr -> WHEN
                    arms.push(arm)
                }
                let mut else_ = self.construct_node(NodeType::CaseArm);
                self.next_token(); // ELSE -> result_expr
                else_.push_node("result", self.construct_node(NodeType::Unknown));
                arms.push(else_);
                left.push_node_vec("arms", arms);
                self.next_token(); // result_expr -> end
                left.push_node("end", self.construct_node(NodeType::Keyword));
            }
            _ => (),
        };
        // infix
        while !self.get_token(1).in_(until) && self.get_precedence(1) < precedence {
            // actually, `until` is not needed
            match self.get_token(1).literal.to_uppercase().as_str() {
                "(" => {
                    let func = self.get_token(0).literal.to_uppercase();
                    if func != "." {
                        // some functions (CAST, EXTRACT and so on) are reserved keywords
                        // but handle them as if they were identifiers
                        left.node_type = NodeType::Identifier;
                    }
                    self.next_token(); // ident -> (
                    let mut node = self.construct_node(NodeType::CallingFunction);
                    if self.get_token(1).is("distinct") {
                        self.next_token(); // ( -> DISTINCT
                        node.push_node("distinct", self.construct_node(NodeType::Keyword));
                    }
                    self.next_token(); // ( -> args
                    node.push_node("func", left);
                    if !self.get_token(0).is(")") {
                        match func.as_str() {
                            "CAST" => {
                                let cast_from = self.parse_expr(999, &vec![")", "AS"], false);
                                self.next_token(); // expr -> AS
                                let mut as_ = self.construct_node(NodeType::CastArgument);
                                self.next_token();
                                as_.push_node("cast_to", self.parse_type(false));
                                as_.push_node("cast_from", cast_from);
                                node.push_node_vec("args", vec![as_]);
                            }
                            "EXTRACT" => {
                                let mut datepart;
                                if self.get_token(1).is("(") {
                                    datepart =
                                        self.parse_expr(999, &vec![")", "from", "at"], false);
                                    datepart.node_type = NodeType::CallingDatePartFunction;
                                } else {
                                    datepart = self.construct_node(NodeType::Keyword);
                                }
                                self.next_token(); // expr -> FROM
                                let mut from = self.construct_node(NodeType::ExtractArgument);
                                self.next_token(); // FROM -> timestamp_expr
                                from.push_node("extract_datepart", datepart);
                                from.push_node(
                                    "extract_from",
                                    self.parse_expr(999, &vec!["at", ")"], false),
                                );
                                if self.get_token(1).is("AT") {
                                    let mut at_time_zone = Vec::new();
                                    self.next_token(); // timestamp_expr -> AT
                                    at_time_zone.push(self.construct_node(NodeType::Keyword));
                                    self.next_token(); // AT -> TIME
                                    at_time_zone.push(self.construct_node(NodeType::Keyword));
                                    self.next_token(); // TIME -> ZONE
                                    at_time_zone.push(self.construct_node(NodeType::Keyword));
                                    from.push_node_vec("at_time_zone", at_time_zone);
                                    self.next_token(); // ZONE -> 'UTC'
                                    from.push_node(
                                        "time_zone",
                                        self.parse_expr(999, &vec![")"], false),
                                    );
                                }
                                node.push_node_vec("args", vec![from]);
                            }
                            _ => {
                                node.push_node_vec(
                                    "args",
                                    self.parse_exprs(
                                        &vec![")", "respect", "ignore", "order", "limit"],
                                        false,
                                    ),
                                );
                            }
                        }
                        if self.get_token(1).in_(&vec!["respect", "ignore"]) {
                            self.next_token(); // expr -> RESPECT, IGNORE
                            let ignore_or_respect = self.construct_node(NodeType::Keyword);
                            self.next_token(); // RESPECT, IGNORE -> NULLS
                            node.push_node_vec(
                                "ignore_nulls",
                                vec![ignore_or_respect, self.construct_node(NodeType::Keyword)],
                            );
                        }
                        if self.get_token(1).is("order") {
                            self.next_token(); // expr -> ORDER
                            let mut orderby = self.construct_node(NodeType::XXXByExprs);
                            self.next_token(); // ORDER -> BY
                            orderby.push_node("by", self.construct_node(NodeType::Keyword));
                            self.next_token(); // BY -> expr
                            orderby.push_node_vec(
                                "exprs",
                                self.parse_exprs(&vec![")", "limit"], false),
                            );
                            node.push_node("orderby", orderby);
                        }
                        if self.get_token(1).is("LIMIT") {
                            self.next_token(); // -> LIMIT
                            let mut limit = self.construct_node(NodeType::KeywordWithExpr);
                            self.next_token();
                            limit.push_node("expr", self.parse_expr(999, &vec![")"], false));
                            node.push_node("limit", limit);
                        }
                        self.next_token(); // expr -> )
                    }
                    node.push_node("rparen", self.construct_node(NodeType::Symbol));
                    if self.get_token(1).is("over") {
                        self.next_token(); // ) -> OVER
                        let mut over = self.construct_node(NodeType::OverCaluse);
                        self.next_token(); // OVER -> (, OVER -> named_expr
                        over.push_node("window", self.parse_window_expr());
                        node.push_node("over", over);
                    }
                    left = node;
                }
                "[" => {
                    self.next_token(); // expr -> [
                    let mut node = self.construct_node(NodeType::ArrayAccessing);
                    node.push_node("left", left);
                    self.next_token(); // [ -> index_expr
                    node.push_node("right", self.parse_expr(999, &vec!["]"], false));
                    self.next_token(); // index_expr -> ]
                    node.push_node("rparen", self.construct_node(NodeType::Symbol));
                    left = node;
                }
                "." | "*" | "/" | "||" | "+" | "-" | "<<" | ">>" | "&" | "^" | "|" | "=" | "<"
                | ">" | "<=" | ">=" | "<>" | "!=" | "LIKE" | "IS" | "AND" | "OR" | "=>" => {
                    self.next_token(); // expr -> binary_operator
                    left = self.parse_binary_operator(left, until);
                }
                "BETWEEN" => {
                    self.next_token(); // expr -> BETWEEN
                    left = self.parse_between_operator(left, until);
                }
                "IN" => {
                    self.next_token(); // expr -> IN
                    left = self.parse_in_operator(left);
                }
                "NOT" => {
                    self.next_token(); // expr -> NOT
                    let not = self.construct_node(NodeType::Keyword);
                    self.next_token(); // NOT -> IN, LIKE, BETWEEN
                    if self.get_token(0).is("IN") {
                        left = self.parse_in_operator(left);
                        left.push_node("not", not);
                    } else if self.get_token(0).is("LIKE") {
                        left = self.parse_binary_operator(left, until);
                        left.push_node("not", not);
                    } else if self.get_token(0).is("BETWEEN") {
                        left = self.parse_between_operator(left, until);
                        left.push_node("not", not);
                    } else {
                        panic!(
                            "Expected `LIKE`, `BETWEEN` or `IN` but got: {:?}",
                            self.get_token(0)
                        );
                    }
                }
                _ => panic!(),
            }
        }
        // alias
        if alias {
            if self.get_token(1).is("AS") {
                self.next_token(); // expr -> AS
                left.push_node("as", self.construct_node(NodeType::Keyword));
                self.next_token(); // AS -> alias
                left.push_node("alias", self.construct_node(NodeType::Identifier));
            } else if self.get_token(1).is_identifier() {
                self.next_token(); // expr -> alias
                left.push_node("alias", self.construct_node(NodeType::Identifier));
            }
        }
        if self.get_token(1).in_(&vec!["ASC", "DESC"]) {
            self.next_token(); // expr -> ASC, DESC
            let order = self.construct_node(NodeType::Keyword);
            left.push_node("order", order);
        }
        if self.get_token(1).in_(&vec!["NULLS"]) {
            let mut nulls_first = Vec::new();
            self.next_token(); // ASC -> NULLS
            nulls_first.push(self.construct_node(NodeType::Keyword));
            self.next_token(); // NULLS -> FIRST, LAST
            nulls_first.push(self.construct_node(NodeType::Keyword));
            left.push_node_vec("null_order", nulls_first);
        }
        left
    }
    fn parse_between_operator(&mut self, left: Node, until: &Vec<&str>) -> Node {
        let precedence = self.get_precedence(0);
        let mut between = self.construct_node(NodeType::BetweenOperator);
        between.push_node("left", left);
        self.next_token(); // between -> expr1
        let mut exprs = Vec::new();
        exprs.push(self.parse_expr(precedence, &vec!["AND"], false));
        self.next_token(); // expr1 -> and
        between.push_node("and", self.construct_node(NodeType::Keyword));
        self.next_token(); // and -> expr2
        exprs.push(self.parse_expr(precedence, until, false));
        between.push_node_vec("right", exprs);
        between
    }
    fn parse_window_expr(&mut self) -> Node {
        if self.get_token(0).is("(") {
            let mut window = self.construct_node(NodeType::WindowSpecification);
            if self.get_token(1).is_identifier() {
                self.next_token(); // ( -> identifier
                window.push_node("name", self.construct_node(NodeType::Identifier));
            }
            if self.get_token(1).is("PARTITION") {
                self.next_token(); // ( -> PARTITION
                let mut partition = self.construct_node(NodeType::XXXByExprs);
                self.next_token(); // PARTITON -> BY
                partition.push_node("by", self.construct_node(NodeType::Keyword));
                self.next_token(); // BY -> exprs
                partition.push_node_vec("exprs", self.parse_exprs(&vec!["order", ")"], false));
                window.push_node("partitionby", partition);
            }
            if self.get_token(1).is("ORDER") {
                self.next_token(); // ( -> ORDER
                let mut order = self.construct_node(NodeType::XXXByExprs);
                self.next_token(); // ORDER -> BY
                order.push_node("by", self.construct_node(NodeType::Keyword));
                self.next_token(); // BY -> exprs
                order.push_node_vec(
                    "exprs",
                    self.parse_exprs(&vec!["rows", "range", ")"], false),
                );
                window.push_node("orderby", order);
            }
            if self.get_token(1).in_(&vec!["RANGE", "ROWS"]) {
                self.next_token(); // ( -> ROWS, expr -> ROWS
                let mut frame = self.construct_node(NodeType::WindowFrameClause);
                if self.get_token(1).is("BETWEEN") {
                    // frame_between
                    self.next_token(); // ROWS -> BETWEEN
                    frame.push_node("between", self.construct_node(NodeType::Keyword));
                    // start
                    self.next_token(); // BETWEEN -> UNBOUNDED, CURRENT
                    let mut frame_start = Vec::new();
                    if self.get_token(0).in_(&vec!["UNBOUNDED", "CURRENT"]) {
                        frame_start.push(self.construct_node(NodeType::Keyword));
                    } else {
                        frame_start.push(self.parse_expr(999, &vec!["PRECEDING"], false));
                    }
                    self.next_token(); // -> PRECEDING, ROW
                    frame_start.push(self.construct_node(NodeType::Keyword));
                    frame.push_node_vec("start", frame_start);
                    self.next_token(); // -> AND
                    frame.push_node("and", self.construct_node(NodeType::Keyword));
                    // end
                    self.next_token(); // AND -> UNBOUNDED, CURRENT
                    let mut frame_end = Vec::new();
                    if self.get_token(0).in_(&vec!["UNBOUNDED", "CURRENT"]) {
                        frame_end.push(self.construct_node(NodeType::Keyword));
                    } else {
                        frame_end.push(self.parse_expr(999, &vec!["FOLLOWING"], false));
                    }
                    self.next_token(); // -> FOLLOWING, ROW
                    frame_end.push(self.construct_node(NodeType::Keyword));
                    frame.push_node_vec("end", frame_end);
                } else {
                    // frame_start
                    if !self.get_token(1).is(")") {
                        self.next_token(); // ROWS -> UNBOUNDED, CURRENT
                        let mut frame_start = Vec::new();
                        if self.get_token(1).in_(&vec!["UNBOUNDED", "CURRENT"]) {
                            frame_start.push(self.construct_node(NodeType::Keyword));
                        } else {
                            frame_start.push(self.parse_expr(999, &vec!["PRECEDING"], false));
                        }
                        self.next_token(); // -> PRECEDING, ROW
                        frame.push_node_vec("start", frame_start);
                    }
                }
                window.push_node("frame", frame)
            }
            self.next_token(); // -> )
            window.push_node("rparen", self.construct_node(NodeType::Symbol));
            window
        } else {
            self.construct_node(NodeType::Identifier)
        }
    }
    fn parse_binary_operator(&mut self, left: Node, until: &Vec<&str>) -> Node {
        let precedence = self.get_precedence(0);
        let mut node = self.construct_node(NodeType::BinaryOperator);
        if self.get_token(1).is("NOT") {
            self.next_token(); // IS -> NOT
            node.push_node("not", self.construct_node(NodeType::Keyword));
        }
        self.next_token(); // binary_operator -> expr
        node.push_node("left", left);
        node.push_node("right", self.parse_expr(precedence, until, false));
        node.node_type = NodeType::BinaryOperator;
        node
    }
    fn parse_in_operator(&mut self, left: Node) -> Node {
        let mut node = self.construct_node(NodeType::InOperator);
        self.next_token(); // IN -> (
        node.push_node("left", left);
        let mut right = self.construct_node(NodeType::GroupedExprs);
        self.next_token(); // ( -> expr
        right.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
        self.next_token(); // expr -> )
        right.push_node("rparen", self.construct_node(NodeType::Symbol));
        node.push_node("right", right);
        node
    }
    fn parse_type(&mut self, schema: bool) -> Node {
        let mut res = match self.get_token(0).literal.to_uppercase().as_str() {
            "ARRAY" => {
                let mut res = self.construct_node(NodeType::Type);
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // ARRAY -> <
                    let mut type_ = self.construct_node(NodeType::GroupedType);
                    self.next_token(); // < -> type
                    type_.push_node("type", self.parse_type(schema));
                    self.next_token(); // type -> >
                    type_.push_node("rparen", self.construct_node(NodeType::Symbol));
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "STRUCT" => {
                let mut res = self.construct_node(NodeType::Type);
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // STRUCT -> <
                    let mut type_ = self.construct_node(NodeType::GroupedTypeDeclarations);
                    self.next_token(); // < -> type or ident
                    let mut type_declarations = Vec::new();
                    while !self.get_token(0).is(">") {
                        let mut type_declaration;
                        if !self.get_token(1).in_(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node(NodeType::TypeDeclaration);
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = Node::empty(NodeType::TypeDeclaration);
                        }
                        type_declaration.push_node("type", self.parse_type(schema));
                        self.next_token(); // type -> , or next_declaration
                        if self.get_token(0).is(",") {
                            type_declaration
                                .push_node("comma", self.construct_node(NodeType::Symbol));
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node(NodeType::Symbol));
                    type_.push_node_vec("declarations", type_declarations);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "ANY" => {
                let mut res = self.construct_node(NodeType::Type);
                self.next_token(); // ANY -> TYPE
                res.push_node("type", self.construct_node(NodeType::Keyword));
                res
            }
            _ => self.construct_node(NodeType::Type),
        };
        if self.get_token(1).is("NOT") && schema {
            self.next_token(); // -> NOT
            let not_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> null
            let null = self.construct_node(NodeType::Unknown);
            res.push_node_vec("not_null", vec![not_, null]);
        }
        if self.get_token(1).is("OPTIONS") && schema {
            self.construct_node(NodeType::Unknown); // -> options
            self.next_token(); // options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.get_token(1).is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            res.push_node("options", options);
        }
        res
    }
    fn get_precedence(&self, offset: usize) -> usize {
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators
        // 001... DATE, TIMESTAMP, r'', b'' (literal)
        // 101... [], ., ( (calling function. it's not mentioned in documentation)
        // 102... +, - , ~ (unary operator)
        // 103... *, / , ||
        // 104... +, - (binary operator)
        // 105... <<, >>
        // 106... & (bit operator)
        // 107... ^ (bit operator)
        // 108... | (bit operator)
        // 109... =, <, >, like, between, in
        // 110... NOT
        // 111... AND
        // 112... OR
        // 200... => (ST_GEOGFROMGEOJSON)
        // 999... LOWEST
        match self.get_token(offset).literal.to_uppercase().as_str() {
            // return precedence of BINARY operator
            "(" | "." | "[" => 101,
            "*" | "/" | "||" => 103,
            "+" | "-" => 104,
            "<<" | ">>" => 105,
            "&" => 106,
            "^" => 107,
            "|" => 108,
            "=" | "<" | ">" | "<=" | ">=" | "!=" | "<>" | "LIKE" | "BETWEEN" | "IN" | "IS" => 109,
            "NOT" => match self.get_token(offset + 1).literal.to_uppercase().as_str() {
                "IN" | "LIKE" | "BETWEEN" => 109,
                _ => panic!(
                    "Expected `IN`, `LIKE` or `BETWEEN` but got: {:?}",
                    self.get_token(offset + 1)
                ),
            },
            "AND" => 111,
            "OR" => 112,
            "=>" => 200,
            _ => 999,
        }
    }
}
