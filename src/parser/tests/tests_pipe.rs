use super::*;

#[test]
fn test_parse_code_pipe() {
    let test_cases = vec![
        // ----- simple pipe syntax -----
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
        // ----- from statement -----
        Box::new(SuccessTestCase::new(
            "\
FROM tabe AS t1
JOIN table AS p2 USING (col)
",
            "\
self: FROM (FromStatement)
expr:
  self: JOIN (JoinOperator)
  left:
    self: tabe (Identifier)
    alias:
      self: t1 (Identifier)
    as:
      self: AS (Keyword)
  right:
    self: table (Identifier)
    alias:
      self: p2 (Identifier)
    as:
      self: AS (Keyword)
  using:
    self: ( (CallingFunction)
    args:
    - self: col (Identifier)
    func:
      self: USING (Identifier)
    rparen:
      self: ) (Symbol)
",
            0,
        )),
        // ----- select pipe operator -----
        Box::new(SuccessTestCase::new(
            // trailing comma is allowed
            "\
FROM t |> SELECT col1, col2,
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: col1 (Identifier)
    comma:
      self: , (Symbol)
  - self: col2 (Identifier)
    comma:
      self: , (Symbol)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM t |> SELECT DISTINCT col
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: col (Identifier)
  keywords:
    self: DISTINCT (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM t |> SELECT ALL AS STRUCT col
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: col (Identifier)
  keywords:
    self: ALL (KeywordSequence)
    next_keyword:
      self: AS (KeywordSequence)
      next_keyword:
        self: STRUCT (Keyword)
",
            0,
        )),
        // ----- extend pipe operator -----
        Box::new(SuccessTestCase::new(
            "\
FROM t |> EXTEND 1 AS one, 2 AS two,;
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: EXTEND (BasePipeOperator)
  exprs:
  - self: 1 (NumericLiteral)
    alias:
      self: one (Identifier)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
    alias:
      self: two (Identifier)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
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
