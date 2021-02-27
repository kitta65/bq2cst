mod token;
mod cst;
mod lexer;
mod parser;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(code: String) -> JsValue {
    let l = lexer::Lexer::new(code);
    let mut p = parser::Parser::new(l);
    let stmts = p.parse_code();
    JsValue::from_serde(&stmts).unwrap()
}

