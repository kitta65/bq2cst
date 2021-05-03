use crate::token;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Children {
    Node(Node),
    NodeVec(Vec<Node>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub token: Option<token::Token>,
    pub children: HashMap<String, Children>,
}

impl Node {
    pub fn new(token: token::Token) -> Node {
        Node {
            token: Some(token),
            children: HashMap::new(),
        }
    }
    pub fn new_none() -> Node {
        Node {
            token: None,
            children: HashMap::new(),
        }
    }
    pub fn to_string(&self, indent: usize, is_array: bool) -> String {
        let mut res = Vec::new();
        // push `self: xxx` or `- self: xxx`
        if is_array {
            res.push(format!(
                "{}- self: {}",
                " ".repeat((indent - 1) * 2),
                match self.token.clone() {
                    Some(token) => token.literal.clone(),
                    None => "None".to_string(),
                },
            ))
        } else {
            res.push(format!(
                "{}self: {}",
                " ".repeat(indent * 2),
                match self.token.clone() {
                    Some(token) => token.literal.clone(),
                    None => "None".to_string(),
                },
            ))
        }
        // prepare keys
        let mut keys = Vec::new();
        for key in self.children.keys() {
            keys.push(key);
        }
        keys.sort();
        for key in keys {
            if key == "self" {
                continue;
            }
            // push `key:`
            res.push(format!("{}{}:", " ".repeat(indent * 2), key.clone()));
            let child_string = match self.children.get(key).unwrap() {
                Children::Node(n) => n.to_string(indent + 1, false),
                Children::NodeVec(ns) => {
                    let mut children = Vec::new();
                    for n in ns {
                        children.push(n.to_string(indent + 1, true));
                    }
                    children.join("\n")
                }
            };
            res.push(child_string);
        }
        res.join("\n")
    }
    pub fn push_node(&mut self, key: &str, node: Node) {
        self.children.insert(key.to_string(), Children::Node(node));
    }
    pub fn push_node_vec(&mut self, key: &str, nodes: Vec<Node>) {
        self.children
            .insert(key.to_string(), Children::NodeVec(nodes));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn new_node(literal: &str) -> Node {
        // this function is only used in test_to_string.
        // so line and column are 0.
        Node {
            token: Some(token::Token::from_str(0, 0, literal)),
            children: HashMap::new(),
        }
    }
    #[test]
    fn test_to_string() {
        let mut root = new_node("root");
        root.push_node_vec("key2", vec![new_node("child2"), new_node("child3")]);
        root.push_node("key1", new_node("child1"));
        assert_eq!(
            root.to_string(0, false),
            "\
self: root
key1:
  self: child1
key2:
- self: child2
- self: child3"
        )
    }
}
