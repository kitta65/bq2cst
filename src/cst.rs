use crate::token;
use std::collections::HashMap;

pub enum Children {
    Node(Node),
    NodeVec(Vec<Node>),
}

pub struct Node {
    pub token: token::Token,
    pub children: HashMap<String, Children>,
}

