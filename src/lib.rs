mod token;
mod cst;
mod lexer;
mod parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(code: String) -> JsValue {
    let l = lexer::Lexer::new(code);
    let mut p = parser::Parser::new(l);
    let stmts = p.parse_code();
    JsValue::from_serde(&stmts).unwrap()
}

