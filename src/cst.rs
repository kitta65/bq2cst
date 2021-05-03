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
    Token(Token),
    Type(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node(pub HashMap<String, ContentType>);

impl Node {
    pub fn new(token: Token, type_: &str) -> Node {
        let mut hash_map = HashMap::new();
        hash_map.insert("self".to_string(), ContentType::Token(token));
        hash_map.insert("type".to_string(), ContentType::Type(type_.to_string()));
        Node(hash_map)
    }
    pub fn empty() -> Node {
        Node(HashMap::new())
    }
    fn format(&self, indent: usize, is_array: bool) -> String {
        // inititalize
        let mut self_ = "".to_string();
        let mut type_ = "".to_string();
        let mut children: Vec<String> = Vec::new();
        // format each value in ABC order
        let mut keys: Vec<&String> = self.0.keys().collect();
        keys.sort();
        for k in keys {
            match self.0.get(k) {
                Some(ContentType::Token(t)) => {
                    let mut literal = t.literal.as_str();
                    if literal == "" {
                        literal = "None"
                    }
                    if is_array {
                        self_ = format!("{}- self: {}", " ".repeat((indent - 1) * 2), literal);
                    } else {
                        self_ = format!("{}self: {}", " ".repeat(indent * 2), literal);
                    }
                }
                Some(ContentType::Type(s)) => {
                    type_ = s.clone();
                }
                Some(ContentType::Node(n)) => {
                    children.push(format!("{}{}:", " ".repeat(indent * 2), k));
                    children.push(n.format(indent + 1, false));
                }
                Some(ContentType::NodeVec(ns)) => {
                    children.push(format!("{}{}:", " ".repeat(indent * 2), k));
                    for n in ns {
                        children.push(n.format(indent + 1, true));
                    }
                }
                None => panic!(),
            }
        }
        let mut res = vec![format!("{} ({})", self_, type_)];
        res.append(&mut children);
        res.join("\n")
    }
    pub fn push_node(&mut self, key: &str, node: Node) {
        self.0.insert(key.to_string(), ContentType::Node(node));
    }
    pub fn push_node_vec(&mut self, key: &str, nodes: Vec<Node>) {
        self.0.insert(key.to_string(), ContentType::NodeVec(nodes));
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(0, false))
    }
}
