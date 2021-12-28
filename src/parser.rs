#[cfg(test)]
mod tests;

use crate::cst::Node;
use crate::cst::NodeType;
use crate::error::{BQ2CSTError, BQ2CSTResult};
use crate::token::Token;

pub struct Parser {
    position: usize,
    leading_comment_indices: Vec<usize>,
    trailing_comment_indices: Vec<usize>,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        let mut p = Parser {
            position: 0,
            leading_comment_indices: Vec::new(),
            trailing_comment_indices: Vec::new(),
            tokens,
        };
        while p.tokens[p.position].is_comment() {
            p.leading_comment_indices.push(p.position);
            p.position += 1;
        }
        if p.position == p.tokens.len() - 1 {
            return p; // no statement was found
        }
        let mut trailing_comment_idx = p.position + 1;
        while p.tokens[trailing_comment_idx].is_comment()
            && p.tokens[p.position].line == p.tokens[trailing_comment_idx].line
        {
            p.trailing_comment_indices.push(trailing_comment_idx);
            trailing_comment_idx += 1;
        }
        p
    }
    pub fn parse_code(&mut self) -> BQ2CSTResult<Vec<Node>> {
        let mut stmts: Vec<Node> = Vec::new();
        while !self.is_eof(0) {
            let stmt = self.parse_statement(true)?;
            stmts.push(stmt);
            self.next_token()?;
        }
        stmts.push(self.construct_node(NodeType::EOF)?);
        Ok(stmts)
    }
    // ----- core -----
    fn construct_node(&self, node_type: NodeType) -> BQ2CSTResult<Node> {
        // NOTE
        // It is possible to avoid cloning tokens (see #20)
        // but it does not improve execution time.
        let curr_token = self.get_token(0)?;
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
                } else if curr_token.is_parameter() {
                    node.node_type = NodeType::Parameter;
                } else if curr_token.literal.to_uppercase() == "NULL" {
                    node.node_type = NodeType::NullLiteral;
                } else if let "(" | "." = self.get_token(1)?.literal.as_str() {
                    node.node_type = NodeType::Identifier;
                }
                node
            }
            _ => Node::new(self.get_token(0)?.clone(), node_type),
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
        Ok(node)
    }
    fn get_precedence(&self, offset: usize, as_table: bool) -> BQ2CSTResult<usize> {
        // https://cloud.google.com/bigquery/docs/reference/standard-sql/operators
        // 001... - (identifier e.g. region-us)
        // 002... DATE, TIMESTAMP, r'', b'' (literal)
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
        let precedence = match self.get_token(offset)?.literal.to_uppercase().as_str() {
            // return precedence of BINARY operator
            "(" | "." | "[" => 101,
            "*" | "/" | "||" => 103,
            "-" => {
                if as_table {
                    001
                } else {
                    104
                }
            }
            "+" => 104,
            "<<" | ">>" => 105,
            "&" => 106,
            "^" => 107,
            "|" => 108,
            "=" | "<" | ">" | "<=" | ">=" | "!=" | "<>" | "LIKE" | "BETWEEN" | "IN" | "IS" => 109,
            "NOT" => match self.get_token(offset + 1)?.literal.to_uppercase().as_str() {
                "IN" | "LIKE" | "BETWEEN" => 109,
                _ => {
                    return Err(BQ2CSTError::from_token(
                        self.get_token(offset + 1)?,
                        format!(
                            "Expected `IN`, `LIKE` or `BETWEEN` but got: {:?}",
                            self.get_token(offset + 1)?
                        ),
                    ))
                }
            },
            "AND" => 111,
            "OR" => 112,
            "=>" => 200,
            _ => usize::MAX,
        };
        Ok(precedence)
    }
    fn get_offset_index(&self, offset: usize) -> BQ2CSTResult<usize> {
        if offset == 0 {
            return Ok(self.position);
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
                return Err(BQ2CSTError::from_token(
                    &self.tokens[self.tokens.len() - 1],
                    "Followed by unexpected EOF".to_string(),
                ));
            }
        }
        Ok(idx)
    }
    fn get_token(&self, offset: usize) -> BQ2CSTResult<&Token> {
        let idx = self.get_offset_index(offset)?;
        Ok(&self.tokens[idx])
    }
    fn is_eof(&self, offset: usize) -> bool {
        let idx = match self.get_offset_index(offset) {
            Ok(i) => i,
            Err(_) => return true,
        };
        self.tokens.len() - 1 <= idx
    }
    fn next_token(&mut self) -> BQ2CSTResult<()> {
        // leading comments
        self.leading_comment_indices = Vec::new();
        let next_token_idx = self.get_offset_index(1)?;
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
            Ok(i) => i,
            Err(_) => return Ok(()), // already reached EOF
        };
        let mut trailing_comment_idx = self.position + 1;
        while trailing_comment_idx < next_token_idx
            && self.get_token(0)?.line == self.tokens[trailing_comment_idx].line
        {
            self.trailing_comment_indices.push(trailing_comment_idx);
            trailing_comment_idx += 1;
        }
        Ok(())
    }
    fn parse_between_operator(&mut self, left: Node) -> BQ2CSTResult<Node> {
        let precedence = self.get_precedence(0, false)?;
        let mut between = self.construct_node(NodeType::BetweenOperator)?;
        between.push_node("left", left);
        self.next_token()?; // BETWEEN -> expr1

        // NOTE `AND` is not parsed as binary operator because of precedence
        between.push_node("right_min", self.parse_expr(precedence, false, false)?);
        self.next_token()?; // expr1 -> AND
        between.push_node("and", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // AND -> expr2
        between.push_node("right_max", self.parse_expr(precedence, false, false)?);
        Ok(between)
    }
    fn parse_binary_operator(&mut self, left: Node) -> BQ2CSTResult<Node> {
        let precedence = self.get_precedence(0, false)?;
        let mut node = self.construct_node(NodeType::BinaryOperator)?;
        if self.get_token(0)?.is("IS") && self.get_token(1)?.is("NOT") {
            self.next_token()?; // IS -> NOT
            node.push_node("not", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // binary_operator -> expr
        node.push_node("left", left);
        node.push_node("right", self.parse_expr(precedence, false, false)?);
        Ok(node)
    }
    fn parse_expr(&mut self, precedence: usize, alias: bool, as_table: bool) -> BQ2CSTResult<Node> {
        // prefix or literal
        let mut left = self.construct_node(NodeType::Unknown)?;
        match self.get_token(0)?.literal.to_uppercase().as_str() {
            "*" => {
                left.node_type = NodeType::Asterisk;
                match self.get_token(1)?.literal.to_uppercase().as_str() {
                    "REPLACE" => {
                        self.next_token()?; // * -> REPLACE
                        let mut replace = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
                        self.next_token()?; // REPLACE -> (
                        replace.push_node("group", self.parse_grouped_exprs(true)?);
                        left.push_node("replace", replace);
                    }
                    "EXCEPT" => {
                        self.next_token()?; // * -> except
                        let mut except = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
                        self.next_token()?; // except -> (
                        except.push_node("group", self.parse_grouped_exprs(false)?);
                        left.push_node("except", except);
                    }
                    _ => (),
                }
            }
            // STRUCT
            "(" => {
                self.next_token()?; // ( -> expr
                let mut exprs;
                let mut statement_flg = false;
                let mut offset = 0;
                loop {
                    if self.get_token(offset)?.in_(&vec!["WITH", "SELECT"]) {
                        statement_flg = true;
                        break;
                    } else if !self.get_token(offset)?.is("(") {
                        break;
                    }
                    offset += 1;
                }
                if statement_flg {
                    left.node_type = NodeType::GroupedStatement;
                    left.push_node("stmt", self.parse_select_statement(false, true)?);
                } else {
                    exprs = self.parse_exprs(&vec![], true)?; // parse alias in the case of struct
                    if exprs.len() == 1 {
                        left.node_type = NodeType::GroupedExpr;
                        left.push_node("expr", exprs.pop().unwrap());
                    } else {
                        left.node_type = NodeType::StructLiteral;
                        left.push_node_vec("exprs", exprs);
                    }
                }
                self.next_token()?; // expr -> )
                left.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            }
            "STRUCT" => {
                let type_ = self.parse_type(false)?;
                self.next_token()?; // STRUCT -> (, > -> (
                let mut struct_literal = self.construct_node(NodeType::StructLiteral)?;
                let mut exprs = vec![];
                while !self.get_token(1)?.is(")") {
                    self.next_token()?; // -> expr
                    let mut expr = self.parse_expr(usize::MAX, true, false)?;
                    if self.get_token(1)?.is(",") {
                        self.next_token()?; // -> ,
                        expr.push_node("comma", self.construct_node(NodeType::Symbol)?);
                    }
                    exprs.push(expr)
                }
                self.next_token()?; // -> )
                struct_literal.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                struct_literal.push_node_vec("exprs", exprs);
                struct_literal.push_node("type", type_);
                left = struct_literal;
            }
            // ARRAY
            "[" => {
                left.node_type = NodeType::ArrayLiteral;
                self.next_token()?; // [ -> exprs
                left.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
                self.next_token()?; // exprs -> ]
                left.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            }
            "ARRAY" => {
                // when used as literal
                if !self.get_token(1)?.is("(") {
                    let type_ = self.parse_type(false)?;
                    self.next_token()?; // > -> [
                    let mut arr = self.construct_node(NodeType::ArrayLiteral)?;
                    self.next_token()?; // [ -> exprs
                    arr.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
                    self.next_token()?; // exprs -> ]
                    arr.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                    arr.push_node("type", type_);
                    left = arr;
                }
            }
            "-" | "+" | "~" => {
                left.node_type = NodeType::UnaryOperator;
                self.next_token()?; // - -> expr
                let right = self.parse_expr(102, false, false)?;
                left.push_node("right", right);
            }
            "DATE" | "TIME" | "DATETIME" | "TIMESTAMP" | "NUMERIC" | "BIGNUMERIC" | "DECIMAL"
            | "BIGDECIMAL" => {
                if self.get_token(1)?.is_string()
                    || self.get_token(1)?.in_(&vec!["b", "r", "br", "rb"])
                        && self.get_token(2)?.is_string()
                {
                    left.node_type = NodeType::UnaryOperator;
                    self.next_token()?; // -> expr
                    let right = self.parse_expr(002, false, false)?;
                    left.push_node("right", right);
                }
            }
            "INTERVAL" => {
                left.node_type = NodeType::IntervalLiteral;
                self.next_token()?; // INTERVAL -> expr
                let right = self.parse_expr(usize::MAX, false, false)?;
                self.next_token()?; // expr -> HOUR
                left.push_node("date_part", self.construct_node(NodeType::Keyword)?);
                if self.get_token(1)?.is("TO") {
                    self.next_token()?; // -> TO
                    left.push_node("to", self.construct_node(NodeType::Keyword)?);
                    self.next_token()?; // -> date_part
                    left.push_node("to_date_part", self.construct_node(NodeType::Keyword)?);
                }
                left.push_node("expr", right);
            }
            "B" | "R" | "BR" | "RB" => {
                if self.get_token(1)?.is_string() {
                    self.next_token()?; // R -> 'string'
                    let right = self.parse_expr(001, false, false)?;
                    left.push_node("right", right);
                    left.node_type = NodeType::UnaryOperator;
                }
            }
            "WITH" | "SELECT" => {
                // in the case of `ARRAY(SELECT 1)`
                left = self.parse_select_statement(false, true)?;
            }
            "NOT" => {
                self.next_token()?; // NOT -> boolean
                let right = self.parse_expr(110, false, false)?;
                left.push_node("right", right);
                left.node_type = NodeType::UnaryOperator;
            }
            "CASE" => {
                left.node_type = NodeType::CaseExpr;
                self.next_token()?; // CASE -> expr, CASE -> when
                if !self.get_token(0)?.is("WHEN") {
                    left.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                    self.next_token()?; // expr -> WHEN
                }
                let mut arms = Vec::new();
                while !self.get_token(0)?.is("ELSE") {
                    let mut arm = self.construct_node(NodeType::CaseExprArm)?;
                    self.next_token()?; // WHEN -> expr
                    arm.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                    self.next_token()?; // expr ->THEN
                    arm.push_node("then", self.construct_node(NodeType::Keyword)?);
                    self.next_token()?; // THEN -> result_expr
                    arm.push_node("result", self.parse_expr(usize::MAX, false, false)?);
                    self.next_token()?; // result_expr -> ELSE, result_expr -> WHEN
                    arms.push(arm)
                }
                let mut else_ = self.construct_node(NodeType::CaseExprArm)?;
                self.next_token()?; // ELSE -> result_expr
                else_.push_node("result", self.parse_expr(usize::MAX, false, false)?);
                arms.push(else_);
                left.push_node_vec("arms", arms);
                self.next_token()?; // result_expr -> end
                left.push_node("end", self.construct_node(NodeType::Keyword)?);
            }
            _ => (),
        };
        // infix
        while self.get_precedence(1, as_table)? < precedence {
            match self.get_token(1)?.literal.to_uppercase().as_str() {
                "(" => {
                    let func = self.get_token(0)?.literal.to_uppercase();
                    self.next_token()?; // ident -> (
                    let mut node = self.construct_node(NodeType::CallingFunction)?;
                    if self.get_token(1)?.is("distinct") {
                        self.next_token()?; // ( -> DISTINCT
                        node.push_node("distinct", self.construct_node(NodeType::Keyword)?);
                    }
                    self.next_token()?; // ( -> args
                    node.push_node("func", left);
                    if !self.get_token(0)?.is(")") {
                        match func.as_str() {
                            "CAST" | "SAFE_CAST" => {
                                let cast_from = self.parse_expr(usize::MAX, false, false)?;
                                self.next_token()?; // expr -> AS
                                let mut as_ = self.construct_node(NodeType::CastArgument)?;
                                as_.push_node("cast_from", cast_from);
                                self.next_token()?; // -> type
                                as_.push_node("cast_to", self.parse_type(false)?);
                                if self.get_token(1)?.is("FORMAT") {
                                    self.next_token()?; // -> FORMAT
                                    let mut format =
                                        self.construct_node(NodeType::KeywordWithExpr)?;
                                    self.next_token()?; // -> string
                                    format.push_node(
                                        "expr",
                                        self.parse_expr(usize::MAX, false, false)?,
                                    );
                                    as_.push_node("format", format);
                                }
                                node.push_node_vec("args", vec![as_]);
                            }
                            "EXTRACT" => {
                                let datepart = self.parse_expr(usize::MAX, false, false)?;
                                self.next_token()?; // expr -> FROM
                                let mut from = self.construct_node(NodeType::ExtractArgument)?;
                                self.next_token()?; // FROM -> timestamp_expr
                                from.push_node("extract_datepart", datepart);
                                from.push_node(
                                    "extract_from",
                                    self.parse_expr(usize::MAX, false, false)?,
                                );
                                if self.get_token(1)?.is("AT") {
                                    let mut at_time_zone = Vec::new();
                                    self.next_token()?; // timestamp_expr -> AT
                                    at_time_zone.push(self.construct_node(NodeType::Keyword)?);
                                    self.next_token()?; // AT -> TIME
                                    at_time_zone.push(self.construct_node(NodeType::Keyword)?);
                                    self.next_token()?; // TIME -> ZONE
                                    at_time_zone.push(self.construct_node(NodeType::Keyword)?);
                                    from.push_node_vec("at_time_zone", at_time_zone);
                                    self.next_token()?; // ZONE -> 'UTC'
                                    from.push_node(
                                        "time_zone",
                                        self.parse_expr(usize::MAX, false, false)?,
                                    );
                                }
                                node.push_node_vec("args", vec![from]);
                            }
                            _ => {
                                node.push_node_vec("args", self.parse_exprs(&vec![], false)?);
                            }
                        }
                        if self.get_token(1)?.in_(&vec!["respect", "ignore"]) {
                            self.next_token()?; // expr -> RESPECT, IGNORE
                            let ignore_or_respect = self.construct_node(NodeType::Keyword)?;
                            self.next_token()?; // RESPECT, IGNORE -> NULLS
                            node.push_node_vec(
                                "ignore_nulls",
                                vec![ignore_or_respect, self.construct_node(NodeType::Keyword)?],
                            );
                        }
                        if self.get_token(1)?.is("order") {
                            self.next_token()?; // expr -> ORDER
                            let mut orderby = self.construct_node(NodeType::XXXByExprs)?;
                            self.next_token()?; // ORDER -> BY
                            orderby.push_node("by", self.construct_node(NodeType::Keyword)?);
                            self.next_token()?; // BY -> expr
                            orderby.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
                            node.push_node("orderby", orderby);
                        }
                        if self.get_token(1)?.is("LIMIT") {
                            self.next_token()?; // -> LIMIT
                            let mut limit = self.construct_node(NodeType::KeywordWithExpr)?;
                            self.next_token()?;
                            limit.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                            node.push_node("limit", limit);
                        }
                        self.next_token()?; // expr -> )
                    }
                    node.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                    if self.get_token(1)?.is("over") {
                        self.next_token()?; // ) -> OVER
                        let mut over = self.construct_node(NodeType::OverClause)?;
                        self.next_token()?; // OVER -> (, OVER -> named_expr
                        over.push_node("window", self.parse_window_expr()?);
                        node.push_node("over", over);
                    }
                    left = node;
                }
                "[" => {
                    self.next_token()?; // expr -> [
                    let mut node = self.construct_node(NodeType::ArrayAccessing)?;
                    node.push_node("left", left);
                    self.next_token()?; // [ -> index_expr
                    let mut index = self.parse_expr(usize::MAX, false, false)?;
                    index.node_type = NodeType::CallingArrayAccessingFunction;
                    node.push_node("right", index);
                    self.next_token()?; // index_expr -> ]
                    node.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                    left = node;
                }
                "." => {
                    self.next_token()?; // -> .
                    let precedence = self.get_precedence(0, as_table)?;
                    let mut dot = self.construct_node(NodeType::DotOperator)?;
                    self.next_token()?; // -> identifier
                    dot.push_node("left", left);
                    if self.get_token(0)?.literal.as_str() == "*" {
                        dot.push_node("right", self.parse_expr(usize::MAX, false, false)?);
                    } else {
                        dot.push_node("right", self.parse_expr(precedence, false, as_table)?);
                    }
                    left = dot;
                }
                "-" => {
                    if as_table {
                        self.next_token()?; // ident -> -
                        let precedence = self.get_precedence(0, as_table)?;
                        let mut dash = self.construct_node(NodeType::MultiTokenIdentifier)?;
                        self.next_token()?; // - -> ident
                        dash.push_node("left", left);
                        dash.push_node("right", self.parse_expr(precedence, false, as_table)?);
                        left = dash
                    } else {
                        self.next_token()?; // expr -> BETWEEN
                        left = self.parse_between_operator(left)?;
                    }
                }
                "*" | "/" | "||" | "+" | "<<" | ">>" | "&" | "^" | "|" | "=" | "<" | ">" | "<="
                | ">=" | "<>" | "!=" | "LIKE" | "IS" | "AND" | "OR" | "=>" => {
                    self.next_token()?; // expr -> binary_operator
                    left = self.parse_binary_operator(left)?;
                }
                "BETWEEN" => {
                    self.next_token()?; // expr -> BETWEEN
                    left = self.parse_between_operator(left)?;
                }
                "IN" => {
                    self.next_token()?; // expr -> IN
                    left = self.parse_in_operator(left)?;
                }
                "NOT" => {
                    self.next_token()?; // expr -> NOT
                    let not = self.construct_node(NodeType::Keyword)?;
                    self.next_token()?; // NOT -> IN, LIKE, BETWEEN
                    if self.get_token(0)?.is("IN") {
                        left = self.parse_in_operator(left)?;
                        left.push_node("not", not);
                    } else if self.get_token(0)?.is("LIKE") {
                        left = self.parse_binary_operator(left)?;
                        left.push_node("not", not);
                    } else if self.get_token(0)?.is("BETWEEN") {
                        left = self.parse_between_operator(left)?;
                        left.push_node("not", not);
                    } else {
                        return Err(BQ2CSTError::from_token(
                            self.get_token(1)?,
                            format!(
                                "Expected `LIKE`, `BETWEEN` or `IN` but got: {:?}",
                                self.get_token(1)?
                            ),
                        ));
                    }
                }
                _ => {
                    return Err(BQ2CSTError::from_token(
                        self.get_token(0)?,
                        "Something went wrong.".to_string(),
                    ))
                }
            }
        }
        // alias
        if alias {
            if self.get_token(1)?.is("AS") {
                self.next_token()?; // expr -> AS
                left.push_node("as", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // AS -> alias
                left.push_node("alias", self.construct_node(NodeType::Identifier)?);
            } else if self.get_token(1)?.is_identifier() {
                self.next_token()?; // expr -> alias
                left.push_node("alias", self.construct_node(NodeType::Identifier)?);
            }
        }
        if self.get_token(1)?.in_(&vec!["ASC", "DESC"]) {
            self.next_token()?; // expr -> ASC, DESC
            let order = self.construct_node(NodeType::Keyword)?;
            left.push_node("order", order);
        }
        if self.get_token(1)?.in_(&vec!["NULLS"]) && self.get_token(2)?.in_(&vec!["FIRST", "LAST"])
        {
            let mut nulls_first = Vec::new();
            self.next_token()?; // ASC -> NULLS
            nulls_first.push(self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // NULLS -> FIRST, LAST
            nulls_first.push(self.construct_node(NodeType::Keyword)?);
            left.push_node_vec("null_order", nulls_first);
        }
        Ok(left)
    }
    fn parse_exprs(&mut self, until: &Vec<&str>, alias: bool) -> BQ2CSTResult<Vec<Node>> {
        let mut exprs: Vec<Node> = Vec::new();
        // first expr
        let mut expr = self.parse_expr(usize::MAX, alias, false)?;
        if self.get_token(1)?.is(",") {
            self.next_token()?; // expr -> ,
            expr.push_node("comma", self.construct_node(NodeType::Symbol)?);
        } else {
            return Ok(vec![expr]);
        }
        exprs.push(expr);
        // second expr and later
        while !self.get_token(1)?.in_(until) && !self.is_eof(1) {
            self.next_token()?;
            let mut expr = self.parse_expr(usize::MAX, alias, false)?;
            if self.get_token(1)?.is(",") {
                self.next_token()?; // expr -> ,
                expr.push_node("comma", self.construct_node(NodeType::Symbol)?);
                exprs.push(expr);
            } else {
                exprs.push(expr);
                break;
            }
        }
        Ok(exprs)
    }
    fn parse_grouped_exprs(&mut self, alias: bool) -> BQ2CSTResult<Node> {
        let mut group = self.construct_node(NodeType::GroupedExprs)?;
        if !self.get_token(1)?.is(")") {
            self.next_token()?; // ( -> exprs
            group.push_node_vec("exprs", self.parse_exprs(&vec![], alias)?);
        }
        self.next_token()?; // exprs -> )
        group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
        Ok(group)
    }
    fn parse_grouped_type_declarations(&mut self, schema: bool) -> BQ2CSTResult<Node> {
        let mut group = self.construct_node(NodeType::GroupedTypeDeclarations)?;
        self.next_token()?; // ( -> INOUT | ident | type
        let mut type_declarations = Vec::new();
        while !self.get_token(0)?.in_(&vec![">", ")"]) {
            let mut type_declaration;
            if self.get_token(0)?.in_(&vec!["IN", "OUT", "INOUT"])
                && !self.get_token(2)?.in_(&vec![",", ">", ")", "TYPE", "<"])
            {
                // `self.get_token(1).is_identifier()` does not work here
                // because `INT64` is also valid identifier
                // , ... INT64,
                // > ... INT64>
                // > ... INT64)
                // <... STRUCT<> | ARRAY<>
                // TYPE... ANY TYPE
                let in_out = self.construct_node(NodeType::Keyword)?;
                self.next_token()?; // -> ident
                type_declaration = self.construct_node(NodeType::TypeDeclaration)?;
                type_declaration.push_node("in_out", in_out);
                self.next_token()?; // -> type
            } else if !self.get_token(1)?.in_(&vec![",", ">", ")", "TYPE", "<"]) {
                type_declaration = self.construct_node(NodeType::TypeDeclaration)?;
                self.next_token()?; // -> type
            } else {
                type_declaration = Node::empty(NodeType::TypeDeclaration);
            }
            type_declaration.push_node("type", self.parse_type(schema)?);
            self.next_token()?; //  -> , | > | )
            if self.get_token(0)?.is(",") {
                type_declaration.push_node("comma", self.construct_node(NodeType::Symbol)?);
                self.next_token()?; // , -> type
            }
            type_declarations.push(type_declaration);
        }
        if 0 < type_declarations.len() {
            group.push_node_vec("declarations", type_declarations);
        }
        group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
        Ok(group)
    }
    fn parse_identifier(&mut self) -> BQ2CSTResult<Node> {
        // NOTE
        // This method is used to parse only identifier.
        // If you want to parse table function, you have to use parse_expr().
        fn parse_single_or_multi_token_identifier(parser: &mut Parser) -> BQ2CSTResult<Node> {
            let mut left = parser.construct_node(NodeType::Identifier)?;
            while parser.get_token(1)?.is("-") {
                parser.next_token()?; // ident -> -
                let mut dash = parser.construct_node(NodeType::MultiTokenIdentifier)?;
                dash.push_node("left", left);
                parser.next_token()?; // - -> ient
                dash.push_node("right", parser.construct_node(NodeType::Identifier)?);
                left = dash;
            }
            Ok(left)
        }

        let mut left = parse_single_or_multi_token_identifier(self)?;
        while self.get_token(1)?.is(".") {
            self.next_token()?; // ident -> .
            let mut operator = self.construct_node(NodeType::DotOperator)?;
            operator.push_node("left", left);
            self.next_token()?; // . -> ident
            operator.push_node("right", parse_single_or_multi_token_identifier(self)?);
            left = operator;
        }
        Ok(left)
    }
    fn parse_in_operator(&mut self, left: Node) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::InOperator)?;
        node.push_node("left", left);
        if self.get_token(1)?.is("UNNEST") {
            self.next_token()?; // IN -> UNNEST
            let mut unnest = self.parse_expr(usize::MAX, false, false)?;
            unnest.node_type = NodeType::CallingUnnest;
            node.push_node("right", unnest);
        } else {
            self.next_token()?; // IN -> (
            if self.get_token(1)?.in_(&vec!["SELECT", "WITH"]) {
                let mut lparen = self.construct_node(NodeType::GroupedStatement)?;
                self.next_token()?; // -> SELECT | WITH
                lparen.push_node("stmt", self.parse_select_statement(false, true)?);
                self.next_token()?; // -> )
                lparen.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                node.push_node("right", lparen);
            } else {
                node.push_node("right", self.parse_grouped_exprs(false)?);
            }
        }
        Ok(node)
    }
    fn parse_keyword_with_grouped_exprs(&mut self, alias: bool) -> BQ2CSTResult<Node> {
        let mut keyword = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
        self.next_token()?; // keyword -> (
        keyword.push_node("group", self.parse_grouped_exprs(alias)?);
        Ok(keyword)
    }
    fn parse_keyword_with_statements(&mut self, until: &Vec<&str>) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::KeywordWithStatements)?;
        let mut stmts = Vec::new();
        while !self.get_token(1)?.in_(until) {
            self.next_token()?; // -> stmt
            stmts.push(self.parse_statement(true)?);
        }
        node.push_node_vec("stmts", stmts);
        Ok(node)
    }
    fn parse_n_keywords(&mut self, n: usize) -> BQ2CSTResult<Vec<Node>> {
        let mut nodes = Vec::new();
        nodes.push(self.construct_node(NodeType::Keyword)?);
        for _ in 1..n {
            self.next_token()?;
            nodes.push(self.construct_node(NodeType::Keyword)?);
        }
        Ok(nodes)
    }
    fn parse_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let node = match self.get_token(0)?.literal.to_uppercase().as_str() {
            // SELECT
            "WITH" | "SELECT" | "(" => self.parse_select_statement(semicolon, true)?,
            // DML
            "INSERT" => self.parse_insert_statement(semicolon)?,
            "DELETE" => self.parse_delete_statement(semicolon)?,
            "TRUNCATE" => self.parse_truncate_statement(semicolon)?,
            "UPDATE" => self.parse_update_statement(semicolon)?,
            "MERGE" => self.parse_merge_statement(semicolon)?,
            // DDL
            "CREATE" => {
                let mut offset = 1;
                loop {
                    match self.get_token(offset)?.literal.to_uppercase().as_str() {
                        "SCHEMA" => return self.parse_create_schema_statement(semicolon),
                        "TABLE" => {
                            if self.get_token(offset + 1)?.literal.to_uppercase().as_str()
                                == "FUNCTION"
                            {
                                return self.parse_create_function_statement(semicolon);
                            } else {
                                return self.parse_create_table_statement(semicolon);
                            }
                        }
                        "VIEW" => return self.parse_create_view_statement(semicolon),
                        "FUNCTION" => return self.parse_create_function_statement(semicolon),
                        "PROCEDURE" => return self.parse_create_procedure_statement(semicolon),
                        "CAPACITY" | "RESERVATION" | "ASSIGNMENT" => {
                            return self.parse_create_reservation_statement(semicolon)
                        }
                        _ => {
                            offset += 1;
                            if 5 < offset {
                                break;
                            }
                        }
                    }
                }
                return Err(BQ2CSTError::from_token(
                    self.get_token(0)?,
                    format!("Expected `SCHEMA`, `TABLE`, `VIEW`, `FUNCTION`, `PROCEDURE`, 'CAPACITY', 'RESERVATION' or 'ASSIGNMENT' but not found around here: {:?}", self.get_token(0)?)
                ));
            }
            "ALTER" => {
                let mut offset = 1;
                loop {
                    match self.get_token(offset)?.literal.to_uppercase().as_str() {
                        "SCHEMA" => return self.parse_alter_schema_statement(semicolon),
                        "TABLE" => return self.parse_alter_table_statement(semicolon),
                        "COLUMN" => return self.parse_alter_column_statement(semicolon),
                        "VIEW" => return self.parse_alter_view_statement(semicolon),
                        _ => {
                            offset += 1;
                            if 5 < offset {
                                break;
                            }
                        }
                    }
                }
                return Err(BQ2CSTError::from_token(
                    self.get_token(0)?,
                    format!(
                        "Expected `SCHEMA`, `TABLE` or `VIEW` but not found around here: {:?}",
                        self.get_token(0)?
                    ),
                ));
            }
            "DROP" => self.parse_drop_statement(semicolon)?,
            // DCL
            "GRANT" => self.parse_grant_statement(semicolon)?,
            "REVOKE" => self.parse_revoke_statement(semicolon)?,
            // script
            "DECLARE" => self.parse_declare_statement(semicolon)?,
            "SET" => self.parse_set_statement(semicolon)?,
            "EXECUTE" => self.parse_execute_statement(semicolon)?,
            "IF" => self.parse_if_statement(semicolon)?,
            "BEGIN" => {
                if self.get_token(1)?.in_(&vec!["TRANSACTION", ";"]) || self.is_eof(1) {
                    return Ok(self.parse_transaction_statement(semicolon)?);
                }
                self.parse_begin_statement(semicolon)?
            }
            "CASE" => self.parse_case_statement(semicolon)?,
            "LOOP" => self.parse_loop_statement(semicolon)?,
            "REPEAT" => self.parse_repeat_statement(semicolon)?,
            "WHILE" => self.parse_while_statement(semicolon)?,
            "BREAK" | "LEAVE" | "CONTINUE" | "ITERATE" => {
                self.parse_break_continue_statement(semicolon)?
            }
            "FOR" => self.parse_for_statement(semicolon)?,
            "COMMIT" | "ROLLBACK" => self.parse_transaction_statement(semicolon)?,
            "RAISE" => self.parse_raise_statement(semicolon)?,
            "RETURN" => self.parse_single_token_statement(semicolon)?,
            "CALL" => self.parse_call_statement(semicolon)?,
            // DEBUG
            "ASSERT" => self.parse_assert_satement(semicolon)?,
            // other
            "EXPORT" => self.parse_export_statement(semicolon)?,
            _ => self.parse_labeled_statement(semicolon)?,
        };
        Ok(node)
    }
    fn parse_table(&mut self, root: bool) -> BQ2CSTResult<Node> {
        let mut left: Node;
        match self.get_token(0)?.literal.to_uppercase().as_str() {
            "(" => {
                let mut group;
                let mut statement_flg = false;
                let mut offset = 0;
                loop {
                    offset += 1;
                    if self.get_token(offset)?.in_(&vec!["WITH", "SELECT"]) {
                        statement_flg = true;
                        break;
                    } else if !self.get_token(offset)?.is("(") {
                        break;
                    }
                }
                if statement_flg {
                    group = self.parse_select_statement(false, true)?;
                } else {
                    group = self.construct_node(NodeType::GroupedExpr)?;
                    self.next_token()?; // ( -> expr
                    group.push_node("expr", self.parse_table(true)?);
                    self.next_token()?; // table -> )
                    group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                }
                left = group;
            }
            "UNNEST" => {
                left = self.parse_expr(usize::MAX, false, false)?;
                left.node_type = NodeType::CallingUnnest;
            }
            _ => {
                left = self.parse_expr(usize::MAX, false, true)?;
            }
        }
        if left.node_type == NodeType::CallingFunction {
            left.node_type = NodeType::CallingTableFunction; // EXTERNAL_QUERY() is included
        }
        // alias
        // NOTE PIVOT and UNPIVOT are not reserved keywords
        if !(self.get_token(1)?.in_(&vec!["PIVOT", "UNPIVOT"])
            && self.get_token(2)?.in_(&vec!["(", "INCLUDE", "EXCLUDE"]))
        {
            left = self.push_trailing_alias(left)?;
        }
        // FOR SYSTEM_TIME AS OF
        if self.get_token(1)?.literal.to_uppercase() == "FOR" {
            self.next_token()?; // TABLE -> FOR
            let mut for_ = self.construct_node(NodeType::ForSystemTimeAsOfClause)?;
            self.next_token()?; // FOR -> SYSTEM_TIME
            let mut system_time_as_of = Vec::new();
            system_time_as_of.push(self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // SYSTEM_TIME -> AS
            system_time_as_of.push(self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // AS -> OF
            system_time_as_of.push(self.construct_node(NodeType::Keyword)?);
            for_.push_node_vec("system_time_as_of", system_time_as_of);
            self.next_token()?; // OF -> timestamp
            for_.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            left.push_node("for_system_time_as_of", for_);
        }
        // WITH, OFFSET
        if self.get_token(1)?.literal.to_uppercase() == "WITH" {
            self.next_token()?; // UNNEST() -> WITH
            let with = self.construct_node(NodeType::Keyword)?;
            self.next_token()?; // WITH -> OFFSET
            let offset = self.construct_node(NodeType::Keyword)?;
            if self.get_token(1)?.is("AS") {
                self.next_token()?; // OFFSET -> AS
                left.push_node("offset_as", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // AS -> alias
                left.push_node("offset_alias", self.construct_node(NodeType::Identifier)?);
            } else if self.get_token(1)?.is_identifier() {
                self.next_token()?; // expr -> alias
                left.push_node("offset_alias", self.construct_node(NodeType::Identifier)?);
            }
            left.push_node_vec("with_offset", vec![with, offset]);
        }
        // PIVOT, UNPIVOT
        if self.get_token(1)?.is("PIVOT") {
            self.next_token()?; // -> PIVOT
            let mut pivot = self.construct_node(NodeType::PivotOperator)?;
            self.next_token()?; // -> (
            let mut config = self.construct_node(NodeType::PivotConfig)?;
            self.next_token()?; // -> expr
            config.push_node_vec("exprs", self.parse_exprs(&vec![], true)?);
            self.next_token()?; // -> FOR
            let mut for_ = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> expr
            for_.push_node("expr", self.construct_node(NodeType::Identifier)?);
            config.push_node("for", for_);
            self.next_token()?; // -> IN
            config.push_node("in", self.parse_keyword_with_grouped_exprs(true)?);
            self.next_token()?; // -> )
            config.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            pivot.push_node("config", config);
            pivot = self.push_trailing_alias(pivot)?;
            left.push_node("pivot", pivot);
        } else if self.get_token(1)?.is("UNPIVOT") {
            self.next_token()?; // -> UNPIVOT
            let mut unpivot = self.construct_node(NodeType::UnpivotOperator)?;
            if self.get_token(1)?.in_(&vec!["INCLUDE", "EXCLUDE"]) {
                self.next_token()?; // -> INCLUDE | EXCLUDE
                unpivot.push_node_vec("include_or_exclude_nulls", self.parse_n_keywords(2)?);
            }
            self.next_token()?; // -> (
            let mut config = self.construct_node(NodeType::UnpivotConfig)?;
            self.next_token()?; // -> expr
            if self.get_token(0)?.is("(") {
                // in the case of multi column unpivot
                config.push_node("expr", self.parse_grouped_exprs(false)?);
            } else {
                config.push_node("expr", self.parse_expr(usize::MAX, true, false)?);
            }
            self.next_token()?; // -> FOR
            let mut for_ = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> expr
            for_.push_node("expr", self.construct_node(NodeType::Identifier)?);
            config.push_node("for", for_);
            self.next_token()?; // -> IN
            let mut in_ = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
            self.next_token()?; // -> (
            let mut group = self.construct_node(NodeType::GroupedExprs)?;
            let mut exprs = Vec::new();
            while !self.get_token(1)?.is(")") {
                self.next_token()?; // -> expr
                let mut expr;
                if self.get_token(0)?.is("(") {
                    // in the case of multi column unpivot
                    expr = self.parse_grouped_exprs(false)?;
                } else {
                    expr = self.parse_expr(usize::MAX, false, false)?;
                }
                if self.get_token(1)?.is("AS") {
                    self.next_token()?; // -> AS
                    expr.push_node("as", self.construct_node(NodeType::Keyword)?);
                }
                if self.get_token(1)?.is_string() || self.get_token(1)?.is_numeric() {
                    self.next_token()?; // -> row_value_alias
                    expr.push_node(
                        "row_value_alias",
                        self.parse_expr(usize::MAX, false, false)?,
                    );
                }
                if self.get_token(1)?.is(",") {
                    self.next_token()?; // -> ,
                    expr.push_node("comma", self.construct_node(NodeType::Symbol)?);
                } else {
                    exprs.push(expr);
                    break;
                }
                exprs.push(expr);
            }
            self.next_token()?; // -> )
            group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            group.push_node_vec("exprs", exprs);
            in_.push_node("group", group);
            config.push_node("in", in_);
            self.next_token()?; // -> )
            config.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            unpivot.push_node("config", config);
            unpivot = self.push_trailing_alias(unpivot)?;
            left.push_node("unpivot", unpivot);
        }
        // TABLESAMPLE
        if self.get_token(1)?.is("tablesample") {
            // TODO check when it becomes GA
            self.next_token()?; // -> TABLESAMPLE
            let mut tablesample = self.construct_node(NodeType::TableSampleClause)?;
            self.next_token()?; // -> SYSTEM
            tablesample.push_node("system", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // -> (
            let mut group = self.construct_node(NodeType::TableSampleRatio)?;
            self.next_token()?; // -> expr
            group.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            self.next_token()?; // -> PERCENT
            group.push_node("percent", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // -> )
            group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            tablesample.push_node("group", group);
            left.push_node("tablesample", tablesample);
        }
        // JOIN
        while self.get_token(1)?.in_(&vec![
            "left", "right", "cross", "inner", "full", "join", ",",
        ]) && root
        {
            self.next_token()?; // table -> LEFT, RIGHT, INNER, CROSS, FULL, JOIN, ","
            let mut join = if self.get_token(0)?.in_(&vec!["join", ","]) {
                let join = self.construct_node(NodeType::JoinOperator)?;
                join
            } else {
                let type_ = self.construct_node(NodeType::Keyword)?;
                self.next_token()?; // join_type -> OUTER, JOIN
                if self.get_token(0)?.is("OUTER") {
                    let outer = self.construct_node(NodeType::Keyword)?;
                    self.next_token()?; // OUTER -> JOIN
                    let mut join = self.construct_node(NodeType::JoinOperator)?;
                    join.push_node("join_type", type_);
                    join.push_node("outer", outer);
                    join
                } else {
                    let mut join = self.construct_node(NodeType::JoinOperator)?;
                    join.push_node("join_type", type_);
                    join
                }
            };
            self.next_token()?; // -> table
            let right = self.parse_table(false)?;
            if self.get_token(1)?.is("on") {
                self.next_token()?; // `table` -> ON
                let mut on = self.construct_node(NodeType::KeywordWithExpr)?;
                self.next_token()?; // ON -> expr
                on.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                join.push_node("on", on);
            } else if self.get_token(1)?.is("using") {
                self.next_token()?; // -> USING
                join.push_node("using", self.parse_expr(usize::MAX, false, false)?)
            }
            join.push_node("left", left);
            join.push_node("right", right);
            left = join;
        }
        Ok(left)
    }
    fn parse_type(&mut self, schema: bool) -> BQ2CSTResult<Node> {
        let mut res = match self.get_token(0)?.literal.to_uppercase().as_str() {
            "ARRAY" => {
                let mut res = self.construct_node(NodeType::Type)?;
                if self.get_token(1)?.literal.as_str() == "<" {
                    self.next_token()?; // ARRAY -> <
                    let mut type_ = self.construct_node(NodeType::GroupedType)?;
                    self.next_token()?; // < -> type
                    type_.push_node("type", self.parse_type(schema)?);
                    self.next_token()?; // type -> >
                    type_.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "STRUCT" | "TABLE" => {
                let mut res = self.construct_node(NodeType::Type)?;
                if self.get_token(1)?.literal.as_str() == "<" {
                    self.next_token()?; // STRUCT -> <
                    let mut type_ = self.construct_node(NodeType::GroupedTypeDeclarations)?;
                    self.next_token()?; // < -> type or ident
                    let mut type_declarations = Vec::new();
                    while !self.get_token(0)?.is(">") {
                        let mut type_declaration;
                        if !self.get_token(1)?.in_(&vec![",", ">", "TYPE", "<"]) {
                            // `is_identifier` is not availabe here,
                            // because `int64` is valid identifier
                            type_declaration = self.construct_node(NodeType::TypeDeclaration)?;
                            self.next_token()?; // ident -> type
                        } else {
                            type_declaration = Node::empty(NodeType::TypeDeclaration);
                        }
                        type_declaration.push_node("type", self.parse_type(schema)?);
                        self.next_token()?; // type -> , or next_declaration
                        if self.get_token(0)?.is(",") {
                            type_declaration
                                .push_node("comma", self.construct_node(NodeType::Symbol)?);
                            self.next_token()?; // , -> next_declaration
                        }
                        type_declarations.push(type_declaration);
                    }
                    type_.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                    type_.push_node_vec("declarations", type_declarations);
                    res.push_node("type_declaration", type_);
                }
                res
            }
            "ANY" => {
                let mut res = self.construct_node(NodeType::Type)?;
                self.next_token()?; // ANY -> TYPE
                res.push_node("type", self.construct_node(NodeType::Keyword)?);
                res
            }
            _ => {
                let mut res = self.construct_node(NodeType::Type)?;
                if self.get_token(1)?.is("(") {
                    self.next_token()?; // -> (
                    res.push_node("parameter", self.parse_grouped_exprs(false)?);
                }
                res
            }
        };
        if self.get_token(1)?.is("NOT") && schema {
            self.next_token()?; // -> NOT
            let not_ = self.construct_node(NodeType::Keyword)?;
            self.next_token()?; // -> null
            let null = self.construct_node(NodeType::Keyword)?;
            res.push_node_vec("not_null", vec![not_, null]);
        }
        if self.get_token(1)?.is("OPTIONS") && schema {
            self.next_token()?; // -> OPTIONS
            let options = self.parse_keyword_with_grouped_exprs(false)?;
            res.push_node("options", options);
        }
        Ok(res)
    }
    fn parse_window_expr(&mut self) -> BQ2CSTResult<Node> {
        if self.get_token(0)?.is("(") {
            let mut window = self.construct_node(NodeType::WindowSpecification)?;
            if self.get_token(1)?.is_identifier() {
                self.next_token()?; // ( -> identifier
                window.push_node("name", self.construct_node(NodeType::Identifier)?);
            }
            if self.get_token(1)?.is("PARTITION") {
                self.next_token()?; // ( -> PARTITION
                let mut partition = self.construct_node(NodeType::XXXByExprs)?;
                self.next_token()?; // PARTITION -> BY
                partition.push_node("by", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // BY -> exprs
                partition.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
                window.push_node("partitionby", partition);
            }
            if self.get_token(1)?.is("ORDER") {
                self.next_token()?; // ( -> ORDER
                let mut order = self.construct_node(NodeType::XXXByExprs)?;
                self.next_token()?; // ORDER -> BY
                order.push_node("by", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // BY -> exprs
                order.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
                window.push_node("orderby", order);
            }
            if self.get_token(1)?.in_(&vec!["RANGE", "ROWS"]) {
                self.next_token()?; // ( -> ROWS, expr -> ROWS
                let mut frame = self.construct_node(NodeType::WindowFrameClause)?;
                if self.get_token(1)?.is("BETWEEN") {
                    // frame_between
                    self.next_token()?; // ROWS -> BETWEEN
                    frame.push_node("between", self.construct_node(NodeType::Keyword)?);
                    // start
                    self.next_token()?; // BETWEEN -> UNBOUNDED, CURRENT
                    let mut frame_start = Vec::new();
                    if self.get_token(0)?.in_(&vec!["UNBOUNDED", "CURRENT"]) {
                        frame_start.push(self.construct_node(NodeType::Keyword)?);
                    } else {
                        frame_start.push(self.parse_expr(usize::MAX, false, false)?);
                    }
                    self.next_token()?; // -> PRECEDING, ROW
                    frame_start.push(self.construct_node(NodeType::Keyword)?);
                    frame.push_node_vec("start", frame_start);
                    self.next_token()?; // -> AND
                    frame.push_node("and", self.construct_node(NodeType::Keyword)?);
                    // end
                    self.next_token()?; // AND -> UNBOUNDED, CURRENT
                    let mut frame_end = Vec::new();
                    if self.get_token(0)?.in_(&vec!["UNBOUNDED", "CURRENT"]) {
                        frame_end.push(self.construct_node(NodeType::Keyword)?);
                    } else {
                        frame_end.push(self.parse_expr(usize::MAX, false, false)?);
                    }
                    self.next_token()?; // -> FOLLOWING, ROW
                    frame_end.push(self.construct_node(NodeType::Keyword)?);
                    frame.push_node_vec("end", frame_end);
                } else {
                    // frame_start
                    if !self.get_token(1)?.is(")") {
                        self.next_token()?; // ROWS -> UNBOUNDED, CURRENT
                        let mut frame_start = Vec::new();
                        if self.get_token(0)?.in_(&vec!["UNBOUNDED", "CURRENT"]) {
                            frame_start.push(self.construct_node(NodeType::Keyword)?);
                        } else {
                            frame_start.push(self.parse_expr(usize::MAX, false, false)?);
                        }
                        self.next_token()?; // -> PRECEDING, ROW
                        frame_start.push(self.construct_node(NodeType::Keyword)?);
                        frame.push_node_vec("start", frame_start);
                    }
                }
                window.push_node("frame", frame)
            }
            self.next_token()?; // -> )
            window.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            Ok(window)
        } else {
            Ok(self.construct_node(NodeType::Identifier)?)
        }
    }
    fn parse_xxxby_exprs(&mut self) -> BQ2CSTResult<Node> {
        let mut xxxby = self.construct_node(NodeType::XXXByExprs)?;
        self.next_token()?; // xxx -> BY
        xxxby.push_node("by", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // BY -> expr
        xxxby.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
        Ok(xxxby)
    }
    fn push_trailing_alias(&mut self, mut node: Node) -> BQ2CSTResult<Node> {
        if self.get_token(1)?.is("AS") {
            self.next_token()?; // -> AS
            node.push_node("as", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // AS -> ident
            node.push_node("alias", self.construct_node(NodeType::Identifier)?);
        } else if self.get_token(1)?.is_identifier() {
            self.next_token()?; // -> ident
            node.push_node("alias", self.construct_node(NodeType::Identifier)?);
        }
        Ok(node)
    }
    // ----- SELECT statement -----
    fn parse_select_statement(&mut self, semicolon: bool, root: bool) -> BQ2CSTResult<Node> {
        if self.get_token(0)?.literal.to_uppercase() == "(" {
            let mut node = self.construct_node(NodeType::GroupedStatement)?;
            self.next_token()?; // ( -> SELECT
            node.push_node("stmt", self.parse_select_statement(false, true)?);
            self.next_token()?; // stmt -> )
            node.push_node("rparen", self.construct_node(NodeType::Symbol)?);
            while self
                .get_token(1)?
                .in_(&vec!["UNION", "INTERSECT", "EXCEPT"])
                && root
            {
                self.next_token()?; // stmt -> UNION
                let mut operator = self.construct_node(NodeType::SetOperator)?;
                self.next_token()?; // UNION -> DISTINCT
                operator.push_node("distinct_or_all", self.construct_node(NodeType::Keyword)?);
                operator.push_node("left", node);
                self.next_token()?; // DISTINCT -> stmt
                operator.push_node("right", self.parse_select_statement(false, false)?);
                node = operator;
            }
            if self.get_token(1)?.is(";") && semicolon && root {
                self.next_token()?; // expr -> ;
                node.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
            }
            return Ok(node);
        }
        if self.get_token(0)?.literal.to_uppercase() == "WITH" {
            let mut with = self.construct_node(NodeType::WithClause)?;
            let mut queries = Vec::new();
            while self.get_token(1)?.literal.to_uppercase() != "SELECT"
                && self.get_token(1)?.literal != "("
            {
                self.next_token()?; // WITH -> ident, ) -> ident
                let mut query = self.construct_node(NodeType::WithQuery)?;
                self.next_token()?; // ident -> AS
                query.push_node("as", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // AS -> (
                query.push_node("stmt", self.parse_select_statement(false, true)?);
                if self.get_token(1)?.literal.as_str() == "," {
                    self.next_token()?; // ) -> ,
                    query.push_node("comma", self.construct_node(NodeType::Symbol)?);
                }
                queries.push(query);
            }
            with.push_node_vec("queries", queries);
            self.next_token()?; // -> SELECT | '('
            let mut node = self.parse_select_statement(semicolon, true)?;
            node.push_node("with", with);
            return Ok(node);
        }
        // SELECT
        let mut node = self.construct_node(NodeType::SelectStatement)?;

        // AS STRUCT, VALUE
        if self.get_token(1)?.literal.to_uppercase() == "AS" {
            self.next_token()?; // SELECT -> AS
            let as_ = self.construct_node(NodeType::Keyword)?;
            self.next_token()?; // AS -> STRUCT, VALUE
            node.push_node_vec(
                "as_struct_or_value",
                vec![as_, self.construct_node(NodeType::Keyword)?],
            );
        }

        // DISTINCT
        if self.get_token(1)?.in_(&vec!["ALL", "DISTINCT"]) {
            self.next_token()?; // select -> all, distinct
            node.push_node("distinct_or_all", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> expr

        // exprs
        node.push_node_vec(
            "exprs",
            self.parse_exprs(
                &vec![
                    "FROM",
                    "WHERE",
                    "GROUP",
                    "HAVING",
                    "QUALIFY",
                    "WINDOW",
                    "ORDER",
                    "LIMIT",
                    "UNION",
                    "INTERSECT",
                    "EXCEPT",
                    ";",
                    ")",
                ],
                true,
            )?,
        );
        // FROM
        if self.get_token(1)?.is("FROM") {
            self.next_token()?; // expr -> FROM
            let mut from = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // FROM -> table
            from.push_node("expr", self.parse_table(true)?);
            node.push_node("from", from);
        }
        // WHERE
        if self.get_token(1)?.is("WHERE") {
            self.next_token()?; // expr -> WHERE
            let mut where_ = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // WHERE -> expr
            where_.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            node.push_node("where", where_);
        }
        // GROUP BY
        if self.get_token(1)?.is("GROUP") {
            self.next_token()?; // expr -> GROUP
            let mut groupby = self.construct_node(NodeType::XXXByExprs)?;
            self.next_token()?; // GROUP -> BY
            groupby.push_node("by", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // BY -> expr
            groupby.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
            node.push_node("groupby", groupby);
        }
        // HAVING
        if self.get_token(1)?.is("HAVING") {
            self.next_token()?; // expr -> HAVING
            let mut having = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // HAVING -> expr
            having.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            node.push_node("having", having);
        }
        // QUALIFY
        // TODO check when it becomes GA
        if self.get_token(1)?.is("QUALIFY") {
            self.next_token()?; // -> QUALIFY
            let mut qualify = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> expr
            qualify.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            node.push_node("qualify", qualify);
        }
        // WINDOW
        if self.get_token(1)?.is("WINDOW") {
            self.next_token()?; // -> WINDOW
            let mut window = self.construct_node(NodeType::WindowClause)?;
            let mut window_exprs = Vec::new();
            while self.get_token(1)?.is_identifier() {
                self.next_token()?; // -> ident
                let mut window_expr = self.construct_node(NodeType::WindowExpr)?;
                self.next_token()?; // ident -> AS
                window_expr.push_node("as", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // AS -> (, AS -> named_window
                window_expr.push_node("window", self.parse_window_expr()?);
                if self.get_token(1)?.is(",") {
                    self.next_token()?; // -> ,
                    window_expr.push_node("comma", self.construct_node(NodeType::Symbol)?);
                }
                window_exprs.push(window_expr);
            }
            window.push_node_vec("window_exprs", window_exprs);
            node.push_node("window", window);
        }
        // ORDER BY
        if self.get_token(1)?.is("ORDER") {
            self.next_token()?; // expr -> ORDER
            let mut order = self.construct_node(NodeType::XXXByExprs)?;
            self.next_token()?; // ORDER -> BY
            order.push_node("by", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // BY -> expr
            order.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
            node.push_node("orderby", order);
        }
        // LIMIT
        if self.get_token(1)?.is("LIMIT") {
            self.next_token()?; // expr -> LIMIT
            let mut limit = self.construct_node(NodeType::LimitClause)?;
            self.next_token()?; // LIMIT -> expr
            limit.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            if self.get_token(1)?.literal.to_uppercase() == "OFFSET" {
                self.next_token()?; // expr -> OFFSET
                let mut offset = self.construct_node(NodeType::KeywordWithExpr)?;
                self.next_token()?; // OFFSET -> expr
                offset.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                limit.push_node("offset", offset);
            }
            node.push_node("limit", limit);
        }
        // UNION
        while self
            .get_token(1)?
            .in_(&vec!["UNION", "INTERSECT", "EXCEPT"])
            && root
        {
            self.next_token()?; // stmt -> UNION
            let mut operator = self.construct_node(NodeType::SetOperator)?;
            self.next_token()?; // UNION -> DISTINCT
            operator.push_node("distinct_or_all", self.construct_node(NodeType::Keyword)?);
            operator.push_node("left", node);
            self.next_token()?; // DISTINCT -> stmt
            operator.push_node("right", self.parse_select_statement(false, false)?);
            node = operator;
        }
        // ;
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // expr -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(node)
    }
    // ----- DML -----
    fn parse_insert_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut insert = self.construct_node(NodeType::InsertStatement)?;
        if self.get_token(1)?.is("INTO") {
            self.next_token()?; // INSERT -> INTO
            insert.push_node("into", self.construct_node(NodeType::Keyword)?);
        }
        if !self.get_token(1)?.in_(&vec!["(", "VALUES", "ROW"]) {
            // identifier does not appear when called by parse_merge_statement()
            self.next_token()?; // INSERT -> identifier
            insert.push_node("target_name", self.parse_identifier()?);
        }
        if self.get_token(1)?.is("(") {
            self.next_token()?; // identifier -> (
            insert.push_node("columns", self.parse_grouped_exprs(false)?);
        }
        if self.get_token(1)?.is("VALUES") {
            self.next_token()?; // ) -> values
            let mut values = self.construct_node(NodeType::KeywordWithExprs)?;
            let mut lparens = Vec::new();
            while self.get_token(1)?.is("(") {
                self.next_token()?; // VALUES -> (, ',' -> (
                let mut lparen = self.parse_grouped_exprs(false)?;
                if self.get_token(1)?.is(",") {
                    self.next_token()?; // ) -> ,
                    lparen.push_node("comma", self.construct_node(NodeType::Symbol)?);
                }
                lparens.push(lparen);
            }
            values.push_node_vec("exprs", lparens);
            insert.push_node("input", values);
        } else if self.get_token(1)?.is("ROW") {
            self.next_token()?; // -> ROW
            insert.push_node("input", self.construct_node(NodeType::Keyword)?);
        } else {
            self.next_token()?; // ) -> SELECT
            insert.push_node("input", self.parse_select_statement(false, true)?);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            insert.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(insert)
    }
    fn parse_delete_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut delete = self.construct_node(NodeType::DeleteStatement)?;
        if self.get_token(1)?.is("FROM") {
            self.next_token()?; // DELETE -> FROM
            delete.push_node("from", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> table_name
        let mut table_name = self.parse_identifier()?;
        if !self.get_token(1)?.is("WHERE") {
            self.next_token()?; // -> AS, ident
            if self.get_token(0)?.is("AS") {
                table_name.push_node("as", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // AS -> ident
            }
            table_name.push_node("alias", self.construct_node(NodeType::Identifier)?);
        }
        delete.push_node("table_name", table_name);
        self.next_token()?; // -> WHERE
        let mut where_ = self.construct_node(NodeType::KeywordWithExpr)?;
        self.next_token()?; // WHERE -> expr
        where_.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        delete.push_node("where", where_);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            delete.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(delete)
    }
    fn parse_truncate_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut truncate = self.construct_node(NodeType::TruncateStatement)?;
        self.next_token()?; // TRUNCATE -> TABLE
        truncate.push_node("table", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // TABLE -> ident
        truncate.push_node("table_name", self.parse_identifier()?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            truncate.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(truncate)
    }
    fn parse_update_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut update = self.construct_node(NodeType::UpdateStatement)?;
        if !self.get_token(1)?.is("SET") {
            self.next_token()?; // -> table_name
            update.push_node("table_name", self.parse_table(true)?);
        }
        self.next_token()?; // -> SET
        let mut set = self.construct_node(NodeType::KeywordWithExprs)?;
        self.next_token()?; // SET -> exprs
        set.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
        if self.get_token(1)?.is("FROM") {
            self.next_token()?; // exprs -> FROM
            let mut from = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // FROM -> target_name
            from.push_node("expr", self.parse_table(true)?);
            update.push_node("from", from);
        }
        update.push_node("set", set);
        if self.get_token(1)?.is("WHERE") {
            self.next_token()?; // exprs -> WHERE
            let mut where_ = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // WHERE -> expr
            where_.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            update.push_node("where", where_);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            update.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(update)
    }
    fn parse_merge_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut merge = self.construct_node(NodeType::MergeStatement)?;
        if self.get_token(1)?.is("INTO") {
            self.next_token()?; // MERGE -> INTO
            merge.push_node("into", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> table_name
        merge.push_node("table_name", self.parse_table(true)?);
        self.next_token()?; // -> USING
        let mut using = self.construct_node(NodeType::KeywordWithExpr)?;
        self.next_token()?; // USING -> expr
        using.push_node("expr", self.parse_expr(usize::MAX, true, false)?);
        merge.push_node("using", using);
        if self.get_token(1)?.is(";") {
            self.next_token()?; // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        self.next_token()?; // -> ON
        let mut on = self.construct_node(NodeType::KeywordWithExpr)?;
        self.next_token()?; // ON -> expr
        on.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        merge.push_node("on", on);
        let mut whens = Vec::new();
        while self.get_token(1)?.is("when") {
            self.next_token()?; // -> WHEN
            let mut when = self.construct_node(NodeType::WhenClause)?;
            if self.get_token(1)?.is("NOT") {
                self.next_token()?; // WHEN -> NOT
                when.push_node("not", self.construct_node(NodeType::Keyword)?);
            }
            self.next_token()?; // -> MATCHED
            when.push_node("matched", self.construct_node(NodeType::Keyword)?);
            if self.get_token(1)?.is("BY") {
                self.next_token()?; // -> BY
                let by = self.construct_node(NodeType::Keyword)?;
                self.next_token()?; // -> TARGET, SOURCE
                let target = self.construct_node(NodeType::Keyword)?;
                when.push_node_vec("by_target_or_source", vec![by, target]);
            }
            if self.get_token(1)?.is("AND") {
                self.next_token()?; // -> AND
                let mut and = self.construct_node(NodeType::KeywordWithExpr)?;
                self.next_token()?; // -> expr
                let cond = self.parse_expr(usize::MAX, false, false)?;
                and.push_node("expr", cond);
                when.push_node("and", and);
            }
            self.next_token()?; // -> THEN
            let mut then = self.construct_node(NodeType::KeywordWithStatement)?;
            self.next_token()?; // THEN -> stmt
            let stmt = match self.get_token(0)?.literal.to_uppercase().as_str() {
                "DELETE" => self.construct_node(NodeType::SingleTokenStatement)?,
                "UPDATE" => self.parse_update_statement(false)?,
                "INSERT" => self.parse_insert_statement(false)?,
                _ => {
                    return Err(BQ2CSTError::from_token(
                        self.get_token(0)?,
                        format!(
                            "Expected `DELETE`, `UPDATE` or `INSERT` but got: {:?}",
                            self.get_token(0)?
                        ),
                    ))
                }
            };
            then.push_node("stmt", stmt);
            when.push_node("then", then);
            whens.push(when);
        }
        merge.push_node_vec("whens", whens);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            merge.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(merge)
    }
    // ----- DDL -----
    fn parse_create_schema_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut create = self.construct_node(NodeType::CreateSchemaStatement)?;
        self.next_token()?; // -> SCHEMA
        create.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            create.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
        }
        self.next_token()?; // -> ident
        create.push_node("ident", self.parse_identifier()?);
        if self.get_token(1)?.is("OPTIONS") {
            self.next_token()?; // OPTIONS
            create.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(create)
    }
    fn parse_create_table_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut create = self.construct_node(NodeType::CreateTableStatement)?;
        let mut external = false;
        let mut snapshot = false;
        if self.get_token(1)?.is("OR") {
            self.next_token()?; // -> OR
            create.push_node_vec("or_replace", self.parse_n_keywords(2)?);
        }
        // NOTE actually, TEMP is not allowed in CREATE EXTERNAL TABLE statement
        // but it is allowed here for simplicity
        if self.get_token(1)?.in_(&vec!["TEMP", "TEMPORARY"]) {
            self.next_token()?; // -> TEMP
            create.push_node("temp", self.construct_node(NodeType::Keyword)?);
        }
        if self.get_token(1)?.is("EXTERNAL") {
            external = true;
            self.next_token()?; // -> EXTERNAL
            create.push_node("external", self.construct_node(NodeType::Keyword)?);
        }
        if self.get_token(1)?.is("SNAPSHOT") {
            snapshot = true;
            self.next_token()?; // -> SNAPSHOT
            create.push_node("snapshot", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> TABLE
        create.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            create.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
        }
        self.next_token()?; // -> ident
        create.push_node("ident", self.parse_identifier()?);
        if self.get_token(1)?.in_(&vec!["LIKE", "COPY"]) {
            self.next_token()?; // LIKE | COPY
            create.push_node("like_or_copy", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // -> ident
            create.push_node("source_table", self.parse_identifier()?);
        }
        if self.get_token(1)?.is("(") {
            self.next_token()?; // -> (
            create.push_node(
                "column_schema_group",
                self.parse_grouped_type_declarations(true)?,
            );
        }
        // NOTE actually, PARTITION BY has only one expr
        // but for simplicity use parse_xxxby_exprs() here
        if self.get_token(1)?.is("PARTITION") && !external && !snapshot {
            self.next_token()?; // -> PARTITION
            create.push_node("partitionby", self.parse_xxxby_exprs()?);
        }
        if self.get_token(1)?.is("CLUSTER") && !external && !snapshot {
            self.next_token()?; // -> CLUSTER
            create.push_node("clusterby", self.parse_xxxby_exprs()?);
        }
        if self.get_token(1)?.is("WITH") && external {
            self.next_token()?; // -> WITH
            let mut with = self.construct_node(NodeType::WithPartitionColumnsClause)?;
            self.next_token()?; // -> PARTITION
            with.push_node_vec("partition_columns", self.parse_n_keywords(2)?);
            if self.get_token(1)?.is("(") {
                self.next_token()?; // -> (
                with.push_node(
                    "column_schema_group",
                    self.parse_grouped_type_declarations(false)?,
                );
            }
            create.push_node("with_partition_columns", with);
        }
        if self.get_token(1)?.is("CLONE") {
            self.next_token()?; // -> CLONE
            let mut clone = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> identifier
            clone.push_node("expr", self.parse_table(true)?);
            create.push_node("clone", clone);
        }
        if self.get_token(1)?.is("OPTIONS") {
            self.next_token()?; // -> OPTIONS
            create.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        }
        if self.get_token(1)?.is("AS") {
            self.next_token()?; // -> AS
            let mut as_ = self.construct_node(NodeType::KeywordWithStatement)?;
            self.next_token()?; // -> SELECT
            as_.push_node("stmt", self.parse_select_statement(false, true)?);
            create.push_node("as", as_)
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(create)
    }
    fn parse_create_view_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut create = self.construct_node(NodeType::CreateViewStatement)?;
        let mut materialized = false;
        // NOTE actually, OR REPLACE is not allowed in CREATE MATERIALIZED VIEW statement
        // but it is allowed here for simplicity
        if self.get_token(1)?.is("OR") {
            self.next_token()?; // -> OR
            create.push_node_vec("or_replace", self.parse_n_keywords(2)?);
        }
        if self.get_token(1)?.is("MATERIALIZED") {
            materialized = true;
            self.next_token()?; // -> MATERIALIZED
            create.push_node("materialized", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> VIEW
        create.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            create.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
        }
        self.next_token()?; // -> ident
        create.push_node("ident", self.parse_identifier()?);
        if self.get_token(1)?.is("(") && !materialized {
            self.next_token()?; // -> (
            create.push_node("column_name_list", self.parse_grouped_exprs(false)?);
        }
        if self.get_token(1)?.is("PARTITION") && materialized {
            self.next_token()?; // -> PARTITION
            create.push_node("partitionby", self.parse_xxxby_exprs()?);
        }
        if self.get_token(1)?.is("CLUSTER") && materialized {
            self.next_token()?; // -> CLUSTER
            create.push_node("clusterby", self.parse_xxxby_exprs()?);
        }
        if self.get_token(1)?.is("OPTIONS") {
            self.next_token()?; // -> OPTIONS
            create.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        }
        if self.get_token(1)?.is("AS") {
            self.next_token()?; // -> AS
            let mut as_ = self.construct_node(NodeType::KeywordWithStatement)?;
            self.next_token()?; // -> SELECT
            as_.push_node("stmt", self.parse_select_statement(false, true)?);
            create.push_node("as", as_)
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(create)
    }
    fn parse_create_function_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::CreateFunctionStatement)?;
        let mut is_tvf = false;
        if self.get_token(1)?.literal.to_uppercase() == "OR" {
            self.next_token()?; // -> OR
            node.push_node_vec("or_replace", self.parse_n_keywords(2)?);
        }
        if self.get_token(1)?.in_(&vec!["TEMPORARY", "TEMP"]) {
            self.next_token()?; // -> TEMP
            node.push_node("temp", self.construct_node(NodeType::Keyword)?);
        }
        if self.get_token(1)?.is("TABLE") {
            self.next_token()?; // -> TABLE
            node.push_node("table", self.construct_node(NodeType::Keyword)?);
            is_tvf = true;
        }
        self.next_token()?; // -> FUNCTION
        node.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.in_(&vec!["IF"]) {
            self.next_token()?; // -> IF
            node.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
        }
        self.next_token()?; // -> ident
        node.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> (
        node.push_node("group", self.parse_grouped_type_declarations(false)?);
        if self.get_token(1)?.is("RETURNS") {
            self.next_token()?; // -> RETURNS
            let mut returns = self.construct_node(NodeType::KeywordWithType)?;
            self.next_token()?; // -> type
            returns.push_node("type", self.parse_type(false)?);
            node.push_node("returns", returns);
        }
        if self.get_token(1)?.is("AS") {
            // sql function definition
            self.next_token()?; // -> AS
            if is_tvf {
                let mut as_ = self.construct_node(NodeType::KeywordWithStatement)?;
                self.next_token()?; // SELECT
                as_.push_node("stmt", self.parse_select_statement(false, true)?);
                node.push_node("as", as_)
            } else {
                let mut as_ = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
                self.next_token()?; // -> (
                let mut group = self.construct_node(NodeType::GroupedExpr)?;
                self.next_token()?; // ( -> expr
                group.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
                self.next_token()?; // expr -> )
                group.push_node("rparen", self.construct_node(NodeType::Symbol)?);
                as_.push_node("group", group);
                node.push_node("as", as_);
            }
        } else {
            // javascript function definition
            if self.get_token(1)?.in_(&vec!["DETERMINISTIC", "NOT"]) {
                self.next_token()?; // -> DETERMINISTIC | NOT
                if self.get_token(0)?.is("NOT") {
                    node.push_node_vec("determinism", self.parse_n_keywords(2)?);
                } else {
                    node.push_node_vec("determinism", self.parse_n_keywords(1)?);
                }
            }
            self.next_token()?; // -> LANGUAGE
            let mut language = self.construct_node(NodeType::LanguageSpecifier)?;
            self.next_token()?; // -> js
            language.push_node("language", self.construct_node(NodeType::Identifier)?);
            node.push_node("language", language);
            if self.get_token(1)?.is("OPTIONS") {
                self.next_token()?; // -> OPTIONS
                node.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
            }
            self.next_token()?; // -> AS
            let mut as_ = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> javascript_code
            as_.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            node.push_node("as", as_);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // ) -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(node)
    }
    fn parse_create_procedure_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut create = self.construct_node(NodeType::CreateProcedureStatement)?;
        if self.get_token(1)?.is("OR") {
            self.next_token()?; // -> OR
            create.push_node_vec("or_replace", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> PROCEDURE
        create.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            create.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
        }
        self.next_token()?; // -> ident
        create.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> (
        create.push_node("group", self.parse_grouped_type_declarations(true)?);
        if self.get_token(1)?.is("OPTIONS") {
            self.next_token()?; // -> OPTIONS
            create.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        }
        self.next_token()?; // -> BEGIN
        create.push_node("stmt", self.parse_begin_statement(false)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(create)
    }
    fn parse_alter_schema_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut alter = self.construct_node(NodeType::AlterSchemaStatement)?;
        self.next_token()?; // -> SCHEMA
        alter.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            alter.push_node_vec("if_exists", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> ident
        alter.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> SET
        alter.push_node("set", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> OPTIONS
        alter.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(alter)
    }
    fn parse_alter_table_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut alter = self.construct_node(NodeType::AlterTableStatement)?;
        self.next_token()?; // -> TABLE
        alter.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            alter.push_node_vec("if_exists", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> ident
        alter.push_node("ident", self.parse_identifier()?);
        match self.get_token(1)?.literal.to_uppercase().as_str() {
            "SET" => {
                self.next_token()?; // -> SET
                alter.push_node("set", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // -> OPTIONS
                alter.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
            }
            "ADD" => {
                let mut add_columns = Vec::new();
                while self.get_token(1)?.is("ADD") {
                    self.next_token()?; // -> ADD
                    let mut add_column = self.construct_node(NodeType::AddColumnClause)?;
                    self.next_token()?; // -> COLUMN
                    add_column.push_node("column", self.construct_node(NodeType::Keyword)?);
                    if self.get_token(1)?.is("IF") {
                        self.next_token()?; // -> IF
                        add_column.push_node_vec("if_not_exists", self.parse_n_keywords(3)?);
                    }
                    self.next_token()?; // -> ident
                    let mut ident = self.construct_node(NodeType::TypeDeclaration)?;
                    self.next_token()?; // -> type
                    ident.push_node("type", self.parse_type(true)?);
                    add_column.push_node("type_declaration", ident);
                    if self.get_token(1)?.is(",") {
                        self.next_token()?; // -> ,
                        add_column.push_node("comma", self.construct_node(NodeType::Symbol)?);
                    }
                    add_columns.push(add_column);
                }
                alter.push_node_vec("add_columns", add_columns);
            }
            "RENAME" => {
                self.next_token()?; // -> RENAME
                alter.push_node("rename", self.construct_node(NodeType::Keyword)?);
                self.next_token()?; // -> TO
                let mut to = self.construct_node(NodeType::KeywordWithExpr)?;
                self.next_token()?; // -> ident
                to.push_node("expr", self.parse_identifier()?);
                alter.push_node("to", to);
            }
            "DROP" => {
                let mut drop_columns = Vec::new();
                while self.get_token(1)?.is("DROP") {
                    self.next_token()?; // -> DROP
                    let mut drop_column = self.construct_node(NodeType::DropColumnClause)?;
                    self.next_token()?; // -> COLUMN
                    drop_column.push_node("column", self.construct_node(NodeType::Keyword)?);
                    if self.get_token(1)?.is("IF") {
                        self.next_token()?; // -> IF
                        drop_column.push_node_vec("if_exists", self.parse_n_keywords(2)?);
                    }
                    self.next_token()?; // -> ident
                    drop_column.push_node("ident", self.parse_identifier()?);
                    if self.get_token(1)?.is(",") {
                        self.next_token()?; // -> ,
                        drop_column.push_node("comma", self.construct_node(NodeType::Symbol)?);
                    }
                    drop_columns.push(drop_column);
                }
                alter.push_node_vec("drop_columns", drop_columns);
            }
            "ALTER" => {
                self.next_token()?; // -> ALTER
                alter.push_node(
                    "alter_column_stmt",
                    self.parse_alter_column_statement(false)?,
                );
            }
            _ => {
                return Err(BQ2CSTError::from_token(
                    self.get_token(1)?,
                    format!(
                        "Expected `SET`, `ADD` `RENAME` or `DROP` but got: {:?}",
                        self.get_token(1)?
                    ),
                ))
            }
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(alter)
    }
    fn parse_alter_column_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut alter = self.construct_node(NodeType::AlterColumnStatement)?;
        self.next_token()?; // -> COLUMN
        alter.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?;
            alter.push_node_vec("if_exists", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> ident
        alter.push_node("ident", self.construct_node(NodeType::Identifier)?);
        self.next_token()?; // -> SET | DROP
        match self.get_token(0)?.literal.to_uppercase().as_str() {
            "SET" => {
                alter.push_node("set", self.construct_node(NodeType::Keyword)?);
                if self.get_token(1)?.is("OPTIONS") {
                    self.next_token()?; // -> OPTIONS
                    alter.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
                } else {
                    self.next_token()?; // -> DATA
                    alter.push_node_vec("data_type", self.parse_n_keywords(2)?);
                    self.next_token()?; // -> type
                    alter.push_node("type", self.parse_type(false)?);
                }
            }
            "DROP" => {
                alter.push_node_vec("drop_not_null", self.parse_n_keywords(3)?);
            }
            _ => {
                return Err(BQ2CSTError::from_token(
                    self.get_token(0)?,
                    format!(
                        "Expected `SET` or `DROP` but got : {:?}",
                        self.get_token(0)?
                    ),
                ))
            }
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(alter)
    }
    fn parse_alter_view_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut alter = self.construct_node(NodeType::AlterViewStatement)?;
        if self.get_token(1)?.is("MATERIALIZED") {
            self.next_token()?; // -> MATERIALIZED
            alter.push_node("materialized", self.construct_node(NodeType::Keyword)?);
        }
        self.next_token()?; // -> VIEW
        alter.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            alter.push_node_vec("if_exists", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> ident
        alter.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> SET
        alter.push_node("set", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> OPTIONS
        alter.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            alter.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(alter)
    }
    fn parse_drop_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut drop = self.construct_node(NodeType::DropStatement)?;
        if self.get_token(1)?.is("EXTERNAL") {
            self.next_token()?; // -> EXTERNAL
            drop.push_node("external", self.construct_node(NodeType::Keyword)?);
        } else if self.get_token(1)?.is("MATERIALIZED") {
            self.next_token()?; // -> MATERIALIZED
            drop.push_node("materialized", self.construct_node(NodeType::Keyword)?);
        } else if self.get_token(1)?.is("TABLE") && self.get_token(2)?.is("FUNCTION") {
            self.next_token()?; // -> TABLE
            drop.push_node("table", self.construct_node(NodeType::Keyword)?)
        }
        self.next_token()?; // -> SCHEMA, TABLE, VIEW, FUNCTION, PROCEDURE
        drop.push_node("what", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is("IF") {
            self.next_token()?; // -> IF
            drop.push_node_vec("if_exists", self.parse_n_keywords(2)?);
        }
        self.next_token()?; // -> ident
        drop.push_node("ident", self.parse_identifier()?);
        if self.get_token(1)?.in_(&vec!["CASCADE", "RESTRICT"]) {
            self.next_token()?; // -> CASCADE, RESTRICT
            drop.push_node(
                "cascade_or_restrict",
                self.construct_node(NodeType::Keyword)?,
            );
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            drop.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(drop)
    }
    // ----- DCL -----
    fn parse_grant_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut grant = self.construct_node(NodeType::GrantStatement)?;
        self.next_token()?; // -> role
        grant.push_node_vec("roles", self.parse_exprs(&vec![], false)?);
        self.next_token()?; // -> ON
        grant.push_node("on", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> resource_type
        grant.push_node("resource_type", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> ident
        grant.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> TO
        let mut to = self.construct_node(NodeType::KeywordWithExprs)?;
        self.next_token()?; // -> user
        to.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
        grant.push_node("to", to);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // ;
            grant.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(grant)
    }
    fn parse_revoke_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut revoke = self.construct_node(NodeType::RevokeStatement)?;
        self.next_token()?; // -> role
        revoke.push_node_vec("roles", self.parse_exprs(&vec![], false)?);
        self.next_token()?; // -> ON
        revoke.push_node("on", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> resource_type
        revoke.push_node("resource_type", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> ident
        revoke.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // -> FROM
        let mut from = self.construct_node(NodeType::KeywordWithExprs)?;
        self.next_token()?; // -> user
        from.push_node_vec("exprs", self.parse_exprs(&vec![], false)?);
        revoke.push_node("from", from);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // ;
            revoke.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(revoke)
    }
    fn parse_create_reservation_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut create = self.construct_node(NodeType::CreateReservationStatement)?;
        self.next_token()?; // -> CAPACITY | RESERVATION | ASSIGNMENT
        create.push_node("what", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> ident
        create.push_node("ident", self.parse_identifier()?);
        self.next_token()?; // AS
        create.push_node("as", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // JSON
        create.push_node("json", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> '''{}'''
        create.push_node("json_string", self.parse_expr(usize::MAX, false, false)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // ;
            create.push_node("semicolon", self.construct_node(NodeType::Symbol)?)
        }
        Ok(create)
    }
    // ----- script -----
    fn parse_declare_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut declare = self.construct_node(NodeType::DeclareStatement)?;
        let mut idents = Vec::new();
        loop {
            self.next_token()?; // -> ident
            if self.get_token(1)?.is(",") {
                let mut ident = self.parse_identifier()?;
                self.next_token()?; // ident -> comma
                ident.push_node("comma", self.construct_node(NodeType::Symbol)?);
                idents.push(ident);
            } else {
                idents.push(self.parse_identifier()?);
                break;
            }
        }
        declare.push_node_vec("idents", idents);
        if !self.get_token(1)?.is("DEFAULT") {
            self.next_token()?; // ident -> variable_type
            declare.push_node("variable_type", self.parse_type(false)?);
        }
        if self.get_token(1)?.is("DEFAULT") {
            self.next_token()?; // -> DEFAULT
            let mut default = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // DEFAULT -> expr
            default.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            declare.push_node("default", default);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?;
            declare.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(declare)
    }
    fn parse_set_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut set = self.construct_node(NodeType::SetStatement)?;
        self.next_token()?; // set -> expr
        set.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?;
            set.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(set)
    }
    fn parse_execute_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut execute = self.construct_node(NodeType::ExecuteStatement)?;
        self.next_token()?; // EXECUTE -> IMMEDIATE
        execute.push_node("immediate", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // IMMEDIATE -> sql_expr
        execute.push_node("sql_expr", self.parse_expr(usize::MAX, false, false)?);
        if self.get_token(1)?.is("INTO") {
            self.next_token()?; // sql_expr -> INTO
            let mut into = self.construct_node(NodeType::KeywordWithExprs)?;
            let mut idents = Vec::new();
            loop {
                self.next_token()?; // -> ident
                if self.get_token(1)?.is(",") {
                    let mut ident = self.parse_identifier()?;
                    self.next_token()?; // ident -> ,
                    ident.push_node("comma", self.construct_node(NodeType::Symbol)?);
                    idents.push(ident);
                } else {
                    idents.push(self.parse_identifier()?);
                    break;
                }
            }
            into.push_node_vec("exprs", idents);
            execute.push_node("into", into);
        }
        if self.get_token(1)?.is("USING") {
            self.next_token()?; // -> using
            let mut using = self.construct_node(NodeType::KeywordWithExprs)?;
            self.next_token()?; // using -> exprs
            using.push_node_vec("exprs", self.parse_exprs(&vec![], true)?);
            execute.push_node("using", using);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?;
            execute.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(execute)
    }
    fn parse_begin_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut begin = self.construct_node(NodeType::BeginStatement)?;
        let mut stmts = Vec::new();
        while !self.get_token(1)?.in_(&vec!["END", "EXCEPTION"]) {
            self.next_token()?; // -> stmt
            stmts.push(self.parse_statement(true)?);
        }
        if 0 < stmts.len() {
            begin.push_node_vec("stmts", stmts);
        }
        if self.get_token(1)?.is("exception") {
            self.next_token()?; // ; -> EXCEPTION
            let exception = self.construct_node(NodeType::Keyword)?;
            self.next_token()?; // EXCEPTION -> WHEN
            let when = self.construct_node(NodeType::Keyword)?;
            self.next_token()?; // WHEN -> ERROR
            let error = self.construct_node(NodeType::Keyword)?;
            begin.push_node_vec("exception_when_error", vec![exception, when, error]);
            self.next_token()?; // ERROR -> THEN
            begin.push_node("then", self.parse_keyword_with_statements(&vec!["END"])?);
        }
        self.next_token()?; // -> end
        begin.push_node("end", self.construct_node(NodeType::Keyword)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            begin.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(begin)
    }
    fn parse_if_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut if_ = self.construct_node(NodeType::IfStatement)?;
        self.next_token()?; // -> condition
        if_.push_node("condition", self.parse_expr(usize::MAX, false, false)?);

        self.next_token()?; // -> THEN
        if_.push_node(
            "then",
            self.parse_keyword_with_statements(&vec!["ELSEIF", "ELSE", "END"])?,
        );

        let mut elseifs = Vec::new();
        while self.get_token(1)?.is("ELSEIF") {
            self.next_token()?; // -> ELSEIF
            let mut elseif = self.construct_node(NodeType::ElseIfClause)?;
            self.next_token()?; // -> condition
            elseif.push_node("condition", self.parse_expr(usize::MAX, false, false)?);
            self.next_token()?; // -> THEN
            elseif.push_node(
                "then",
                self.parse_keyword_with_statements(&vec!["ELSEIF", "ELSE", "END"])?,
            );
            elseifs.push(elseif);
        }
        if 0 < elseifs.len() {
            if_.push_node_vec("elseifs", elseifs);
        }

        if self.get_token(1)?.is("ELSE") {
            self.next_token()?; // -> ELSE
            if_.push_node("else", self.parse_keyword_with_statements(&vec!["END"])?);
        }
        self.next_token()?; // -> END
        let end = self.construct_node(NodeType::Keyword)?;
        self.next_token()?; // -> IF
        if_.push_node_vec("end_if", vec![end, self.construct_node(NodeType::Keyword)?]);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            if_.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(if_)
    }
    fn parse_labeled_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let label = self.construct_node(NodeType::Identifier)?;
        self.next_token()?; // -> :
        let colon = self.construct_node(NodeType::Symbol)?;
        self.next_token()?; // -> stmt
        let mut stmt = self.parse_statement(false)?;
        if stmt
            .children
            .keys()
            .any(|k| k == "leading_label" || k == "colon")
        {
            return Err(BQ2CSTError::from_token(
                self.get_token(0)?,
                format!(
                    "The statement is not properly labeled: {:?}",
                    self.get_token(0)?
                ),
            ));
        };
        stmt.push_node("leading_label", label);
        stmt.push_node("colon", colon);
        if !self.get_token(1)?.is(";") {
            self.next_token()?; // -> trailing_label
            stmt.push_node("trailing_label", self.construct_node(NodeType::Identifier)?);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            stmt.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(stmt)
    }
    fn parse_break_continue_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::BreakContinueStatement)?;
        if !self.get_token(1)?.is(";") {
            self.next_token()?; // -> label
            node.push_node("label", self.construct_node(NodeType::Identifier)?);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(node)
    }
    fn parse_loop_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut loop_ = self.parse_keyword_with_statements(&vec!["END"])?;
        loop_.node_type = NodeType::LoopStatement;
        self.next_token()?; // -> END
        let end = self.construct_node(NodeType::Keyword)?;
        self.next_token()?; // -> LOOP
        loop_.push_node_vec(
            "end_loop",
            vec![end, self.construct_node(NodeType::Keyword)?],
        );
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            loop_.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(loop_)
    }
    fn parse_repeat_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut repeat = self.parse_keyword_with_statements(&vec!["UNTIL"])?;
        repeat.node_type = NodeType::RepeatStatement;
        self.next_token()?; // -> UNTIL
        let mut until = self.construct_node(NodeType::KeywordWithExpr)?;
        self.next_token()?; // -> expr
        until.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        repeat.push_node("until", until);
        self.next_token()?; // -> END
        repeat.push_node_vec("end_repeat", self.parse_n_keywords(2)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            repeat.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(repeat)
    }
    fn parse_while_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut while_ = self.construct_node(NodeType::WhileStatement)?;
        self.next_token()?; // -> condition
        while_.push_node("condition", self.parse_expr(usize::MAX, false, false)?);
        self.next_token()?; // -> DO
        while_.push_node("do", self.parse_keyword_with_statements(&vec!["END"])?);
        self.next_token()?; // -> END
        let end = self.construct_node(NodeType::Keyword)?;
        self.next_token()?; // -> WHILE
        while_.push_node_vec(
            "end_while",
            vec![end, self.construct_node(NodeType::Keyword)?],
        );
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            while_.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(while_)
    }
    fn parse_single_token_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::SingleTokenStatement)?;
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(node)
    }
    fn parse_for_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut for_ = self.construct_node(NodeType::ForStatement)?;
        self.next_token()?; // -> ident
        for_.push_node("ident", self.construct_node(NodeType::Identifier)?);
        self.next_token()?; // -> IN
        let mut in_ = self.construct_node(NodeType::KeywordWithGroupedXXX)?;
        self.next_token()?; // -> (table_expression)
        in_.push_node("group", self.parse_select_statement(false, true)?);
        for_.push_node("in", in_);
        self.next_token()?; // -> DO
        for_.push_node("do", self.parse_keyword_with_statements(&vec!["END"])?);
        self.next_token()?; // -> END
        for_.push_node_vec("end_for", self.parse_n_keywords(2)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            for_.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(for_)
    }
    fn parse_transaction_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut node = self.construct_node(NodeType::TransactionStatement)?;
        if self.get_token(1)?.is("TRANSACTION") {
            self.next_token()?; // -> TRANSACTION
            node.push_node("transaction", self.construct_node(NodeType::Keyword)?);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            node.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(node)
    }
    fn parse_raise_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut raise = self.construct_node(NodeType::RaiseStatement)?;
        if self.get_token(1)?.is("using") {
            self.next_token()?; // -> USING
            let mut using = self.construct_node(NodeType::KeywordWithExpr)?;
            self.next_token()?; // -> MESSAGE
            using.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            raise.push_node("using", using);
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            raise.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(raise)
    }
    fn parse_case_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut case = self.construct_node(NodeType::CaseStatement)?;
        if !self.get_token(1)?.is("WHEN") {
            self.next_token()?; // -> expr
            case.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        }
        let mut arms = Vec::new();
        while self.get_token(1)?.is("WHEN") {
            self.next_token()?; // -> WHEN
            let mut when = self.construct_node(NodeType::CaseStatementArm)?;
            self.next_token()?; // -> expr
            when.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
            self.next_token()?; // -> THEN
            when.push_node("then", self.construct_node(NodeType::Keyword)?);
            let mut stmts = Vec::new();
            while !self.get_token(1)?.in_(&vec!["WHEN", "ELSE", "END"]) {
                self.next_token()?; // -> stmt
                stmts.push(self.parse_statement(true)?);
            }
            when.push_node_vec("stmts", stmts);
            arms.push(when)
        }
        if self.get_token(1)?.is("ELSE") {
            self.next_token()?; // -> ELSE
            let mut else_ = self.construct_node(NodeType::CaseStatementArm)?;
            let mut stmts = Vec::new();
            while !self.get_token(1)?.is("END") {
                self.next_token()?; // -> stmt
                stmts.push(self.parse_statement(true)?);
            }
            else_.push_node_vec("stmts", stmts);
            arms.push(else_);
        }
        case.push_node_vec("arms", arms);
        self.next_token()?; // -> END
        case.push_node_vec("end_case", self.parse_n_keywords(2)?);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            case.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(case)
    }
    fn parse_call_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut call = self.construct_node(NodeType::CallStatement)?;
        self.next_token()?; // -> procedure_name
        let procedure = self.parse_expr(usize::MAX, false, false)?;
        call.push_node("procedure", procedure);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            call.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(call)
    }
    // ----- debug -----
    fn parse_assert_satement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut assert = self.construct_node(NodeType::AssertStatement)?;
        self.next_token()?; // -> expr
        assert.push_node("expr", self.parse_expr(usize::MAX, false, false)?);
        if self.get_token(1)?.is("AS") {
            self.next_token()?; // -> AS
            assert.push_node("as", self.construct_node(NodeType::Keyword)?);
            self.next_token()?; // -> description
            assert.push_node("description", self.parse_expr(usize::MAX, false, false)?)
        }
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            assert.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(assert)
    }
    // ----- other -----
    fn parse_export_statement(&mut self, semicolon: bool) -> BQ2CSTResult<Node> {
        let mut export = self.construct_node(NodeType::ExportStatement)?;
        self.next_token()?; // -> DATA
        export.push_node("data", self.construct_node(NodeType::Keyword)?);
        self.next_token()?; // -> OPTIONS
        export.push_node("options", self.parse_keyword_with_grouped_exprs(false)?);
        self.next_token()?; // -> AS
        let mut as_ = self.construct_node(NodeType::KeywordWithStatement)?;
        self.next_token()?; // -> stmt
        as_.push_node("stmt", self.parse_statement(false)?);
        export.push_node("as", as_);
        if self.get_token(1)?.is(";") && semicolon {
            self.next_token()?; // -> ;
            export.push_node("semicolon", self.construct_node(NodeType::Symbol)?);
        }
        Ok(export)
    }
}
