use super::*;

#[test]
fn test_parse_code_other() {
    let test_cases = vec![
        // ----- EXPORT statement -----
        TestCase::new(
            "\
EXPORT DATA OPTIONS(
  uri = 'gs://bucket/folder/*.csv',
  format = 'CSV'
) AS SELECT 1;
",
            "\
self: EXPORT (ExportStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
data:
  self: DATA (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      comma:
        self: , (Symbol)
      left:
        self: uri (Identifier)
      right:
        self: 'gs://bucket/folder/*.csv' (StringLiteral)
    - self: = (BinaryOperator)
      left:
        self: format (Identifier)
      right:
        self: 'CSV' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}
