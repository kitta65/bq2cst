use super::*;
#[test]
fn test_parse_code_no_statement() {
    let test_cases = vec![
        TestCase::new(
            "",
            "\
self: None (EOF)
",
        ),
        TestCase::new(
            "\
-- comment
",
            "\
self: None (EOF)
leading_comments:
- self: -- comment (Comment)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}

#[test]
fn test_parse_code_eof() {
    let test_cases = vec![
        TestCase::new(
            "\
SELECT 1;
",
            "\
self: None (EOF)
",
        ),
        TestCase::new(
            "\
SELECT 1;
-- EOF
",
            "\
self: None (EOF)
leading_comments:
- self: -- EOF (Comment)
",
        ),
    ];
    for t in test_cases {
        t.test(1);
    }
}

#[test]
fn test_parse_code_core() {
    let test_cases = vec![
        // ----- comment -----
        TestCase::new(
            "\
#standardSQL
SELECT /* */ 1
; -- end of statement
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 1 (NumericLiteral)
leading_comments:
- self: #standardSQL (Comment)
semicolon:
  self: ; (Symbol)
  trailing_comments:
  - self: -- end of statement (Comment)
trailing_comments:
- self: /* */ (Comment)
",
        ),
        // ----- unary operator -----
        TestCase::new(
            "\
SELECT
  -1,
  +1,
  r'xxx',
  DATE '2020-01-01',
  TIMESTAMP r'2020-01-01',
  NOT TRUE,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: - (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: 1 (NumericLiteral)
- self: + (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: 1 (NumericLiteral)
- self: r (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: 'xxx' (StringLiteral)
- self: DATE (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: '2020-01-01' (StringLiteral)
- self: TIMESTAMP (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: r (UnaryOperator)
    right:
      self: '2020-01-01' (StringLiteral)
- self: NOT (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: TRUE (BooleanLiteral)
",
        ),
        // ----- binary operator -----
        // +, -, *, /
        TestCase::new(
            "\
SELECT
  1 + 2,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: + (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  right:
    self: 2 (NumericLiteral)
",
        ),
        // BETWEEN
        TestCase::new(
            "\
SELECT
  1 BETWEEN 0 AND 3,
  1 NOT BETWEEN 0 AND 3,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: BETWEEN (BetweenOperator)
  and:
    self: AND (Keyword)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  right_max:
    self: 3 (NumericLiteral)
  right_min:
    self: 0 (NumericLiteral)
- self: BETWEEN (BetweenOperator)
  and:
    self: AND (Keyword)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  not:
    self: NOT (Keyword)
  right_max:
    self: 3 (NumericLiteral)
  right_min:
    self: 0 (NumericLiteral)
",
        ),
        // IN
        TestCase::new(
            "\
SELECT
  1 IN (1, 2, 3),
  1 NOT IN (1, 2, 3),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: IN (InOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 3 (NumericLiteral)
    rparen:
      self: ) (Symbol)
- self: IN (InOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  not:
    self: NOT (Keyword)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 3 (NumericLiteral)
    rparen:
      self: ) (Symbol)
",
        ),
        // LIKE
        TestCase::new(
            "\
SELECT
  'x' LIKE '%x%',
  'x' NOT LIKE '%x%',
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: LIKE (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 'x' (StringLiteral)
  right:
    self: '%x%' (StringLiteral)
- self: LIKE (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 'x' (StringLiteral)
  not:
    self: NOT (Keyword)
  right:
    self: '%x%' (StringLiteral)
",
        ),
        // IS
        TestCase::new(
            "\
SELECT
  x IS NULL,
  x IS NOT NULL,
  TRUE IS NOT FALSE,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: IS (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: x (Identifier)
  right:
    self: NULL (NullLiteral)
- self: IS (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: x (Identifier)
  not:
    self: NOT (Keyword)
  right:
    self: NULL (NullLiteral)
- self: IS (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: TRUE (BooleanLiteral)
  not:
    self: NOT (Keyword)
  right:
    self: FALSE (BooleanLiteral)
",
        ),
        // '.' 
        TestCase::new(
            "\
SELECT
  t.struct_col.num + 1,
  1 + t.struct_col.num,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: + (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: . (DotOperator)
    left:
      self: . (DotOperator)
      left:
        self: t (Identifier)
      right:
        self: struct_col (Identifier)
    right:
      self: num (Identifier)
  right:
    self: 1 (NumericLiteral)
- self: + (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (NumericLiteral)
  right:
    self: . (DotOperator)
    left:
      self: . (DotOperator)
      left:
        self: t (Identifier)
      right:
        self: struct_col (Identifier)
    right:
      self: num (Identifier)
",
        ),
        // precedence
        TestCase::new(
            "\
SELECT (1+(-2)) * 3 IN (9)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: IN (InOperator)
  left:
    self: * (BinaryOperator)
    left:
      self: ( (GroupedExpr)
      expr:
        self: + (BinaryOperator)
        left:
          self: 1 (NumericLiteral)
        right:
          self: ( (GroupedExpr)
          expr:
            self: - (UnaryOperator)
            right:
              self: 2 (NumericLiteral)
          rparen:
            self: ) (Symbol)
      rparen:
        self: ) (Symbol)
    right:
      self: 3 (NumericLiteral)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 9 (NumericLiteral)
    rparen:
      self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT (1+2) * 3 NOT BETWEEN 10 + 0 AND 11 + 2 OR TRUE
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: OR (BinaryOperator)
  left:
    self: BETWEEN (BetweenOperator)
    and:
      self: AND (Keyword)
    left:
      self: * (BinaryOperator)
      left:
        self: ( (GroupedExpr)
        expr:
          self: + (BinaryOperator)
          left:
            self: 1 (NumericLiteral)
          right:
            self: 2 (NumericLiteral)
        rparen:
          self: ) (Symbol)
      right:
        self: 3 (NumericLiteral)
    not:
      self: NOT (Keyword)
    right_max:
      self: + (BinaryOperator)
      left:
        self: 11 (NumericLiteral)
      right:
        self: 2 (NumericLiteral)
    right_min:
      self: + (BinaryOperator)
      left:
        self: 10 (NumericLiteral)
      right:
        self: 0 (NumericLiteral)
  right:
    self: TRUE (BooleanLiteral)
",
        ),
        // ----- array -----
        TestCase::new(
            "\
SELECT
  [1, 2],
  ARRAY[1,2],
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: [ (ArrayLiteral)
  comma:
    self: , (Symbol)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
  rparen:
    self: ] (Symbol)
- self: [ (ArrayLiteral)
  comma:
    self: , (Symbol)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
  rparen:
    self: ] (Symbol)
  type:
    self: ARRAY (Type)
",
        ),
        // array with type declaration
        TestCase::new(
            "\
SELECT ARRAY<INT64>[1]
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: [ (ArrayLiteral)
  exprs:
  - self: 1 (NumericLiteral)
  rparen:
    self: ] (Symbol)
  type:
    self: ARRAY (Type)
    type_declaration:
      self: < (GroupedType)
      rparen:
        self: > (Symbol)
      type:
        self: INT64 (Type)
",
        ),
        TestCase::new(
            "\
SELECT ARRAY<STRUCT<INT64, INT64>>[(1,2)]
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: [ (ArrayLiteral)
  exprs:
  - self: ( (StructLiteral)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
  rparen:
    self: ] (Symbol)
  type:
    self: ARRAY (Type)
    type_declaration:
      self: < (GroupedType)
      rparen:
        self: > (Symbol)
      type:
        self: STRUCT (Type)
        type_declaration:
          self: < (GroupedTypeDeclarations)
          declarations:
          - self: None (TypeDeclaration)
            comma:
              self: , (Symbol)
            type:
              self: INT64 (Type)
          - self: None (TypeDeclaration)
            type:
              self: INT64 (Type)
          rparen:
            self: > (Symbol)
",
        ),
        // accessing array
        TestCase::new(
            "\
SELECT arr[OFFSET(1)]
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: [ (ArrayAccessing)
  left:
    self: arr (Identifier)
  right:
    self: ( (CallingFunction)
    args:
    - self: 1 (NumericLiteral)
    func:
      self: OFFSET (Identifier)
    rparen:
      self: ) (Symbol)
  rparen:
    self: ] (Symbol)
",
        ),
    // ----- struct -----
        TestCase::new(
            "\
SELECT
  (1,2),
  STRUCT(1,2),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (StructLiteral)
  comma:
    self: , (Symbol)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
  rparen:
    self: ) (Symbol)
- self: ( (StructLiteral)
  comma:
    self: , (Symbol)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
  rparen:
    self: ) (Symbol)
  type:
    self: STRUCT (Type)
",
        ),
        // struct with type declarations
        TestCase::new(
            "\
SELECT STRUCT<INT64>(1)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (StructLiteral)
  exprs:
  - self: 1 (NumericLiteral)
  rparen:
    self: ) (Symbol)
  type:
    self: STRUCT (Type)
    type_declaration:
      self: < (GroupedTypeDeclarations)
      declarations:
      - self: None (TypeDeclaration)
        type:
          self: INT64 (Type)
      rparen:
        self: > (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT STRUCT<ARRAY<INT64>, x FLOAT64>([1], .1)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (StructLiteral)
  exprs:
  - self: [ (ArrayLiteral)
    comma:
      self: , (Symbol)
    exprs:
    - self: 1 (NumericLiteral)
    rparen:
      self: ] (Symbol)
  - self: .1 (NumericLiteral)
  rparen:
    self: ) (Symbol)
  type:
    self: STRUCT (Type)
    type_declaration:
      self: < (GroupedTypeDeclarations)
      declarations:
      - self: None (TypeDeclaration)
        comma:
          self: , (Symbol)
        type:
          self: ARRAY (Type)
          type_declaration:
            self: < (GroupedType)
            rparen:
              self: > (Symbol)
            type:
              self: INT64 (Type)
      - self: x (TypeDeclaration)
        type:
          self: FLOAT64 (Type)
      rparen:
        self: > (Symbol)
",
        ),
        // ----- case expr -----
        TestCase::new(
            "\
SELECT CASE c1 WHEN 1 THEN 'one' WHEN 2 THEN 'two' ELSE NULL END
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: CASE (CaseExpr)
  arms:
  - self: WHEN (CaseArm)
    expr:
      self: 1 (NumericLiteral)
    result:
      self: 'one' (StringLiteral)
    then:
      self: THEN (Keyword)
  - self: WHEN (CaseArm)
    expr:
      self: 2 (NumericLiteral)
    result:
      self: 'two' (StringLiteral)
    then:
      self: THEN (Keyword)
  - self: ELSE (CaseArm)
    result:
      self: NULL (NullLiteral)
  end:
    self: END (Keyword)
  expr:
    self: c1 (Identifier)
",
        ),
        TestCase::new(
            "\
SELECT CASE WHEN c1 = 1 THEN 'one' ELSE f() END
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: CASE (CaseExpr)
  arms:
  - self: WHEN (CaseArm)
    expr:
      self: = (BinaryOperator)
      left:
        self: c1 (Identifier)
      right:
        self: 1 (NumericLiteral)
    result:
      self: 'one' (StringLiteral)
    then:
      self: THEN (Keyword)
  - self: ELSE (CaseArm)
    result:
      self: ( (CallingFunction)
      func:
        self: f (Identifier)
      rparen:
        self: ) (Symbol)
  end:
    self: END (Keyword)
",
        ),
        // ----- function -----
        TestCase::new(
            "\
SELECT f(c1, c2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: c1 (Identifier)
    comma:
      self: , (Symbol)
  - self: c2 (Identifier)
  func:
    self: f (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // CAST
        TestCase::new(
            "\
SELECT CAST('1' AS INT64),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: AS (CastArgument)
    cast_from:
      self: '1' (StringLiteral)
    cast_to:
      self: INT64 (Type)
  comma:
    self: , (Symbol)
  func:
    self: CAST (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // EXTRACT
        TestCase::new(
            "\
SELECT EXTRACT(DAY FROM ts)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    extract_datepart:
      self: DAY (Keyword)
    extract_from:
      self: ts (Identifier)
  func:
    self: EXTRACT (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT EXTRACT(WEEK(SUNDAY) FROM ts)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    extract_datepart:
      self: ( (CallingDatePartFunction)
      args:
      - self: SUNDAY (Identifier)
      func:
        self: WEEK (Identifier)
      rparen:
        self: ) (Symbol)
    extract_from:
      self: ts (Identifier)
  func:
    self: EXTRACT (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT EXTRACT(DAY FROM ts AT TIME ZONE 'UTC')
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    at_time_zone:
    - self: AT (Keyword)
    - self: TIME (Keyword)
    - self: ZONE (Keyword)
    extract_datepart:
      self: DAY (Keyword)
    extract_from:
      self: ts (Identifier)
    time_zone:
      self: 'UTC' (StringLiteral)
  func:
    self: EXTRACT (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // ARRAY_AGG
        TestCase::new(
            "\
SELECT ARRAY_AGG(DISTINCT x IGNORE NULLS ORDER BY z DESC LIMIT 100)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Identifier)
  distinct:
    self: DISTINCT (Keyword)
  func:
    self: ARRAY_AGG (Identifier)
  ignore_nulls:
  - self: IGNORE (Keyword)
  - self: NULLS (Keyword)
  limit:
    self: LIMIT (KeywordWithExpr)
    expr:
      self: 100 (NumericLiteral)
  orderby:
    self: ORDER (XXXByExprs)
    by:
      self: BY (Keyword)
    exprs:
    - self: z (Identifier)
      order:
        self: DESC (Keyword)
  rparen:
    self: ) (Symbol)
",
        ),
        // ARRAY
        TestCase::new(
            "\
SELECT ARRAY(SELECT 1 UNION ALL SELECT 2),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: UNION (SetOperator)
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
  comma:
    self: , (Symbol)
  func:
    self: ARRAY (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // ST_GEOGFROMTEXT
        TestCase::new(
            "\
SELECT ST_GEOGFROMTEXT(p, oriented => TRUE),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: p (Identifier)
    comma:
      self: , (Symbol)
  - self: => (BinaryOperator)
    left:
      self: oriented (Identifier)
    right:
      self: TRUE (BooleanLiteral)
  comma:
    self: , (Symbol)
  func:
    self: ST_GEOGFROMTEXT (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // INTERVAL x date_part
        TestCase::new(
            "\
SELECT DATE_ADD(dt, INTERVAL 1 + 1 DAY),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: dt (Identifier)
    comma:
      self: , (Symbol)
  - self: INTERVAL (IntervalLiteral)
    date_part:
      self: DAY (Keyword)
    right:
      self: + (BinaryOperator)
      left:
        self: 1 (NumericLiteral)
      right:
        self: 1 (NumericLiteral)
  comma:
    self: , (Symbol)
  func:
    self: DATE_ADD (Identifier)
  rparen:
    self: ) (Symbol)
",
        ),
        // ----- window function -----
        TestCase::new(
            "\
SELECT SUM(x) OVER ()
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Identifier)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        // PARTITION BY, ORDER BY
        TestCase::new(
            "\
SELECT SUM(x) OVER (PARTITION BY a)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Identifier)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      partitionby:
        self: PARTITION (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: a (Identifier)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT SUM(x) OVER (ORDER BY a DESC)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Identifier)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      orderby:
        self: ORDER (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: a (Identifier)
          order:
            self: DESC (Keyword)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT SUM(x) OVER (PARTITION BY a ORDER BY b ASC, c)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Identifier)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      orderby:
        self: ORDER (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: b (Identifier)
          comma:
            self: , (Symbol)
          order:
            self: ASC (Keyword)
        - self: c (Identifier)
      partitionby:
        self: PARTITION (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: a (Identifier)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        // window frame clause
        TestCase::new(
            "\
SELECT SUM() OVER (ROWS 1 + 1 PRECEDING)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      frame:
        self: ROWS (WindowFrameClause)
        start:
        - self: + (BinaryOperator)
          left:
            self: 1 (NumericLiteral)
          right:
            self: 1 (NumericLiteral)
        - self: PRECEDING (Keyword)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT SUM() OVER (PARTITION BY a ORDER BY b, c ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      frame:
        self: ROWS (WindowFrameClause)
        and:
          self: AND (Keyword)
        between:
          self: BETWEEN (Keyword)
        end:
        - self: UNBOUNDED (Keyword)
        - self: FOLLOWING (Keyword)
        start:
        - self: UNBOUNDED (Keyword)
        - self: PRECEDING (Keyword)
      orderby:
        self: ORDER (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: b (Identifier)
          comma:
            self: , (Symbol)
        - self: c (Identifier)
      partitionby:
        self: PARTITION (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: a (Identifier)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        // named window specification
        TestCase::new(
            "\
SELECT
  SUM() OVER named_window,
  SUM() OVER (named_window),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  comma:
    self: , (Symbol)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: named_window (Identifier)
  rparen:
    self: ) (Symbol)
- self: ( (CallingFunction)
  comma:
    self: , (Symbol)
  func:
    self: SUM (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      name:
        self: named_window (Identifier)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT last_value(col3) OVER (c ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: col3 (Identifier)
  func:
    self: last_value (Identifier)
  over:
    self: OVER (OverClause)
    window:
      self: ( (WindowSpecification)
      frame:
        self: ROWS (WindowFrameClause)
        and:
          self: AND (Keyword)
        between:
          self: BETWEEN (Keyword)
        end:
        - self: 2 (NumericLiteral)
        - self: FOLLOWING (Keyword)
        start:
        - self: 2 (NumericLiteral)
        - self: PRECEDING (Keyword)
      name:
        self: c (Identifier)
      rparen:
        self: ) (Symbol)
  rparen:
    self: ) (Symbol)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}
