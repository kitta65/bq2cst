mod constants;
mod cst;
mod error;
mod lexer;
mod parser;
mod token;
mod types;
mod utils;

use wasm_bindgen::prelude::*;

// you can ignore diagnostic message
// https://github.com/rustwasm/wasm-bindgen/issues/2882
#[wasm_bindgen(skip_typescript)]
pub fn parse(code: String) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    let l = lexer::Lexer::new(code);
    let mut p = parser::Parser::new(match l.tokenize_code() {
        Ok(tokens) => tokens,
        Err(bq2cst_error) => {
            return Err(serde_wasm_bindgen::to_value(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    });
    let stmts = match p.parse_code() {
        Ok(stmts) => stmts,
        Err(bq2cst_error) => {
            return Err(serde_wasm_bindgen::to_value(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    };
    Ok(serde_wasm_bindgen::to_value(&stmts).expect("Problem converting stmts to json."))
}

#[wasm_bindgen(skip_typescript)]
pub fn tokenize(code: String) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    let l = lexer::Lexer::new(code);
    let tokens = match l.tokenize_code() {
        Ok(tokens) => tokens,
        Err(bq2cst_error) => {
            return Err(serde_wasm_bindgen::to_value(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    };
    Ok(serde_wasm_bindgen::to_value(&tokens).expect("Problem converting tokens to json."))
}
