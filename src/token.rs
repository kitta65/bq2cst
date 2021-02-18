#[derive(PartialEq, Debug)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub literal: String,
}

impl Token {
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
        } else {
            match self.literal.as_str() {
                "SELECT" => false,
                "CREATE" => false,
                _ => true,
            }
        }
    }
    pub fn is_comment(&self) -> bool {
        let mut iter = self.literal.chars();
        let first_char = iter.next().unwrap();
        let second_char = iter.next().unwrap();
        if first_char == '#' {
            true
        } else if first_char == '-' || second_char == '-' {
            true
        } else {
            false
        }

    }
    fn quoted_by(&self, ch: char) -> bool {
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
