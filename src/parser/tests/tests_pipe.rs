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
        // ----- select statement -----
        Box::new(SuccessTestCase::new(
            "\
SELECT 1
|> SELECT *
",
            "\
self: |> (PipeStatement)
left:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: * (Asterisk)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
(SELECT 1)
|> SELECT *
",
            "\
self: |> (PipeStatement)
left:
  self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: * (Asterisk)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
SELECT 1 UNION ALL
SELECT 2
|> SELECT *
",
            "\
self: |> (PipeStatement)
left:
  self: UNION (SetOperator)
  distinct_or_all:
    self: ALL (Keyword)
  left:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
  right:
    self: SELECT (SelectStatement)
    exprs:
    - self: 2 (NumericLiteral)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: * (Asterisk)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
(select 1) order by 1 limit 1
|> SELECT *
",
            "\
self: |> (PipeStatement)
left:
  self: ( (GroupedStatement)
  limit:
    self: limit (LimitClause)
    expr:
      self: 1 (NumericLiteral)
  orderby:
    self: order (XXXByExprs)
    by:
      self: by (Keyword)
    exprs:
    - self: 1 (NumericLiteral)
  rparen:
    self: ) (Symbol)
  stmt:
    self: select (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: * (Asterisk)
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
        Box::new(SuccessTestCase::new(
            "\
(FROM table)
",
            "\
self: ( (GroupedStatement)
rparen:
  self: ) (Symbol)
stmt:
  self: FROM (FromStatement)
  expr:
    self: table (Identifier)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
(FROM table AS t) ORDER BY 1 LIMIT 1
|> SELECT *
         ",
            "\
self: |> (PipeStatement)
left:
  self: ( (GroupedStatement)
  limit:
    self: LIMIT (LimitClause)
    expr:
      self: 1 (NumericLiteral)
  orderby:
    self: ORDER (XXXByExprs)
    by:
      self: BY (Keyword)
    exprs:
    - self: 1 (NumericLiteral)
  rparen:
    self: ) (Symbol)
  stmt:
    self: FROM (FromStatement)
    expr:
      self: table (Identifier)
      alias:
        self: t (Identifier)
      as:
        self: AS (Keyword)
right:
  self: SELECT (BasePipeOperator)
  exprs:
  - self: * (Asterisk)
",
            0,
        )),
        // ----- base pipe operator -----
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
        Box::new(SuccessTestCase::new(
            // alias is array... but do not mind it!
            "\
FROM t |> AS u
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: AS (BasePipeOperator)
  exprs:
  - self: u (Identifier)
",
            0,
        )),
        // single keyword
        Box::new(SuccessTestCase::new(
            "\
FROM t |> ORDER BY col1 DESC NULLS LAST, col2
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: ORDER (BasePipeOperator)
  exprs:
  - self: col1 (Identifier)
    comma:
      self: , (Symbol)
    null_order:
    - self: NULLS (Keyword)
    - self: LAST (Keyword)
    order:
      self: DESC (Keyword)
  - self: col2 (Identifier)
  keywords:
    self: BY (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM t |> UNION ALL (SELECT 1), (SELECT 2);
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: UNION (BasePipeOperator)
  exprs:
  - self: ( (GroupedStatement)
    comma:
      self: , (Symbol)
    rparen:
      self: ) (Symbol)
    stmt:
      self: SELECT (SelectStatement)
      exprs:
      - self: 1 (NumericLiteral)
  - self: ( (GroupedStatement)
    rparen:
      self: ) (Symbol)
    stmt:
      self: SELECT (SelectStatement)
      exprs:
      - self: 2 (NumericLiteral)
  keywords:
    self: ALL (Keyword)
semicolon:
  self: ; (Symbol)
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
        // ----- limit pipe operator -----
        Box::new(SuccessTestCase::new(
            "\
FROM t |> LIMIT 1
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: LIMIT (LimitPipeOperator)
  exprs:
  - self: 1 (NumericLiteral)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM t |> LIMIT 1 OFFSET 2;
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: LIMIT (LimitPipeOperator)
  exprs:
  - self: 1 (NumericLiteral)
  offset:
    self: OFFSET (KeywordWithExpr)
    expr:
      self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
            0,
        )),
        // ----- aggregate pipe operator -----
        Box::new(SuccessTestCase::new(
            "\
FROM t |> AGGREGATE COUNT(*) AS cnt DESC NULLS LAST
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: AGGREGATE (AggregatePipeOperator)
  exprs:
  - self: ( (CallingFunction)
    alias:
      self: cnt (Identifier)
    args:
    - self: * (Asterisk)
    as:
      self: AS (Keyword)
    func:
      self: COUNT (Identifier)
    null_order:
    - self: NULLS (Keyword)
    - self: LAST (Keyword)
    order:
      self: DESC (Keyword)
    rparen:
      self: ) (Symbol)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
FROM t
|>
  AGGREGATE COUNT(*)
  GROUP BY col1 AS col_a DESC NULLS LAST, col2
",
            "\
self: |> (PipeStatement)
left:
  self: FROM (FromStatement)
  expr:
    self: t (Identifier)
right:
  self: AGGREGATE (AggregatePipeOperator)
  exprs:
  - self: ( (CallingFunction)
    args:
    - self: * (Asterisk)
    func:
      self: COUNT (Identifier)
    rparen:
      self: ) (Symbol)
  groupby:
    self: GROUP (GroupByExprs)
    by:
      self: BY (Keyword)
    exprs:
    - self: col1 (Identifier)
      alias:
        self: col_a (Identifier)
      as:
        self: AS (Keyword)
      comma:
        self: , (Symbol)
      null_order:
      - self: NULLS (Keyword)
      - self: LAST (Keyword)
      order:
        self: DESC (Keyword)
    - self: col2 (Identifier)
",
            0,
        )),
    ];
    for t in test_cases {
        t.test();
    }
}
