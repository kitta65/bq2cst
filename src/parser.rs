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
    fn construct_node(&self, node_type: NodeType) -> Node {
        let mut node = match node_type {
            NodeType::EOF => Node::empty(node_type),
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
    fn parse_statement(&mut self) -> Node {
        let node = match self.get_token(0).as_uppercase_str() {
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
                let mut target = self.get_token(offset).as_uppercase_str();
                loop {
                    match target {
                        "TEMP" => {
                            offset += 1;
                            target = self.get_token(offset).as_uppercase_str();
                        }
                        "TEMPORARY" => {
                            offset += 1;
                            target = self.get_token(offset).as_uppercase_str();
                        }
                        "OR" => {
                            offset += 2;
                            target = self.get_token(offset).as_uppercase_str();
                        }
                        _ => break,
                    }
                }
                // TODO separete functions
                match target {
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
    fn parse_select_statement(&mut self, root: bool) -> Node {
        if self.get_token(0).literal.as_str() == "(" {
            let mut node = self.construct_node(NodeType::Unknown);
            self.next_token(); // ( -> select
            node.push_node("stmt", self.parse_select_statement(true));
            self.next_token(); // stmt -> )
            node.push_node("rparen", self.construct_node(NodeType::Unknown));
            while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
                self.next_token(); // stmt -> union
                let mut operator = self.construct_node(NodeType::Unknown);
                self.next_token(); // union -> distinct
                operator.push_node("distinct", self.construct_node(NodeType::Unknown));
                operator.push_node("left", node);
                self.next_token(); // distinct -> stmt
                operator.push_node("right", self.parse_select_statement(false));
                node = operator;
            }
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node(NodeType::Symbol))
            }
            return node;
        }
        if self.get_token(0).literal.to_uppercase().as_str() == "WITH" {
            let mut with = self.construct_node(NodeType::Unknown);
            let mut queries = Vec::new();
            while self.get_token(1).literal.to_uppercase().as_str() != "SELECT" {
                self.next_token(); // with -> ident, ) -> ident
                let mut query = self.construct_node(NodeType::Unknown);
                self.next_token(); // ident -> as
                query.push_node("as", self.construct_node(NodeType::Unknown));
                self.next_token(); // as -> (
                query.push_node("stmt", self.parse_select_statement(true));
                if self.get_token(1).literal.as_str() == "," {
                    self.next_token(); // ) -> ,
                    query.push_node("comma", self.construct_node(NodeType::Unknown));
                }
                queries.push(query);
            }
            with.push_node_vec("queries", queries);
            self.next_token(); // ) -> select
            let mut node = self.parse_select_statement(true);
            node.push_node("with", with);
            return node;
        }
        // SELECT
        let mut node = self.construct_node(NodeType::SelectStatement);

        // as struct, as value
        if self.get_token(1).literal.to_uppercase().as_str() == "AS" {
            self.next_token(); // select -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> struct, value
            as_.push_node("struct_value", self.construct_node(NodeType::Unknown));
            node.push_node("as", as_);
        }

        // distinct
        if self.peek_token_in(&vec!["all", "distinct"]) {
            self.next_token(); // select -> all, distinct
            node.push_node("distinct", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> expr

        // columns
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
        // from
        if self.peek_token_is("FROM") {
            self.next_token(); // expr -> from
            let mut from = self.construct_node(NodeType::Unknown);
            self.next_token(); // from -> table
            from.push_node("expr", self.parse_table(true));
            node.push_node("from", from);
        }
        // where
        if self.peek_token_is("WHERE") {
            self.next_token(); // expr -> where
            let mut where_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // limit -> expr
            where_.push_node(
                "expr",
                self.parse_expr(
                    999,
                    &vec!["group", "having", ";", "order", ",", "window"],
                    false,
                ),
            );
            //self.next_token(); // parse_expr needs next_token()
            node.push_node("where", where_);
        }
        // group by
        if self.peek_token_is("GROUP") {
            self.next_token(); // expr -> group
            let mut groupby = self.construct_node(NodeType::Unknown);
            self.next_token(); // group -> by
            groupby.push_node("by", self.construct_node(NodeType::Unknown));
            self.next_token(); // by -> expr
            groupby.push_node_vec(
                "exprs",
                self.parse_exprs(&vec!["having", "limit", ";", "order", ")", "window"], false),
            );
            node.push_node("groupby", groupby);
        }
        // having
        if self.peek_token_is("HAVING") {
            self.next_token(); // expr -> having
            let mut having = self.construct_node(NodeType::Unknown);
            self.next_token(); // by -> expr
            having.push_node(
                "expr",
                self.parse_expr(999, &vec!["LIMIT", ";", "order", ")", "window"], false),
            );
            //self.next_token(); // expr -> limit
            node.push_node("having", having);
        }
        // window
        if self.peek_token_is("WINDOW") {
            self.next_token(); // table -> window
            let mut window = self.construct_node(NodeType::Unknown);
            let mut window_exprs = Vec::new();
            while self.get_token(1).is_identifier() {
                self.next_token(); // -> ident
                let mut window_expr = self.construct_node(NodeType::Unknown);
                self.next_token(); // ident -> as
                window_expr.push_node("as", self.construct_node(NodeType::Unknown));
                self.next_token(); // as -> (, as -> named_window
                window_expr.push_node("window", self.parse_window_expr());
                if self.peek_token_is(",") {
                    self.next_token(); // -> ,
                    window_expr.push_node("comma", self.construct_node(NodeType::Unknown));
                }
                window_exprs.push(window_expr);
            }
            window.push_node_vec("window_exprs", window_exprs);
            node.push_node("window", window);
        }
        // oeder by
        if self.peek_token_is("order") {
            self.next_token(); // expr -> order
            let mut order = self.construct_node(NodeType::Unknown);
            self.next_token(); // order -> by
            order.push_node("by", self.construct_node(NodeType::Unknown));
            self.next_token(); // by -> expr
            order.push_node_vec(
                "exprs",
                self.parse_exprs(&vec!["limit", ",", ";", ")"], false),
            );
            node.push_node("orderby", order);
        }
        // limit
        if self.peek_token_is("LIMIT") {
            self.next_token(); // expr -> limit
            let mut limit = self.construct_node(NodeType::Unknown);
            self.next_token(); // limit -> expr
            limit.push_node(
                "expr",
                self.parse_expr(999, &vec![";", ",", "offset", ")"], false),
            );
            if self.get_token(1).literal.to_uppercase().as_str() == "OFFSET" {
                self.next_token(); // expr -> offset
                let mut offset = self.construct_node(NodeType::Unknown);
                self.next_token(); // offset -> expr
                offset.push_node(
                    "expr",
                    self.parse_expr(999, &vec!["union", "intersect", "except", ";", ")"], false),
                );
                limit.push_node("offset", offset);
            }
            node.push_node("limit", limit);
        }
        // union
        while self.peek_token_in(&vec!["union", "intersect", "except"]) && root {
            self.next_token(); // stmt -> union
            let mut operator = self.construct_node(NodeType::Unknown);
            self.next_token(); // union -> distinct
            operator.push_node("distinct", self.construct_node(NodeType::Unknown));
            operator.push_node("left", node);
            self.next_token(); // distinct -> stmt
            operator.push_node("right", self.parse_select_statement(false));
            node = operator;
            if self.peek_token_is(";") && root {
                self.next_token(); // expr -> ;
                node.push_node("semicolon", self.construct_node(NodeType::Symbol))
            }
        }
        // ;
        if self.peek_token_is(";") && root {
            self.next_token(); // expr -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol))
        }
        node
    }
    fn parse_insert_statement(&mut self, root: bool) -> Node {
        let mut insert = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("into") {
            self.next_token(); // insert -> into
            insert.push_node("into", self.construct_node(NodeType::Unknown));
        }
        if !self.peek_token_in(&vec!["(", "value", "row"]) {
            self.next_token(); // insert -> identifier
            insert.push_node("target_name", self.parse_identifier());
        }
        if self.peek_token_is("(") {
            self.next_token(); // identifier -> (
            let mut group = self.construct_node(NodeType::Unknown);
            self.next_token(); // ( -> columns
            group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            self.next_token(); // columns -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            insert.push_node("columns", group);
        }
        if self.peek_token_is("values") {
            self.next_token(); // ) -> values
            let mut values = self.construct_node(NodeType::Unknown);
            let mut lparens = Vec::new();
            while self.peek_token_is("(") {
                self.next_token(); // vlaues -> (, ',' -> (
                let mut lparen = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> expr
                lparen.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                self.next_token(); // expr -> )
                lparen.push_node("rparen", self.construct_node(NodeType::Unknown));
                if self.peek_token_is(",") {
                    self.next_token(); // ) -> ,
                    lparen.push_node("comma", self.construct_node(NodeType::Unknown));
                }
                lparens.push(lparen);
            }
            values.push_node_vec("exprs", lparens);
            insert.push_node("input", values);
        } else if self.peek_token_is("row") {
            self.next_token(); // -> row
            insert.push_node("input", self.construct_node(NodeType::Unknown));
        } else {
            self.next_token(); // ) -> select
            insert.push_node("input", self.parse_select_statement(false));
        }
        if self.peek_token_is(";") && root {
            self.next_token(); // -> ;
            insert.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        insert
    }
    fn parse_delete_statement(&mut self) -> Node {
        let mut delete = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("from") {
            self.next_token(); // delete -> from
            delete.push_node("from", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> target_name
        let mut target_name = self.parse_identifier();
        if !self.peek_token_is("where") {
            target_name = self.parse_alias(target_name);
        }
        delete.push_node("target_name", target_name);
        self.next_token(); // target_name -> where, alias -> where
        let mut where_ = self.construct_node(NodeType::Unknown);
        self.next_token(); // where -> expr
        where_.push_node("expr", self.parse_expr(999, &vec![";"], false));
        delete.push_node("where", where_);
        if self.peek_token_is(";") {
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
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            truncate.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        truncate
    }
    fn parse_update_statement(&mut self, root: bool) -> Node {
        let mut update = self.construct_node(NodeType::Unknown);
        if !self.peek_token_is("set") {
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
        if self.peek_token_is("from") {
            self.next_token(); // exprs -> from
            let mut from = self.construct_node(NodeType::Unknown);
            self.next_token(); // from -> target_name
            from.push_node("expr", self.parse_table(true));
            update.push_node("from", from);
        }
        update.push_node("set", set);
        if self.peek_token_is("where") {
            self.next_token(); // exprs -> where
            let mut where_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // where -> expr
            where_.push_node("expr", self.parse_expr(999, &vec![";"], false));
            update.push_node("where", where_);
        }
        if self.peek_token_is(";") && root {
            self.next_token(); // -> ;
            update.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        update
    }
    fn parse_merge_statement(&mut self, root: bool) -> Node {
        let mut merge = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("into") {
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
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        self.next_token(); // -> on
        let mut on = self.construct_node(NodeType::Unknown);
        self.next_token(); // on -> expr
        on.push_node("expr", self.parse_expr(999, &vec!["when"], false));
        merge.push_node("on", on);
        let mut whens = Vec::new();
        while self.peek_token_is("when") {
            self.next_token(); // -> when
            let mut when = self.construct_node(NodeType::Unknown);
            if self.peek_token_is("not") {
                self.next_token(); // when -> not
                when.push_node("not", self.construct_node(NodeType::Unknown));
            }
            self.next_token(); // -> matched
            when.push_node("matched", self.construct_node(NodeType::Unknown));
            if self.peek_token_is("by") {
                self.next_token(); // -> by
                let by = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> target, source
                let target = self.construct_node(NodeType::Unknown);
                when.push_node_vec("by_target", vec![by, target]);
            }
            if self.peek_token_is("and") {
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
        if self.peek_token_is(";") && root {
            self.next_token(); // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        merge
    }
    fn parse_create_table_statement(&mut self) -> Node {
        let mut create = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("or") {
            self.next_token(); // -> or
            let or_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> replace
            let replace = self.construct_node(NodeType::Unknown);
            create.push_node_vec("or_replace", vec![or_, replace]);
        }
        if self.peek_token_is("materialized") {
            self.next_token();
            create.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        if self.peek_token_is("external") {
            self.next_token();
            create.push_node("external", self.construct_node(NodeType::Unknown));
        }
        if self.peek_token_in(&vec!["temp", "temporary"]) {
            self.next_token(); // -> temporary
            create.push_node("temp", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table
        create.push_node("what", self.construct_node(NodeType::Unknown));
        if self.peek_token_is("if") {
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
        if self.peek_token_is("(") {
            self.next_token(); // -> (
            let mut group = self.construct_node(NodeType::Unknown);
            let mut column_definitions = Vec::new();
            while !self.peek_token_is(")") {
                self.next_token(); // -> column_identifier
                let mut column = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> type
                column.push_node("type", self.parse_type(true));
                if self.peek_token_is(",") {
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
        if self.peek_token_is("partition") {
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
        if self.peek_token_is("cluster") {
            self.next_token(); // -> cluster
            let mut clusterby = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> by
            clusterby.push_node("by", self.construct_node(NodeType::Unknown));
            self.next_token(); // -> expr
            clusterby.push_node_vec("exprs", self.parse_exprs(&vec!["options", "as"], false));
            create.push_node("clusterby", clusterby);
        }
        if self.peek_token_is("with") {
            self.next_token(); // -> with
            let mut with = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> partition
            let partition = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> columns
            let columns = self.construct_node(NodeType::Unknown);
            with.push_node_vec("partition_columns", vec![partition, columns]);
            if self.peek_token_is("(") {
                self.next_token(); // -> "("
                let mut group = self.construct_node(NodeType::Unknown);
                let mut column_definitions = Vec::new();
                while !self.peek_token_is(")") {
                    self.next_token(); // -> column_identifier
                    let mut column = self.construct_node(NodeType::Unknown);
                    self.next_token(); // -> type
                    column.push_node("type", self.parse_type(true));
                    if self.peek_token_is(",") {
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
        if self.peek_token_is("options") {
            self.next_token(); // options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.peek_token_is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            create.push_node("options", options);
        }
        if self.peek_token_is("as") {
            self.next_token(); // -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> stmt
            as_.push_node("stmt", self.parse_select_statement(false));
            create.push_node("as", as_);
        }
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        create
    }
    fn parse_create_function_statement(&mut self) -> Node {
        let mut node = self.construct_node(NodeType::Unknown);
        if self.get_token(1).literal.to_uppercase().as_str() == "OR" {
            let mut or_replace = Vec::new();
            self.next_token(); // create -> or
            or_replace.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // or -> replace
            or_replace.push(self.construct_node(NodeType::Unknown));
            node.push_node_vec("or_replace", or_replace);
        }
        if self.peek_token_in(&vec!["temporary", "temp"]) {
            self.next_token(); // -> temp
            node.push_node("temp", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> function
        node.push_node("what", self.construct_node(NodeType::Unknown));
        if self.peek_token_in(&vec!["if"]) {
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
        while !self.peek_token_is(")") {
            self.next_token(); // ( -> arg, ',' -> arg
            let mut arg = self.construct_node(NodeType::Unknown);
            self.next_token(); // arg -> type
            arg.push_node("type", self.parse_type(false));
            if self.peek_token_is(",") {
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
        if self.peek_token_is("returns") {
            self.next_token(); // ) -> return
            let mut return_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // return -> type
            return_.push_node("type", self.parse_type(false));
            node.push_node("returns", return_);
        }
        if self.peek_token_is("as") {
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
            if self.peek_token_in(&vec!["deterministic", "not"]) {
                self.next_token(); // type -> determinism
                let mut determinism = self.construct_node(NodeType::Unknown);
                if self.get_token(0).literal.to_uppercase().as_str() == "NOT" {
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
            if self.peek_token_is("options") {
                self.next_token(); // js -> options
                let mut options = self.construct_node(NodeType::Unknown);
                self.next_token(); // options -> (
                let mut group = self.construct_node(NodeType::Unknown);
                if !self.peek_token_is(")") {
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
        if self.peek_token_is(";") {
            self.next_token(); // ) -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        node
    }
    fn parse_create_procedure_statement(&mut self) -> Node {
        let mut create = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("or") {
            self.next_token(); // -> or
            let or_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> replace
            let replace = self.construct_node(NodeType::Unknown);
            create.push_node_vec("or_replace", vec![or_, replace]);
        }
        self.next_token(); // -> procedure
        create.push_node("what", self.construct_node(NodeType::Unknown));
        if self.peek_token_is("if") {
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
        while !self.peek_token_is(")") {
            self.next_token(); // ) -> arg, in
            let mut arg = self.construct_node(NodeType::Unknown);
            match self.get_token(2).literal.to_uppercase().as_str() {
                "TYPE" => (),
                "," => (),
                "<" => (),
                ")" => (),
                _ => {
                    self.next_token(); // -> ident
                    let mut ident = self.construct_node(NodeType::Unknown);
                    ident.push_node("in_out", arg);
                    arg = ident;
                }
            }
            self.next_token(); // arg -> type
            arg.push_node("type", self.parse_type(false));
            if self.peek_token_is(",") {
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
        if self.peek_token_is("options") {
            self.next_token(); // js -> options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.peek_token_is(")") {
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
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        create
    }
    fn parse_alter_statement(&mut self) -> Node {
        let mut alter = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("materialized") {
            self.next_token(); // -> materialized
            alter.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table, view
        alter.push_node("what", self.construct_node(NodeType::Unknown));
        self.next_token(); // -> ident
        alter.push_node("ident", self.parse_identifier());
        if self.peek_token_is("set") {
            self.next_token(); // -> set
            alter.push_node("set", self.construct_node(NodeType::Unknown));
            self.next_token(); // js -> options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.peek_token_is(")") {
                self.next_token(); // ( -> expr
                group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
            }
            self.next_token(); // expr -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            options.push_node("group", group);
            alter.push_node("options", options);
        }
        let mut add_columns = Vec::new();
        while self.peek_token_is("add") {
            self.next_token(); // -> add
            let mut add_column = self.construct_node(NodeType::Unknown);
            self.next_token();
            add_column.push_node("column", self.construct_node(NodeType::Unknown));
            if self.peek_token_is("if") {
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
            if self.peek_token_is(",") {
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
        while self.peek_token_is("drop") {
            self.next_token(); // -> drop
            let mut drop_column = self.construct_node(NodeType::Unknown);
            self.next_token();
            drop_column.push_node("column", self.construct_node(NodeType::Unknown));
            if self.peek_token_is("if") {
                self.next_token(); // -> if
                let if_ = self.construct_node(NodeType::Unknown);
                self.next_token(); // -> exists
                let exists = self.construct_node(NodeType::Unknown);
                drop_column.push_node_vec("if_exists", vec![if_, exists]);
            }
            self.next_token(); // -> column_name
            drop_column.push_node("column_name", self.construct_node(NodeType::Unknown));
            if self.peek_token_is(",") {
                self.next_token(); // -> ,
                drop_column.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            drop_columns.push(drop_column);
        }
        if 0 < drop_columns.len() {
            alter.push_node_vec("drop_columns", drop_columns);
        }
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        alter
    }
    fn parse_drop_statement(&mut self) -> Node {
        let mut drop = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("external") {
            self.next_token(); // -> external
            drop.push_node("external", self.construct_node(NodeType::Unknown));
        }
        if self.peek_token_is("materialized") {
            self.next_token(); // -> materialized
            drop.push_node("materialized", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // -> table, view, function, procedure
        drop.push_node("what", self.construct_node(NodeType::Unknown));
        if self.peek_token_is("if") {
            self.next_token(); // -> if
            let if_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> exists
            let exists = self.construct_node(NodeType::Unknown);
            drop.push_node_vec("if_exists", vec![if_, exists]);
        }
        self.next_token(); // -> ident
        drop.push_node("ident", self.parse_identifier());
        if self.peek_token_in(&vec!["cascade", "restrict"]) {
            self.next_token(); // -> cascade, restrict
            drop.push_node(
                "cascade_or_restrict",
                self.construct_node(NodeType::Unknown),
            );
        }
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            drop.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        drop
    }
    fn parse_call_statement(&mut self) -> Node {
        let mut call = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> procedure_name
        call.push_node("expr", self.parse_expr(999, &vec![";"], false));
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            call.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        call
    }
    fn parse_raise_statement(&mut self) -> Node {
        let mut raise = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("using") {
            self.next_token(); // -> using
            let mut using = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> message
            using.push_node("expr", self.parse_expr(999, &vec![";"], false));
            raise.push_node("using", using);
        }
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            raise.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        raise
    }
    fn parse_loop_statement(&mut self) -> Node {
        let mut loop_ = self.construct_node(NodeType::Unknown);
        let mut stmts = Vec::new();
        while !self.peek_token_is("end") {
            self.next_token(); // -> stmt
            stmts.push(self.parse_statement());
        }
        if 0 < stmts.len() {
            loop_.push_node_vec("stmts", stmts);
        }
        self.next_token(); // -> end
        let end = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> loop
        loop_.push_node_vec(
            "end_loop",
            vec![end, self.construct_node(NodeType::Unknown)],
        );
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            loop_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        loop_
    }
    fn parse_while_statement(&mut self) -> Node {
        let mut while_ = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> boolean_expression
        while_.push_node("condition", self.parse_expr(999, &vec!["do"], false));
        self.next_token(); // -> do
        while_.push_node("do", self.construct_node(NodeType::Unknown));
        let mut stmts = Vec::new();
        while !self.peek_token_is("end") {
            self.next_token(); // -> stmt
            stmts.push(self.parse_statement());
        }
        if 0 < stmts.len() {
            while_.push_node_vec("stmts", stmts);
        }
        self.next_token(); // -> end
        let end = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> while
        while_.push_node_vec(
            "end_while",
            vec![end, self.construct_node(NodeType::Unknown)],
        );
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            while_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        while_
    }
    fn parse_single_token_statement(&mut self) -> Node {
        let mut node = self.construct_node(NodeType::Unknown);
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        node
    }
    fn parse_if_statement(&mut self) -> Node {
        let mut if_ = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> cond
        if_.push_node("condition", self.parse_expr(999, &vec!["then"], false));

        self.next_token(); // -> then
        let mut then = self.construct_node(NodeType::Unknown);
        let mut then_stmts = Vec::new();
        while !self.peek_token_in(&vec!["elseif", "else", "end"]) {
            self.next_token(); // -> stmt
            then_stmts.push(self.parse_statement());
        }
        if 0 < then_stmts.len() {
            then.push_node_vec("stmts", then_stmts);
        }
        if_.push_node("then", then);

        let mut elseifs = Vec::new();
        while self.peek_token_is("elseif") {
            self.next_token(); // -> elseif
            let mut elseif = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> condition
            elseif.push_node("condition", self.parse_expr(999, &vec!["then"], false));
            self.next_token(); // -> then
            let mut then = self.construct_node(NodeType::Unknown);
            let mut then_stmts = Vec::new();
            while !self.peek_token_in(&vec!["elseif", "else", "end"]) {
                self.next_token(); // -> stmt
                then_stmts.push(self.parse_statement());
            }
            if 0 < then_stmts.len() {
                then.push_node_vec("stmts", then_stmts);
            }
            elseif.push_node("then", then);
            elseifs.push(elseif);
        }
        if 0 < elseifs.len() {
            if_.push_node_vec("elseifs", elseifs);
        }

        if self.peek_token_is("else") {
            self.next_token(); // -> else
            let mut else_ = self.construct_node(NodeType::Unknown);
            let mut else_stmts = Vec::new();
            while !self.peek_token_is("end") {
                self.next_token(); // -> stmt
                else_stmts.push(self.parse_statement());
            }
            if 0 < else_stmts.len() {
                else_.push_node_vec("stmts", else_stmts);
            }
            if_.push_node("else", else_);
        }
        self.next_token(); // -> end
        let end = self.construct_node(NodeType::Unknown);
        self.next_token(); // -> if
        if_.push_node_vec("end_if", vec![end, self.construct_node(NodeType::Unknown)]);
        if self.peek_token_is(";") {
            self.next_token(); // -> ;
            if_.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        if_
    }
    fn parse_begin_statement(&mut self, root: bool) -> Node {
        let mut begin = self.construct_node(NodeType::Unknown);
        let mut stmts = Vec::new();
        while !self.peek_token_in(&vec!["end", "exception"]) {
            self.next_token(); // -> stmt
            stmts.push(self.parse_statement());
        }
        if 0 < stmts.len() {
            begin.push_node_vec("stmts", stmts);
        }
        if self.peek_token_is("exception") {
            self.next_token(); // ; -> exception
            let exception = self.construct_node(NodeType::Unknown);
            self.next_token(); // exception -> when
            let when = self.construct_node(NodeType::Unknown);
            self.next_token(); // exception -> error
            let error = self.construct_node(NodeType::Unknown);
            self.next_token(); // when -> then
            let then = self.construct_node(NodeType::Unknown);
            begin.push_node_vec(
                "exception_when_error_then",
                vec![exception, when, error, then],
            );
            let mut exception_stmts = Vec::new();
            while !self.peek_token_is("end") {
                self.next_token(); // -> stmt
                exception_stmts.push(self.parse_statement());
            }
            if 0 < exception_stmts.len() {
                begin.push_node_vec("exception_stmts", exception_stmts);
            }
        }
        self.next_token(); // -> end
        begin.push_node("end", self.construct_node(NodeType::Unknown));
        if self.peek_token_is(";") && root {
            self.next_token(); // -> ;
            begin.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        begin
    }
    fn parse_execute_statement(&mut self) -> Node {
        let mut execute = self.construct_node(NodeType::Unknown);
        self.next_token(); // execute -> immediate
        execute.push_node("immediate", self.construct_node(NodeType::Unknown));
        self.next_token(); // immediate -> sql_expr
        execute.push_node(
            "sql_expr",
            self.parse_expr(999, &vec!["into", "using", ";"], false),
        );
        if self.peek_token_is("into") {
            self.next_token(); // sql_expr -> into
            let mut into = self.construct_node(NodeType::Unknown);
            let mut idents = Vec::new();
            loop {
                self.next_token(); // -> ident
                if self.peek_token_is(",") {
                    let mut ident = self.parse_identifier();
                    self.next_token(); // ident -> ,
                    ident.push_node("comma", self.construct_node(NodeType::Unknown));
                    idents.push(ident);
                } else {
                    idents.push(self.parse_identifier());
                    break;
                }
            }
            into.push_node_vec("idents", idents);
            execute.push_node("into", into);
        }
        if self.peek_token_is("using") {
            self.next_token(); // -> using
            let mut using = self.construct_node(NodeType::Unknown);
            self.next_token(); // using -> exprs
            using.push_node_vec("exprs", self.parse_exprs(&vec![";"], true));
            execute.push_node("using", using);
        }
        if self.peek_token_is(";") {
            self.next_token();
            execute.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        execute
    }
    fn parse_set_statement(&mut self) -> Node {
        let mut set = self.construct_node(NodeType::Unknown);
        self.next_token(); // set -> expr
        set.push_node("expr", self.parse_expr(999, &vec![";"], false));
        if self.peek_token_is(";") {
            self.next_token();
            set.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        set
    }
    fn parse_declare_statement(&mut self) -> Node {
        let mut declare = self.construct_node(NodeType::Unknown);
        let mut idents = Vec::new();
        loop {
            self.next_token(); // -> ident
            if self.peek_token_is(",") {
                let mut ident = self.parse_identifier();
                self.next_token(); // ident -> comma
                ident.push_node("comma", self.construct_node(NodeType::Unknown));
                idents.push(ident);
            } else {
                idents.push(self.parse_identifier());
                break;
            }
        }
        declare.push_node_vec("idents", idents);
        if !self.peek_token_is("default") {
            self.next_token(); // ident -> variable_type
            declare.push_node("variable_type", self.parse_type(false));
        }
        if self.peek_token_is("default") {
            self.next_token(); // -> default
            let mut default = self.construct_node(NodeType::Unknown);
            self.next_token(); // default -> expr
            default.push_node("expr", self.parse_expr(999, &vec![";"], false));
            declare.push_node("default", default);
        }
        if self.peek_token_is(";") {
            self.next_token();
            declare.push_node("semicolon", self.construct_node(NodeType::Symbol));
        }
        declare
    }
    fn parse_identifier(&mut self) -> Node {
        let mut left = self.construct_node(NodeType::Unknown);
        while self.peek_token_is(".") {
            self.next_token(); // ident -> .
            let mut operator = self.construct_node(NodeType::Unknown);
            operator.push_node("left", left);
            self.next_token(); // . -> ident
            operator.push_node("right", self.construct_node(NodeType::Unknown));
            left = operator;
        }
        left
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
    fn parse_table(&mut self, root: bool) -> Node {
        let mut left: Node;
        match self.get_token(0).literal.to_uppercase().as_str() {
            "(" => {
                let mut group = self.construct_node(NodeType::Unknown);
                self.next_token(); // ( -> table
                group.push_node("expr", self.parse_table(true));
                self.next_token(); // table -> )
                group.push_node("rparen", self.construct_node(NodeType::Unknown));
                group = self.parse_alias(group);
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
        if self.get_token(1).literal.to_uppercase().as_str() == "FOR" {
            self.next_token(); // table -> for
            let mut for_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // for -> system_time
            let mut system_time_as_of = Vec::new();
            system_time_as_of.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // system_time -> as
            system_time_as_of.push(self.construct_node(NodeType::Unknown));
            self.next_token(); // as -> of
            system_time_as_of.push(self.construct_node(NodeType::Unknown));
            for_.push_node_vec("system_time_as_of", system_time_as_of);
            self.next_token(); // of -> timestamp
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
        if self.peek_token_is("tablesample") {
            // TODO check when it becomes GA
            self.next_token(); // -> tablesample
            let mut tablesample = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> system
            tablesample.push_node("system", self.construct_node(NodeType::Unknown));
            self.next_token(); // -> (
            let mut group = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> expr
            group.push_node("expr", self.parse_expr(999, &vec!["percent"], false));
            self.next_token(); // -> percent
            group.push_node("percent", self.construct_node(NodeType::Unknown));
            self.next_token(); // -> )
            group.push_node("rparen", self.construct_node(NodeType::Unknown));
            tablesample.push_node("group", group);
            left.push_node("tablesample", tablesample);
        }
        if self.get_token(1).literal.to_uppercase().as_str() == "WITH" {
            self.next_token(); // unnest() -> with
            let mut with = self.construct_node(NodeType::Unknown);
            self.next_token(); // with -> offset
            with.push_node(
                "unnest_offset",
                self.parse_expr(
                    999,
                    &vec![
                        "on", "left", "right", "cross", "inner", ",", "full", "join", "where",
                        "group", "having", ";", ")",
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
                let join = self.construct_node(NodeType::Unknown);
                join
            } else {
                let mut type_ = self.construct_node(NodeType::Unknown);
                self.next_token(); // type -> outer, type -> join
                if self.cur_token_is("outer") {
                    type_.push_node("outer", self.construct_node(NodeType::Unknown));
                    self.next_token(); // outer -> join
                }
                let mut join = self.construct_node(NodeType::Unknown);
                join.push_node("join_type", type_);
                join
            };
            self.next_token(); // -> table
            let right = self.parse_table(false);
            if self.peek_token_is("on") {
                self.next_token(); // `table` -> on
                let mut on = self.construct_node(NodeType::Unknown);
                self.next_token(); // on -> expr
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
            } else if self.peek_token_is("using") {
                self.next_token(); // -> using
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
        if self.peek_token_is(",") {
            self.next_token(); // expr -> ,
            expr.push_node("comma", self.construct_node(NodeType::Unknown));
        }
        exprs.push(expr);
        // second expr and later
        while !self.peek_token_in(until) && !self.is_eof(1) {
            self.next_token();
            let mut expr = self.parse_expr(999, until, alias);
            if self.peek_token_is(",") {
                self.next_token(); // expr -> ,
                expr.push_node("comma", self.construct_node(NodeType::Unknown));
            }
            exprs.push(expr);
        }
        exprs
    }
    fn parse_expr(&mut self, precedence: usize, until: &Vec<&str>, alias: bool) -> Node {
        // prefix or literal
        let mut left = self.construct_node(NodeType::Unknown);
        match self.get_token(0).literal.to_uppercase().as_str() {
            "*" => {
                match self.get_token(1).literal.to_uppercase().as_str() {
                    "REPLACE" => {
                        self.next_token(); // * -> replace
                        let mut replace = self.construct_node(NodeType::Unknown);
                        self.next_token(); // replace -> (
                        let mut group = self.construct_node(NodeType::Unknown);
                        let mut exprs = Vec::new();
                        while self.get_token(1).literal.as_str() != ")" {
                            self.next_token(); // ( -> expr, ident -> expr
                            let expr = self.parse_expr(999, &vec![")"], true);
                            exprs.push(expr);
                        }
                        self.next_token(); // ident -> )
                        group.push_node("rparen", self.construct_node(NodeType::Unknown));
                        group.push_node_vec("exprs", exprs);
                        replace.push_node("group", group);
                        left.push_node("replace", replace);
                    }
                    "EXCEPT" => {
                        self.next_token(); // * -> except
                        let mut except = self.construct_node(NodeType::Unknown);
                        self.next_token(); // except -> (
                        let mut group = self.construct_node(NodeType::Unknown);
                        self.next_token(); // ( -> exprs
                        group.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                        self.next_token(); // exprs -> )
                        group.push_node("rparen", self.construct_node(NodeType::Unknown));
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
                self.next_token(); // expr -> )
                left.push_node("rparen", self.construct_node(NodeType::Unknown));
            }
            "-" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until, false);
                left.push_node("right", right);
            }
            "+" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until, false);
                left.push_node("right", right);
            }
            "~" => {
                self.next_token(); // - -> expr
                let right = self.parse_expr(102, until, false);
                left.push_node("right", right);
            }
            "[" => {
                self.next_token(); // [ -> exprs
                left.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                self.next_token(); // exprs -> ]
                left.push_node("rparen", self.construct_node(NodeType::Unknown));
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
            "TIME" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // time -> 'yyyy-mm-dd'
                    let right = self.parse_expr(001, until, false);
                    left.push_node("right", right);
                }
            }
            "DATETIME" => {
                if self.get_token(1).is_string()
                    || self.peek_token_in(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2).is_string()
                {
                    self.next_token(); // datetime -> 'yyyy-mm-dd'
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
                left.push_node("date_part", self.construct_node(NodeType::Unknown));
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
                    let mut type_ = self.construct_node(NodeType::Unknown);
                    self.next_token(); // < -> type
                    type_.push_node("type", self.parse_type(false));
                    self.next_token(); // type -> >
                    type_.push_node("rparen", self.construct_node(NodeType::Unknown));
                    left.push_node("type_declaration", type_);
                }
                if !self.peek_token_is("(") {
                    self.next_token(); // ARRAY -> [, > -> [
                    let mut right = self.construct_node(NodeType::Unknown);
                    self.next_token(); // [ -> exprs
                    right.push_node_vec("exprs", self.parse_exprs(&vec!["]"], false));
                    self.next_token(); // exprs -> ]
                    right.push_node("rparen", self.construct_node(NodeType::Unknown));
                    left.push_node("right", right);
                }
            }
            "STRUCT" => {
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // struct -> <
                    let mut type_ = self.construct_node(NodeType::Unknown);
                    let mut type_declarations = Vec::new();
                    self.next_token(); // < -> ident or type
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node(NodeType::Unknown);
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = Node::empty(NodeType::Unknown);
                        }
                        type_declaration.push_node("type", self.parse_type(false));
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration
                                .push_node("comma", self.construct_node(NodeType::Unknown));
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node(NodeType::Unknown));
                    type_.push_node_vec("declarations", type_declarations);
                    left.push_node("type_declaration", type_);
                }
                self.next_token(); // struct -> (, > -> (
                let mut right = self.construct_node(NodeType::Unknown);
                self.next_token(); // ( -> exprs
                right.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
                self.next_token(); // exprs -> )
                right.push_node("rparen", self.construct_node(NodeType::Unknown));
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
                    let mut arm = self.construct_node(NodeType::Unknown);
                    self.next_token(); // when -> expr
                    arm.push_node("expr", self.parse_expr(999, &vec!["then"], false));
                    self.next_token(); // expr -> then
                    arm.push_node("then", self.construct_node(NodeType::Unknown));
                    self.next_token(); // then -> result_expr
                    arm.push_node("result", self.parse_expr(999, &vec!["else", "when"], false));
                    self.next_token(); // result_expr -> else, result_expr -> when
                    arms.push(arm)
                }
                let mut arm = self.construct_node(NodeType::Unknown);
                self.next_token(); // else -> result_expr
                arm.push_node("result", self.construct_node(NodeType::Unknown));
                arms.push(arm);
                left.push_node_vec("arms", arms);
                self.next_token(); // result_expr -> end
                left.push_node("end", self.construct_node(NodeType::Unknown));
            }
            _ => (),
        };
        // infix
        while !self.peek_token_in(until) && self.get_precedence(1) < precedence {
            // actually, until is not needed
            match self.get_token(1).literal.to_uppercase().as_str() {
                "(" => {
                    let func = self.get_token(0).literal.to_uppercase();
                    self.next_token(); // ident -> (
                    let mut node = self.construct_node(NodeType::Unknown);
                    self.next_token(); // ( -> args
                    if self.cur_token_is("distinct") {
                        node.push_node("distinct", self.construct_node(NodeType::Unknown));
                        self.next_token(); // distinct -> args
                    }
                    node.push_node("func", left);
                    if !self.cur_token_is(")") {
                        match func.as_str() {
                            "CAST" => {
                                let cast_from = self.parse_expr(999, &vec![")", "as"], false);
                                self.next_token(); // expr -> as
                                let mut as_ = self.construct_node(NodeType::Unknown);
                                self.next_token();
                                as_.push_node("cast_to", self.parse_type(false));
                                as_.push_node("cast_from", cast_from);
                                node.push_node_vec("args", vec![as_]);
                            }
                            "EXTRACT" => {
                                let datepart =
                                    self.parse_expr(999, &vec![")", "from", "at"], false);
                                self.next_token(); // expr -> from
                                let mut from = self.construct_node(NodeType::Unknown);
                                self.next_token(); // from -> timestamp_expr
                                from.push_node("extract_datepart", datepart);
                                from.push_node(
                                    "extract_from",
                                    self.parse_expr(999, &vec!["at", ")"], false),
                                );
                                if self.peek_token_is("at") {
                                    self.next_token(); // timestamp_expr -> at
                                    let mut at = self.construct_node(NodeType::Unknown);
                                    self.next_token(); // at -> time
                                    let mut time_zone = Vec::new();
                                    time_zone.push(self.construct_node(NodeType::Unknown));
                                    self.next_token(); // time -> zone
                                    time_zone.push(self.construct_node(NodeType::Unknown));
                                    at.push_node_vec("time_zone", time_zone);
                                    self.next_token(); // zone -> 'UTC'
                                    at.push_node("expr", self.parse_expr(999, &vec![")"], false));
                                    from.push_node("at", at);
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
                        if self.peek_token_in(&vec!["respect", "ignore"]) {
                            self.next_token(); // expr -> respect, ignore
                            let mut ignore_nulls = self.construct_node(NodeType::Unknown);
                            self.next_token(); // respect, ignore -> nulls
                            ignore_nulls.push_node("nulls", self.construct_node(NodeType::Unknown));
                            node.push_node("ignore_nulls", ignore_nulls);
                        }
                        if self.peek_token_is("order") {
                            self.next_token(); // expr -> order
                            let mut orderby = self.construct_node(NodeType::Unknown);
                            self.next_token(); // order -> by
                            orderby.push_node("by", self.construct_node(NodeType::Unknown));
                            self.next_token(); // by -> expr
                            orderby.push_node_vec(
                                "exprs",
                                self.parse_exprs(&vec![")", "limit"], false),
                            );
                            node.push_node("orderby", orderby);
                        }
                        if self.peek_token_is("limit") {
                            self.next_token(); // -> limit
                            let mut limit = self.construct_node(NodeType::Unknown);
                            self.next_token();
                            limit.push_node("expr", self.parse_expr(999, &vec![")"], false));
                            node.push_node("limit", limit);
                        }
                        self.next_token(); // expr -> )
                    }
                    node.push_node("rparen", self.construct_node(NodeType::Unknown));
                    if self.peek_token_is("over") {
                        self.next_token(); // ) -> over
                        let mut over = self.construct_node(NodeType::Unknown);
                        self.next_token(); // over -> (, over -> named_expr
                        over.push_node("window", self.parse_window_expr());
                        node.push_node("over", over);
                    }
                    left = node;
                }
                "." => {
                    self.next_token(); // expr -> .
                    left = self.parse_binary_operator(left, until);
                }
                "[" => {
                    self.next_token(); // expr -> [
                    let mut node = self.construct_node(NodeType::Unknown);
                    node.push_node("left", left);
                    //let precedence = self.get_precedence(0);
                    self.next_token(); // [ -> index_expr
                    node.push_node("right", self.parse_expr(999, &vec!["]"], false));
                    self.next_token(); // index_expr -> ]
                    node.push_node("rparen", self.construct_node(NodeType::Unknown));
                    left = node;
                }
                "*" => {
                    self.next_token(); // expr -> *
                    left = self.parse_binary_operator(left, until);
                }
                "/" => {
                    self.next_token(); // expr -> /
                    left = self.parse_binary_operator(left, until);
                }
                "||" => {
                    self.next_token(); // expr -> ||
                    left = self.parse_binary_operator(left, until);
                }
                "+" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                "-" => {
                    self.next_token(); // expr -> +
                    left = self.parse_binary_operator(left, until);
                }
                "<<" => {
                    self.next_token(); // expr -> <<
                    left = self.parse_binary_operator(left, until);
                }
                ">>" => {
                    self.next_token(); // expr -> >>
                    left = self.parse_binary_operator(left, until);
                }
                "&" => {
                    self.next_token(); // expr -> &
                    left = self.parse_binary_operator(left, until);
                }
                "^" => {
                    self.next_token(); // expr -> ^
                    left = self.parse_binary_operator(left, until);
                }
                "|" => {
                    self.next_token(); // expr -> |
                    left = self.parse_binary_operator(left, until);
                }
                "=" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "<" => {
                    self.next_token(); // expr -> <
                    left = self.parse_binary_operator(left, until);
                }
                ">" => {
                    self.next_token(); // expr -> >
                    left = self.parse_binary_operator(left, until);
                }
                "<=" => {
                    self.next_token(); // expr -> <=
                    left = self.parse_binary_operator(left, until);
                }
                ">=" => {
                    self.next_token(); // expr -> >=
                    left = self.parse_binary_operator(left, until);
                }
                "!=" => {
                    self.next_token(); // expr -> >=
                    left = self.parse_binary_operator(left, until);
                }
                "<>" => {
                    self.next_token(); // expr -> >=
                    left = self.parse_binary_operator(left, until);
                }
                "LIKE" => {
                    self.next_token(); // expr -> like
                    left = self.parse_binary_operator(left, until);
                }
                "BETWEEN" => {
                    self.next_token(); // expr -> between
                    let precedence = self.get_precedence(0);
                    let mut between = self.construct_node(NodeType::Unknown);
                    between.push_node("left", left);
                    left = between;
                    self.next_token(); // between -> expr1
                    let mut exprs = Vec::new();
                    exprs.push(self.parse_expr(precedence, until, false));
                    self.next_token(); // expr1 -> and
                    left.push_node("and", self.construct_node(NodeType::Unknown));
                    self.next_token(); // and -> expr2
                    exprs.push(self.parse_expr(precedence, until, false));
                    left.push_node_vec("right", exprs);
                }
                "IN" => {
                    self.next_token(); // expr -> in
                    left = self.parse_in_operator(left);
                }
                "IS" => {
                    self.next_token(); // expr -> in
                    left = self.parse_binary_operator(left, until);
                }
                "NOT" => {
                    self.next_token(); // expr -> not
                    let not = self.construct_node(NodeType::Unknown);
                    self.next_token(); // not -> in, like
                    if self.cur_token_is("in") {
                        left = self.parse_in_operator(left);
                        left.push_node("not", not);
                    } else if self.cur_token_is("like") {
                        left = self.parse_binary_operator(left, until);
                        left.push_node("not", not);
                    } else if self.cur_token_is("between") {
                        let precedence = self.get_precedence(0);
                        let mut between = self.construct_node(NodeType::Unknown);
                        between.push_node("left", left);
                        left = between;
                        self.next_token(); // between -> expr1
                        let mut exprs = Vec::new();
                        exprs.push(self.parse_expr(precedence, until, false));
                        self.next_token(); // expr1 -> and
                        left.push_node("and", self.construct_node(NodeType::Unknown));
                        self.next_token(); // and -> expr2
                        exprs.push(self.parse_expr(precedence, until, false));
                        left.push_node_vec("right", exprs);
                        left.push_node("not", not);
                    } else {
                        panic!();
                    }
                }
                "AND" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "OR" => {
                    self.next_token(); // expr -> =
                    left = self.parse_binary_operator(left, until);
                }
                "=>" => {
                    self.next_token(); // expr -> <=
                    left = self.parse_binary_operator(left, until);
                }
                _ => panic!(),
            }
        }
        // alias
        if self.peek_token_is("as") && precedence == 999 && alias {
            self.next_token(); // expr -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> alias
            as_.push_node("alias", self.construct_node(NodeType::Unknown));
            left.push_node("as", as_);
        }
        if self.get_token(1).is_identifier() && !self.is_eof(1) && precedence == 999 && alias {
            self.next_token(); // expr -> alias
            let mut as_ = Node::empty(NodeType::Unknown);
            as_.push_node("alias", self.construct_node(NodeType::Unknown));
            left.push_node("as", as_);
        }
        if self.peek_token_in(&vec!["asc", "desc"]) {
            self.next_token(); // expr -> asc
            let order = self.construct_node(NodeType::Unknown);
            left.push_node("order", order);
        }
        if self.peek_token_in(&vec!["nulls"]) {
            self.next_token(); // asc -> nulls, expr -> nulls
            let mut nulls = self.construct_node(NodeType::Unknown);
            self.next_token(); // nulls -> first, last
            nulls.push_node("first", self.construct_node(NodeType::Unknown));
            left.push_node("null_order", nulls);
        }
        left
    }
    fn parse_window_expr(&mut self) -> Node {
        if self.cur_token_is("(") {
            let mut window = self.construct_node(NodeType::Unknown);
            if self.get_token(1).is_identifier() {
                self.next_token(); // ( -> identifier
                window.push_node("name", self.construct_node(NodeType::Unknown));
            }
            if self.peek_token_is("partition") {
                self.next_token(); // ( -> partition, order, frame
                let mut partition = self.construct_node(NodeType::Unknown);
                self.next_token(); // partition -> by
                partition.push_node("by", self.construct_node(NodeType::Unknown));
                self.next_token(); // by -> exprs
                partition.push_node_vec("exprs", self.parse_exprs(&vec!["order", ")"], false));
                window.push_node("partitionby", partition);
            }
            if self.peek_token_is("order") {
                self.next_token(); // ( -> order, expr -> order
                let mut order = self.construct_node(NodeType::Unknown);
                self.next_token(); // order -> by
                order.push_node("by", self.construct_node(NodeType::Unknown));
                self.next_token(); // by -> exprs
                order.push_node_vec(
                    "exprs",
                    self.parse_exprs(&vec!["rows", "range", ")"], false),
                );
                window.push_node("orderby", order);
            }
            if self.peek_token_in(&vec!["range", "rows"]) {
                self.next_token(); // ( -> rows, expr -> rows
                let mut frame = self.construct_node(NodeType::Unknown);
                if self.peek_token_is("between") {
                    // frame between
                    self.next_token(); // rows -> between
                    frame.push_node("between", self.construct_node(NodeType::Unknown));
                    self.next_token(); // between -> expr
                    let mut start = self.parse_expr(999, &vec!["preceding"], false);
                    self.next_token(); // expr -> preceding
                    start.push_node("preceding", self.construct_node(NodeType::Unknown));
                    frame.push_node("start", start);
                    self.next_token(); // preceding -> and
                    frame.push_node("and", self.construct_node(NodeType::Unknown));
                    self.next_token(); // and -> expr
                    let mut end = self.parse_expr(999, &vec![")"], false);
                    self.next_token(); // expr -> trailing
                    end.push_node("trailing", self.construct_node(NodeType::Unknown));
                    frame.push_node("end", end);
                } else {
                    // frame start
                    if !self.peek_token_is(")") {
                        self.next_token(); // rows -> expr
                        let mut start = self.parse_expr(999, &vec!["preceding"], false);
                        self.next_token(); // expr -> preceding, row
                        start.push_node("preceding", self.construct_node(NodeType::Unknown));
                        frame.push_node("start", start);
                    }
                }
                window.push_node("frame", frame)
            }
            self.next_token(); // -> )
            window.push_node("rparen", self.construct_node(NodeType::Unknown));
            window
        } else {
            self.construct_node(NodeType::Unknown)
        }
    }
    fn parse_alias(&mut self, node: Node) -> Node {
        let mut node = node.clone();
        if self.peek_token_is("as") {
            self.next_token(); // expr -> as
            let mut as_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // as -> alias
            as_.push_node("alias", self.construct_node(NodeType::Unknown));
            node.push_node("as", as_);
        } else if self.get_token(1).is_identifier() && !self.is_eof(1) {
            self.next_token(); // expr -> alias
            let mut as_ = Node::empty(NodeType::Unknown);
            as_.push_node("alias", self.construct_node(NodeType::Unknown));
            node.push_node("as", as_);
        }
        node
    }
    fn parse_binary_operator(&mut self, left: Node, until: &Vec<&str>) -> Node {
        let precedence = self.get_precedence(0);
        let mut node = self.construct_node(NodeType::Unknown);
        if self.peek_token_is("not") {
            self.next_token(); // is -> not
            node.push_node("not", self.construct_node(NodeType::Unknown));
        }
        self.next_token(); // binary_operator -> expr
        node.push_node("left", left);
        node.push_node("right", self.parse_expr(precedence, until, false));
        node
    }
    fn parse_in_operator(&mut self, left: Node) -> Node {
        let mut node = self.construct_node(NodeType::Unknown);
        self.next_token(); // in -> (
        node.push_node("left", left);
        let mut right = self.construct_node(NodeType::Unknown);
        self.next_token(); // ( -> expr
        right.push_node_vec("exprs", self.parse_exprs(&vec![")"], false));
        self.next_token(); // expr -> )
        right.push_node("rparen", self.construct_node(NodeType::Unknown));
        node.push_node("right", right);
        node
    }
    fn peek_token_is(&self, s: &str) -> bool {
        self.get_token(1).literal.to_uppercase() == s.to_uppercase()
    }
    fn cur_token_is(&self, s: &str) -> bool {
        self.get_token(0).literal.to_uppercase() == s.to_uppercase()
    }
    fn parse_type(&mut self, schema: bool) -> Node {
        let mut res = match self.get_token(0).literal.to_uppercase().as_str() {
            "ARRAY" => {
                let mut res = self.construct_node(NodeType::Unknown);
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node(NodeType::Unknown);
                    self.next_token(); // < -> type_expr
                    type_.push_node("type", self.parse_type(schema));
                    self.next_token(); // type_expr -> >
                    type_.push_node("rparen", self.construct_node(NodeType::Unknown));
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "STRUCT" => {
                let mut res = self.construct_node(NodeType::Unknown);
                if self.get_token(1).literal.as_str() == "<" {
                    self.next_token(); // array -> <
                    let mut type_ = self.construct_node(NodeType::Unknown);
                    self.next_token(); // < -> type or ident
                    let mut type_declarations = Vec::new();
                    while !self.cur_token_is(">") {
                        let mut type_declaration;
                        if !self.peek_token_in(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node(NodeType::Unknown);
                            self.next_token(); // ident -> type
                        } else {
                            type_declaration = Node::empty(NodeType::Unknown);
                        }
                        type_declaration.push_node("type", self.parse_type(schema));
                        self.next_token(); // type -> , or next_declaration
                        if self.cur_token_is(",") {
                            type_declaration
                                .push_node("comma", self.construct_node(NodeType::Unknown));
                            self.next_token(); // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node(NodeType::Unknown));
                    type_.push_node_vec("declarations", type_declarations);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "ANY" => {
                let mut res = self.construct_node(NodeType::Unknown);
                self.next_token(); // ANY -> TYPE
                res.push_node("type", self.construct_node(NodeType::Unknown));
                res
            }
            _ => self.construct_node(NodeType::Unknown),
        };
        if self.peek_token_is("not") && schema {
            self.next_token(); // -> not
            let not_ = self.construct_node(NodeType::Unknown);
            self.next_token(); // -> null
            let null = self.construct_node(NodeType::Unknown);
            res.push_node_vec("not_null", vec![not_, null]);
        }
        if self.peek_token_is("options") && schema {
            self.construct_node(NodeType::Unknown); // -> options
            self.next_token(); // options
            let mut options = self.construct_node(NodeType::Unknown);
            self.next_token(); // options -> (
            let mut group = self.construct_node(NodeType::Unknown);
            if !self.peek_token_is(")") {
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
        // 001... date, timestamp, r'', b'' (literal)
        // 101... [], ., ( (call expression. it's not mentioned in documentation)
        // 102... +, - , ~ (unary operator)
        // 103... *, / , ||
        // 104... +, - (binary operator)
        // 105... <<, >>
        // 106... & (bit operator)
        // 107... ^ (bit operator)
        // 108... | (bit operator)
        // 109... =, <, >, like, between, in
        // 110... not
        // 111... and
        // 112... or
        // 200... => (ST_GEOGFROMGEOJSON)
        // 999... LOWEST
        match self.get_token(offset).literal.to_uppercase().as_str() {
            "(" => 101,
            "." => 101,
            "[" => 101,
            "*" => 103,
            "/" => 103,
            "||" => 103,
            "+" => 104,
            "-" => 104,
            "<<" => 105,
            ">>" => 105,
            "&" => 106,
            "^" => 107,
            "|" => 108,
            "=" => 109,
            "<" => 109,
            ">" => 109,
            "<=" => 109,
            ">=" => 109,
            "!=" => 109,
            "<>" => 109,
            "LIKE" => 109,
            "BETWEEN" => 109,
            "IN" => 109,
            "IS" => 109,
            "NOT" => match self.get_token(offset + 1).literal.to_uppercase().as_str() {
                "IN" => 109,
                "LIKE" => 109,
                "BETWEEN" => 109,
                _ => 110,
            },
            "AND" => 111,
            "OR" => 112,
            "=>" => 200,
            _ => 999,
        }
    }
}
