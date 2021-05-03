use super::*;

#[test]
fn test_is_string() {
    assert!(Token::from_str0("'abc'").is_string());
    assert!(Token::from_str0("'''abc'''").is_string());
    assert!(Token::from_str0("\"abc\"").is_string());
    assert!(Token::from_str0("\"\"\"abc\"\"\"").is_string());
}

#[test]
fn test_is_identifier() {
    // valid
    assert!(Token::from_str0("`SELECT`").is_identifier());
    assert!(Token::from_str0("xxx").is_identifier());
    assert!(Token::from_str0("_xxx").is_identifier());
    assert!(Token::from_str0("__").is_identifier());
    assert!(Token::from_str0("x1").is_identifier());

    // invalid
    assert!(!Token::from_str0("SELECT").is_identifier());
    assert!(!Token::from_str0("select").is_identifier());
    assert!(!Token::from_str0("999").is_identifier());
    assert!(!Token::from_str0("").is_identifier());
}

#[test]
fn test_is_comment() {
    assert!(Token::from_str0("-- comment").is_comment());
    assert!(Token::from_str0("/* xxx */").is_comment());
    assert!(Token::from_str0("/*\nxxx\n*/").is_comment());
    assert!(Token::from_str0("# xxx").is_comment());
}
