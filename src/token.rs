#[derive(PartialEq, Debug)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub literal: String,
}

//impl Token {
//    fn is_string(&self) {
//        self.literal
//    }
//}
