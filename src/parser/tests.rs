use super::*;
use difference::Changeset;

mod tests_core;
mod tests_select;
mod tests_dml;
mod tests_ddl;
mod tests_dcl;
mod tests_script;
mod tests_debug;
mod tests_other;

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
    pub fn test(&self, index: usize) {
        let p = Parser::new(self.code.clone());
        let stmts = p.parse_code();
        println!(
            "\
========== testing ==========
{}
=============================
",
            self.code.trim()
        );
        let result = stmts[index].to_string();
        let changeset = Changeset::new(self.expected_output.as_str(), result.as_str(), "\n");
        println!("{}\n", changeset.to_string());
        assert_eq!(self.expected_output, result);
    }
}

