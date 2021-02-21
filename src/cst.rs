use crate::token;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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

