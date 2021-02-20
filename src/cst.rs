use crate::token;
use std::collections::HashMap;

pub enum Children {
    Node(Node),
    NodeVec(Vec<Node>),
}

pub struct Node {
    token: token::Token,
    children: HashMap<String, Children>,
}
