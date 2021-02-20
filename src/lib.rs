mod ast;
mod lexer;
mod parser;
mod token;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize, Deserialize)]
pub struct SelectClause {
    pub distinct: bool,
    pub name: String,
    pub map: HashMap<String, usize>
}

#[derive(Serialize, Deserialize)]
pub struct Cst {
    pub token: Node,
    pub children: HashMap<String, Cst>,
}

//#[wasm_bindgen]
//impl SelectClause {
//    #[wasm_bindgen(getter)]
//    pub fn name(&self) -> String {
//        self.name.clone()
//    }
//    #[wasm_bindgen(setter)]
//    pub fn set_name(&mut self, name: String) {
//        self.name = name;
//    }
//}

#[derive(Serialize, Deserialize)]
pub struct Node { pub s: String }

#[wasm_bindgen]
pub fn parse(code: String) -> JsValue {
    //let mut sc = SelectClause {
    //    distinct: true,
    //    name: String::from("aaa"),
    //    map: HashMap::new(),
    //};
    //sc.map.insert("test".to_string(), 9999);
    let child_cst = Cst {
        token: Node {s: "children".to_string()},
        children: HashMap::new(),
    };
    let mut cst = Cst {
        token: Node {s: "parent".to_string()},
        children: HashMap::new(),
    };
    cst.children.insert("test".to_string(), child_cst);
    JsValue::from_serde(&cst).unwrap()
}

