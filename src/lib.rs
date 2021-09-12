mod constants;
mod cst;
mod error;
mod lexer;
mod parser;
mod token;
mod types;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(skip_typescript)]
pub fn parse(code: String) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    let l = lexer::Lexer::new(code);
    let mut p = parser::Parser::new(match l.tokenize_code() {
        Ok(tokens) => tokens,
        Err(bq2cst_error) => {
            return Err(JsValue::from_serde(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    });
    let stmts = match p.parse_code() {
        Ok(stmts) => stmts,
        Err(bq2cst_error) => {
            return Err(JsValue::from_serde(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    };
    Ok(JsValue::from_serde(&stmts).expect("Problem converting stmts to json."))
}

#[wasm_bindgen(skip_typescript)]
pub fn tokenize(code: String) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    let l = lexer::Lexer::new(code);
    let tokens = match l.tokenize_code() {
        Ok(tokens) => tokens,
        Err(bq2cst_error) => {
            return Err(JsValue::from_serde(&bq2cst_error)
                .expect("Problem converting error struct to json."))
        }
    };
    Ok(JsValue::from_serde(&tokens).expect("Problem converting tokens to json."))
}
