use super::*;

#[test]
fn test_parse_code_pipe() {
    let test_cases = vec![
        Box::new(SuccessTestCase::new(
            "\
FROM table
",
            "\
self: FROM (FromStatement)
expr:
  self: table (Identifier)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM table;
",
            "\
self: FROM (FromStatement)
expr:
  self: table (Identifier)
semicolon:
  self: ; (Symbol)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM table |> SELECT col
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: table (Identifier)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: col (Identifier)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM table |> SELECT col |> SELECT col;
",
            "\
self: |> (PipeStatement)
left:
  self: |> (PipeStatement)
  left:
    self: FROM (FromStatement)
    expr:
      self: table (Identifier)
  right:
    self: SELECT (BasePipeOperator)
    exprs:
    - self: col (Identifier)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: col (Identifier)
semicolon:
  self: ; (Symbol)
",
            0,
        )),
    ];
    for t in test_cases {
        t.test();
    }
}
