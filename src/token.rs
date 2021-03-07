use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub literal: String,
}

impl Token {
    pub fn new(line: usize, column: usize, literal: &str) -> Token {
        Token {
            line,
            column,
            literal: literal.to_string(),
        }
    }
    pub fn is_string(&self) -> bool {
        if self.quoted_by('"') {
            true
        } else if self.quoted_by('\'') {
            true
        } else {
            false
        }
    }
    pub fn is_identifier(&self) -> bool {
        if self.quoted_by('`') {
            return true;
        }
        !vec![
            "AND",
            "ANY",
            "ARRAY",
            "AS",
            "ASC",
            "ASSERT_ROWS_MODIFIED",
            "AT",
            "BETWEEN",
            "BY",
            "CASE",
            "CAST",
            "COLLATE",
            "CONTAINS",
            "CREATE",
            "CROSS",
            "CUBE",
            "CURRENT",
            "DEFAULT",
            "DEFINE",
            "DESC",
            "DISTINCT",
            "ELSE",
            "END",
            "ENUM",
            "ESCAPE",
            "EXCEPT",
            "EXCLUDE",
            "EXISTS",
            "EXTRACT",
            "FALSE",
            "FETCH",
            "FOLLOWING",
            "FOR",
            "FROM",
            "FULL",
            "GROUP",
            "GROUPING",
            "GROUPS",
            "HASH",
            "HAVING",
            "IF",
            "IGNORE",
            "IN",
            "INNER",
            "INTERSECT",
            "INTERVAL",
            "INTO",
            "IS",
            "JOIN",
            "LATERAL",
            "LEFT",
            "LIKE",
            "LIMIT",
            "LOOKUP",
            "MERGE",
            "NATURAL",
            "NEW",
            "NO",
            "NOT",
            "NULL",
            "NULLS",
            "OF",
            "ON",
            "OR",
            "ORDER",
            "OUTER",
            "OVER",
            "PARTITION",
            "PRECEDING",
            "PROTO",
            "RANGE",
            "RECURSIVE",
            "RESPECT",
            "RIGHT",
            "ROLLUP",
            "ROWS",
            "SELECT",
            "SET",
            "SOME",
            "STRUCT",
            "TABLESAMPLE",
            "THEN",
            "TO",
            "TREAT",
            "TRUE",
            "UNBOUNDED",
            "UNION",
            "UNNEST",
            "USING",
            "WHEN",
            "WHERE",
            "WINDOW",
            "WITH",
            "WITHIN",
            ";",
            ")",
            "]",
            ">",
            ",",
            "",
        ]
        .contains(&self.literal.to_uppercase().as_str())
    }
    pub fn is_comment(&self) -> bool {
        let mut iter = self.literal.chars();
        let first_char = match iter.next() {
            Some(c) => match c {
                '#' => {
                    return true;
                }
                '-' => c,
                '/' => c,
                _ => {
                    return false;
                }
            },
            None => {
                return false;
            }
        };
        let second_char = match iter.next() {
            Some(c) => c,
            None => {
                return false;
            }
        };
        if first_char == '-' && second_char == '-' || first_char == '/' && second_char == '*' {
            true
        } else {
            false
        }
    }
    pub fn is_prefix(&self) -> bool {
        match self.literal.as_str() {
            "-" => true,
            "!" => true,
            _ => false,
        }
    }
    fn quoted_by(&self, ch: char) -> bool {
        if self.literal.len() < 2 {
            return false;
        }
        self.literal.chars().next().unwrap() == ch
            && self.literal.chars().rev().next().unwrap() == ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn str2token(s: &str) -> Token {
        Token {
            line: 0,
            column: 0,
            literal: s.to_string(),
        }
    }
    #[test]
    fn test_is_string() {
        assert!(str2token("'abc'").is_string());
        assert!(str2token("\"abc\"").is_string());
        assert!(str2token("`SELECT`").is_identifier());
        assert!(!str2token("SELECT").is_identifier());
        assert!(str2token("-- comment").is_comment());
    }
}
