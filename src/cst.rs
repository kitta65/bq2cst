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
    Unknown, // TODO develop only
    SelectStatement,
    GroupedStatement,
    SetOperator,
    Comment,
    Keyword,
    Symbol,
    EOF,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub token: Option<Token>,
    node_type: NodeType,
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
