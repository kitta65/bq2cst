mod constants;
mod cst;
mod lexer;
mod parser;
mod token;
mod types;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(skip_typescript)]
pub fn parse(code: String) -> JsValue {
    utils::set_panic_hook();
    let mut p = parser::Parser::new(code);
    let stmts = p.parse_code();
    match JsValue::from_serde(&stmts) {
        Ok(json) => json,
        Err(error) => panic!("Probrem converting struct to json: {:?}", error),
    }
}

#[wasm_bindgen(skip_typescript)]
pub fn tokenize(code: String) -> JsValue {
    utils::set_panic_hook();
    let mut l = lexer::Lexer::new(code);
    let tokens = l.tokenize_code();
    match JsValue::from_serde(&tokens) {
        Ok(json) => json,
        Err(error) => panic!("Probrem converting Vec to json: {:?}", error),
    }
}
