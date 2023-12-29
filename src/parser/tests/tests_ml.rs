use super::*;

#[test]
fn test_parse_code_ml() {
    let test_cases = vec![
        // EXPORT MODEL statement
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
        // ALTER MODEL statement
        Box::new(SuccessTestCase::new(
            "\
ALTER MODEL ident SET OPTIONS ()
",
            "\
self: ALTER (AlterModelStatement)
ident:
  self: ident (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
set:
  self: SET (Keyword)
what:
  self: MODEL (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
ALTER MODEL IF EXISTS ident SET OPTIONS (description = '')
",
            "\
self: ALTER (AlterModelStatement)
ident:
  self: ident (Identifier)
if_exists:
- self: IF (Keyword)
- self: EXISTS (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: description (Identifier)
      right:
        self: '' (StringLiteral)
    rparen:
      self: ) (Symbol)
set:
  self: SET (Keyword)
what:
  self: MODEL (Keyword)
",
            0,
        )),
        // DROP statement
        Box::new(SuccessTestCase::new(
            "\
DROP MODEL ident
",
            "\
self: DROP (DropStatement)
ident:
  self: ident (Identifier)
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
