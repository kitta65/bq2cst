use super::*;
use difference::Changeset;

mod tests_common;
mod tests_select;
mod tests_script;

struct TestCase {
    code: String,
    expected_output: String,
}

impl TestCase {
    pub fn new(code: &str, expected_output: &str) -> TestCase {
        TestCase {
            code: code.to_string(),
            expected_output: expected_output.to_string(),
        }
    }
    pub fn test(&self) {
        let mut p = Parser::new(self.code.clone());
        let stmts = p.parse_code();
        println!(
            "\
========== testing ==========
{}
=============================
",
            self.code.trim()
        );
        let result = stmts[0].to_string();
        let changeset = Changeset::new(self.expected_output.as_str(), result.as_str(), "\n");
        println!("{}\n", changeset.to_string());
        assert_eq!(2, stmts.len());
        assert_eq!(self.expected_output, result);
    }
    pub fn test_eof(&self) {
        let mut p = Parser::new(self.code.clone());
        let stmts = p.parse_code();
        println!(
            "\
========== testing ==========
{}
=============================
",
            self.code.trim()
        );
        let result = stmts[1].to_string();
        let changeset = Changeset::new(self.expected_output.as_str(), result.as_str(), "\n");
        println!("{}\n", changeset.to_string());
        assert_eq!(2, stmts.len());
        assert_eq!(self.expected_output, result);
    }
}

