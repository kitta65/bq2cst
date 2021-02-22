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
    pub token: token::Token,
    pub children: HashMap<String, Children>,
}

impl Node {
    fn to_str(&self, indent: usize) -> &str {
        self.token.literal.as_str()
    }
}

#[cfg(test)]
mod tests {
    fn new_node(literal: &str) -> Node {
        // this function is only used in test_to_string.
        // so line and column are 0.
        Node {
            token: token::Token::new(0, 0, literal),
            children: HashMap::new(),
        }
    }
    use super::*;
    fn test_to_string() {
        let node = new_node("root");
        assert_eq!(node.to_str(0), "root")
    }
}
