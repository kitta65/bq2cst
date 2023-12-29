use super::*;

#[test]
fn test_parse_code_ml() {
    let test_cases = vec![
        Box::new(SuccessTestCase::new(
            "\
EXPORT MODEL ident
",
            "\
self: EXPORT (ExportModelStatement)
ident:
  self: ident (Identifier)
what:
  self: MODEL (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
EXPORT MODEL ident OPTIONS(uri = '');
",
            "\
self: EXPORT (ExportModelStatement)
ident:
  self: ident (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: uri (Identifier)
      right:
        self: '' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: MODEL (Keyword)
",
            0,
        )),
    ];
    for t in test_cases {
        t.test();
    }
}
