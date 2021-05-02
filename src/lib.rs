mod cst;
mod lexer;
mod parser;
mod token;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(code: String) -> JsValue {
    utils::set_panic_hook();
    let l = lexer::Lexer::new(code);
    let mut p = parser::Parser::new(l);
    let stmts = p.parse_code();
    match JsValue::from_serde(&stmts) {
        Ok(json) => json,
        Err(error) => panic!("Probrem converting struct to json: {:?}", error),
    }
}
