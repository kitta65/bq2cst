#[cfg(test)]
mod tests;

use crate::token::Token;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Node(Node),
    NodeVec(Vec<Node>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    AddColumnClause, // ADD COLUMN x INT64 OPTIONS()
    AlterSchemaStatement,
    AlterTableStatement,
    AlterViewStatement,
    ArrayAccessing,
    ArrayLiteral,   // [1, 2]
    BinaryOperator, // + | - | = | ...
    BeginStatement,
    BetweenOperator,
    BooleanLiteral,          // TRUE | FALSE
    CallingFunction,         // (
    CallingDatePartFunction, // WEEK(SUNDAY)
    CallStatement,           // CALL procedure_name (arg);
    CaseArm,                 // WHEN a THEN b
    CaseExpr,                // CASE WHEN a then b ELSE c END
    CastArgument,            // x AS INT64
    CreateFunctionStatement,
    CreateProcedureStatement,
    CreateSchemaStatement,
    CreateTableStatement,
    CreateViewStatement,
    Comment,
    DeclareStatement,
    DeleteStatement,
    DropColumnClause, // DROP COLUMN x
    DropStatement,
    EOF,
    ExecuteStatement,        // EXECUTE IMMEDIATE 'SELECT 1;'
    ExtractArgument,         // DAY FROM expr
    ForSystemTimeAsOfClause, // FOR SYSTEM_TIME AS OF ts
    GroupedExpr,             // (1)
    GroupedExprs,            // (1, 2, 3)
    GroupedStatement,        // (SELECT 1)
    GroupedType,             // <INT64>
    GroupedTypeDeclarations, // <x INT64, y FLOAT64> | (x INT64, y FLOAT64)
    Keyword,
    KeywordWithExpr,         // WHEN expr
    KeywordWithExprs,        // USING 1, 2
    KeywordWithType,         // RETURNS INT64
    KeywordWithGroupedExpr,  // AS (1 + 1)
    KeywordWithGroupedExprs, // EXCEPT (c1, c2)
    KeywordWithStatement,    // THEN INSERT ROW
    KeywordWithStatements,   // THEN SELECT 1;
    Identifier,
    IfStatement,
    InsertStatement,
    InOperator,
    JoinOperator,      // JOIN | ,
    LanguageSpecifier, // LANGUAGE js
    LimitClause,
    LoopStatement,
    MergeStatement,
    NumericLiteral, // 1 | 1.1 | .1E10
    NullLiteral,
    OnClause,   // on x.c1 = y.c2
    OverCaluse, // OVER (PARTITON BY x, y)
    RaiseStatement,
    SelectStatement,      // SELECT 1;
    SetOperator,          // UNION | INTERSECT | EXCEPT
    SetStatement,         // SET x = 5
    SingleTokenStatement, // BREAK; | LEAVE; | ...
    StringLiteral,
    StructLiteral,
    Symbol,            // ) | ] | * | ...
    TableSampleCaluse, // TABLESAMPLE SYSTEM (10 PERCENT)
    TableSampleRatio,  // (10 PERCENT)
    TruncateStatement,
    Type,            // INT64
    TypeDeclaration, // x INT64
    UnaryOperator,   // - | + | TIMESTAMP | ...
    Unknown,
    UpdateStatement,
    WhenClause, // WHEN MATCHED THEN DELETE
    WhileStatement,
    WindowClause,               // WINDOW x AS (PARTITION BY c1)
    WindowExpr,                 // x AS (PARTITION BY c1 ORDER BY c2)
    WindowFrameClause,          // ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING
    WindowSpecification,        // PARTITION BY c1 ORDER BY c2 ROWS UNBOUNDED PRECEDING
    WithClause,                 // WITH x AS (SELECT 1)
    WithPartitionColumnsClause, // WITH PARTITION COLUMNS (c1 INT64, c2 FLOAT64)
    WithQuery,                  // x AS (SELECT 1)
    XXXByExprs,                 // ORDER BY expr
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub token: Option<Token>,
    pub node_type: NodeType,
    children: HashMap<String, ContentType>,
}

impl Node {
    pub fn new(token: Token, node_type: NodeType) -> Node {
        Node {
            token: Some(token),
            node_type,
            children: HashMap::new(),
        }
    }
    pub fn empty(node_type: NodeType) -> Node {
        Node {
            token: None,
            node_type,
            children: HashMap::new(),
        }
    }
    fn format(&self, indent: usize, is_array: bool) -> String {
        let mut res = Vec::new();
        // self & node_type
        let literal = match self.token.clone() {
            Some(t) => t.literal,
            None => "None".to_string(),
        };
        let self_;
        if is_array {
            self_ = format!("{}- self: {}", " ".repeat((indent - 1) * 2), literal);
        } else {
            self_ = format!("{}self: {}", " ".repeat(indent * 2), literal);
        }
        let type_ = format!("{:?}", self.node_type);
        res.push(format!("{} ({})", self_, type_));
        // children
        let mut keys: Vec<&String> = self.children.keys().collect();
        keys.sort();
        for k in keys {
            match self.children.get(k) {
                Some(ContentType::Node(n)) => {
                    res.push(format!("{}{}:", " ".repeat(indent * 2), k));
                    res.push(n.format(indent + 1, false));
                }
                Some(ContentType::NodeVec(ns)) => {
                    res.push(format!("{}{}:", " ".repeat(indent * 2), k));
                    for n in ns {
                        res.push(n.format(indent + 1, true));
                    }
                }
                None => panic!(),
            }
        }
        res.join("\n")
    }
    pub fn push_node(&mut self, key: &str, node: Node) {
        self.children
            .insert(key.to_string(), ContentType::Node(node));
    }
    pub fn push_node_vec(&mut self, key: &str, nodes: Vec<Node>) {
        self.children
            .insert(key.to_string(), ContentType::NodeVec(nodes));
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.format(0, false))
    }
}
