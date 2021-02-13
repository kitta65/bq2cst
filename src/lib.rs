mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//#[wasm_bindgen]
//pub struct Ast {
//    //pub children: Vec<Ast>,
//    pub type_enum: TypeEnum,
//}
//
//#[wasm_bindgen]
//pub enum TypeEnum {
//    Statement = "statement",
//    Clause = "clause",
//}

#[wasm_bindgen]
pub struct SelectClause {
    pub distinct: bool,
    name: String,
}

#[wasm_bindgen]
impl SelectClause {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[wasm_bindgen]
pub fn greet(int: u32) -> SelectClause {
    SelectClause {
        distinct: true,
        name: String::from("aaa"),
    }
}
