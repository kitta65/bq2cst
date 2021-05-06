use super::*;

#[test]
fn test_parse_code_select() {
    let test_cases = vec![
        TestCase::new(
            "\
SELECT 1;
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 1 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
(SELECT 1);
",
            "\
self: ( (GroupedStatement)
rparen:
  self: ) (Symbol)
semicolon:
  self: ; (Symbol)
stmt:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
",
        ),
        // ----- set operator -----
        TestCase::new(
            "\
SELECT 1 UNION ALL SELECT 2;
",
            "\
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
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT 1 INTERSECT DISTINCT (SELECT 2);
",
            "\
self: INTERSECT (SetOperator)
distinct_or_all:
  self: DISTINCT (Keyword)
left:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
right:
  self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
(SELECT 1) EXCEPT DISTINCT SELECT 2;
",
            "\
self: EXCEPT (SetOperator)
distinct_or_all:
  self: DISTINCT (Keyword)
left:
  self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
right:
  self: SELECT (SelectStatement)
  exprs:
  - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT 1 UNION ALL SELECT 2 UNION ALL SELECT 3;
",
            "\
self: UNION (SetOperator)
distinct_or_all:
  self: ALL (Keyword)
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
  self: SELECT (SelectStatement)
  exprs:
  - self: 3 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT 1 UNION ALL (SELECT 2 UNION ALL SELECT 3);
",
            "\
self: UNION (SetOperator)
distinct_or_all:
  self: ALL (Keyword)
left:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
right:
  self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: UNION (SetOperator)
    distinct_or_all:
      self: ALL (Keyword)
    left:
      self: SELECT (SelectStatement)
      exprs:
      - self: 2 (NumericLiteral)
    right:
      self: SELECT (SelectStatement)
      exprs:
      - self: 3 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- WITH clause -----
        TestCase::new(
            "\
WITH a AS (SELECT 1) SELECT 2;
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
with:
  self: WITH (WithClause)
  queries:
  - self: a (WithQuery)
    as:
      self: AS (Keyword)
    stmt:
      self: ( (GroupedStatement)
      rparen:
        self: ) (Symbol)
      stmt:
        self: SELECT (SelectStatement)
        exprs:
        - self: 1 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
WITH
  a AS (SELECT 1),
  b AS (SELECT 2 FROM t WHERE TRUE)
SELECT 3
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 3 (NumericLiteral)
with:
  self: WITH (WithClause)
  queries:
  - self: a (WithQuery)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
    stmt:
      self: ( (GroupedStatement)
      rparen:
        self: ) (Symbol)
      stmt:
        self: SELECT (SelectStatement)
        exprs:
        - self: 1 (NumericLiteral)
  - self: b (WithQuery)
    as:
      self: AS (Keyword)
    stmt:
      self: ( (GroupedStatement)
      rparen:
        self: ) (Symbol)
      stmt:
        self: SELECT (SelectStatement)
        exprs:
        - self: 2 (NumericLiteral)
        from:
          self: FROM (KeywordWithExpr)
          expr:
            self: t (Identifier)
        where:
          self: WHERE (KeywordWithExpr)
          expr:
            self: TRUE (BooleanLiteral)
",
        ),
        // ----- SELECT clause -----
        // DISTINCT
        TestCase::new(
            "\
SELECT DISTINCT 1;
",
            "\
self: SELECT (SelectStatement)
distinct_or_all:
  self: DISTINCT (Keyword)
exprs:
- self: 1 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        // ALL
        TestCase::new(
            "\
SELECT ALL 1;
",
            "\
self: SELECT (SelectStatement)
distinct_or_all:
  self: ALL (Keyword)
exprs:
- self: 1 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        // alias
        TestCase::new(
            "\
SELECT 1 AS one, 2 two
",
            "\
self: SELECT (SelectStatement)
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
",
        ),
        // * EXCEPT
        TestCase::new(
            "\
SELECT * EXCEPT (col1)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
  except:
    self: EXCEPT (KeywordWithGroupedExprs)
    group:
      self: ( (GroupedExprs)
      exprs:
      - self: col1 (Identifier)
      rparen:
        self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT t.* EXCEPT(col1, col2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: . (BinaryOperator)
  left:
    self: t (Identifier)
  right:
    self: * (Symbol)
    except:
      self: EXCEPT (KeywordWithGroupedExprs)
      group:
        self: ( (GroupedExprs)
        exprs:
        - self: col1 (Identifier)
          comma:
            self: , (Symbol)
        - self: col2 (Identifier)
        rparen:
          self: ) (Symbol)
",
        ),
        // * REPLACE
        TestCase::new(
            "\
SELECT * REPLACE (col1 * 2 AS _col1)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
  replace:
    self: REPLACE (KeywordWithGroupedExprs)
    group:
      self: ( (GroupedExprs)
      exprs:
      - self: * (BinaryOperator)
        alias:
          self: _col1 (Identifier)
        as:
          self: AS (Keyword)
        left:
          self: col1 (Identifier)
        right:
          self: 2 (NumericLiteral)
      rparen:
        self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT t.* REPLACE (col2 * 2 AS _col2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: . (BinaryOperator)
  left:
    self: t (Identifier)
  right:
    self: * (Symbol)
    replace:
      self: REPLACE (KeywordWithGroupedExprs)
      group:
        self: ( (GroupedExprs)
        exprs:
        - self: * (BinaryOperator)
          alias:
            self: _col2 (Identifier)
          as:
            self: AS (Keyword)
          left:
            self: col2 (Identifier)
          right:
            self: 2 (NumericLiteral)
        rparen:
          self: ) (Symbol)
",
        ),
        // AS STRUCT, VALUE
        TestCase::new(
            "\
SELECT (SELECT AS STRUCT 1 a, 2 b) ab
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (GroupedStatement)
  alias:
    self: ab (Identifier)
  rparen:
    self: ) (Symbol)
  stmt:
    self: SELECT (SelectStatement)
    as_struct_or_value:
    - self: AS (Keyword)
    - self: STRUCT (Keyword)
    exprs:
    - self: 1 (NumericLiteral)
      alias:
        self: a (Identifier)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
      alias:
        self: b (Identifier)
",
        ),
        TestCase::new(
            "\
SELECT AS VALUE STRUCT(1 AS a, 2 AS b) xyz
",
            "\
self: SELECT (SelectStatement)
as_struct_or_value:
- self: AS (Keyword)
- self: VALUE (Keyword)
exprs:
- self: ( (StructLiteral)
  alias:
    self: xyz (Identifier)
  exprs:
  - self: 1 (NumericLiteral)
    alias:
      self: a (Identifier)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
    alias:
      self: b (Identifier)
    as:
      self: AS (Keyword)
  rparen:
    self: ) (Symbol)
  type:
    self: STRUCT (Type)
",
        ),
        // sub query
        TestCase::new(
            "\
SELECT (SELECT 1)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
SELECT (SELECT 1 EXCEPT DISTINCT SELECT 2);
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (GroupedStatement)
  rparen:
    self: ) (Symbol)
  stmt:
    self: EXCEPT (SetOperator)
    distinct_or_all:
      self: DISTINCT (Keyword)
    left:
      self: SELECT (SelectStatement)
      exprs:
      - self: 1 (NumericLiteral)
    right:
      self: SELECT (SelectStatement)
      exprs:
      - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- FROM clause -----
        // alias
        TestCase::new(
            "\
SELECT 1
FROM t1 AS t
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 1 (NumericLiteral)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t1 (Identifier)
    alias:
      self: t (Identifier)
    as:
      self: AS (Keyword)
",
        ),
        // sub query
        TestCase::new(
            "\
SELECT * FROM (SELECT 1,2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: ( (GroupedStatement)
    rparen:
      self: ) (Symbol)
    stmt:
      self: SELECT (SelectStatement)
      exprs:
      - self: 1 (NumericLiteral)
        comma:
          self: , (Symbol)
      - self: 2 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
SELECT SUB.* FROM (SELECT 1,2) AS SUB;
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: . (BinaryOperator)
  left:
    self: SUB (Identifier)
  right:
    self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: ( (GroupedStatement)
    alias:
      self: SUB (Identifier)
    as:
      self: AS (Keyword)
    rparen:
      self: ) (Symbol)
    stmt:
      self: SELECT (SelectStatement)
      exprs:
      - self: 1 (NumericLiteral)
        comma:
          self: , (Symbol)
      - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT *
FROM main m
WHERE NOT EXISTS(SELECT 1 FROM sub s WHERE s.x = m.x);
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: main (Identifier)
    alias:
      self: m (Identifier)
semicolon:
  self: ; (Symbol)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: NOT (UnaryOperator)
    right:
      self: ( (CallingFunction)
      args:
      - self: SELECT (SelectStatement)
        exprs:
        - self: 1 (NumericLiteral)
        from:
          self: FROM (KeywordWithExpr)
          expr:
            self: sub (Identifier)
            alias:
              self: s (Identifier)
        where:
          self: WHERE (KeywordWithExpr)
          expr:
            self: = (BinaryOperator)
            left:
              self: . (BinaryOperator)
              left:
                self: s (Identifier)
              right:
                self: x (Identifier)
            right:
              self: . (BinaryOperator)
              left:
                self: m (Identifier)
              right:
                self: x (Identifier)
      func:
        self: EXISTS (Identifier)
      rparen:
        self: ) (Symbol)
",
        ),
        // FOR SYSTEM_TIME AS OF
        TestCase::new(
            "\
SELECT c1 FROM t FOR SYSTEM_TIME AS OF CURRENT_TIMESTAMP()
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: c1 (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
    for_system_time_as_of:
      self: FOR (ForSystemTimeAsOfClause)
      expr:
        self: ( (CallingFunction)
        func:
          self: CURRENT_TIMESTAMP (Identifier)
        rparen:
          self: ) (Symbol)
      system_time_as_of:
      - self: SYSTEM_TIME (Keyword)
      - self: AS (Keyword)
      - self: OF (Keyword)
",
        ),
        // TABLESAMPLE
        TestCase::new(
            "\
SELECT *
FROM t TABLESAMPLE SYSTEM (20 PERCENT)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
    tablesample:
      self: TABLESAMPLE (TableSampleCaluse)
      group:
        self: ( (TableSampleRatio)
        expr:
          self: 20 (NumericLiteral)
        percent:
          self: PERCENT (Keyword)
        rparen:
          self: ) (Symbol)
      system:
        self: SYSTEM (Keyword)
",
        ),
        // UNNEST
        TestCase::new(
            "\
SELECT * FROM UNNEST([1,2])
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: ( (CallingFunction)
    args:
    - self: [ (ArrayLiteral)
      exprs:
      - self: 1 (NumericLiteral)
        comma:
          self: , (Symbol)
      - self: 2 (NumericLiteral)
      rparen:
        self: ] (Symbol)
    func:
      self: UNNEST (Identifier)
    rparen:
      self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT * FROM UNNEST([1]) WITH OFFSET
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: ( (CallingFunction)
    args:
    - self: [ (ArrayLiteral)
      exprs:
      - self: 1 (NumericLiteral)
      rparen:
        self: ] (Symbol)
    func:
      self: UNNEST (Identifier)
    rparen:
      self: ) (Symbol)
    with_offset:
    - self: WITH (Keyword)
    - self: OFFSET (Keyword)
",
        ),
        TestCase::new(
            "\
SELECT * FROM UNNEST([1]) a WITH OFFSET AS b
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: ( (CallingFunction)
    alias:
      self: a (Identifier)
    args:
    - self: [ (ArrayLiteral)
      exprs:
      - self: 1 (NumericLiteral)
      rparen:
        self: ] (Symbol)
    func:
      self: UNNEST (Identifier)
    offset_alias:
      self: b (Identifier)
    offset_as:
      self: AS (Keyword)
    rparen:
      self: ) (Symbol)
    with_offset:
    - self: WITH (Keyword)
    - self: OFFSET (Keyword)
",
        ),
        // JOIN
        TestCase::new(
            "\
SELECT * FROM (SELECT 1 FROM t1) INNER JOIN t2;
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: JOIN (JoinOperator)
    join_type:
      self: INNER (Keyword)
    left:
      self: ( (GroupedStatement)
      rparen:
        self: ) (Symbol)
      stmt:
        self: SELECT (SelectStatement)
        exprs:
        - self: 1 (NumericLiteral)
        from:
          self: FROM (KeywordWithExpr)
          expr:
            self: t1 (Identifier)
    right:
      self: t2 (Identifier)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- WHERE clause -----
        TestCase::new(
            "\
SELECT x FROM t WHERE true
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: x (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: true (BooleanLiteral)
",
        ),
        // ----- GROUP BY clause -----
        TestCase::new(
            "\
SELECT x, y FROM t GROUP BY 1, 2
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: x (Identifier)
  comma:
    self: , (Symbol)
- self: y (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
groupby:
  self: GROUP (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
",
        ),
        // ----- HAVING clause -----
        TestCase::new(
            "\
SELECT x, COUNT(*) cnt FROM t GROUP BY 1 HAVING cnt < 10
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: x (Identifier)
  comma:
    self: , (Symbol)
- self: ( (CallingFunction)
  alias:
    self: cnt (Identifier)
  args:
  - self: * (Symbol)
  func:
    self: COUNT (Identifier)
  rparen:
    self: ) (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
groupby:
  self: GROUP (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: 1 (NumericLiteral)
having:
  self: HAVING (KeywordWithExpr)
  expr:
    self: < (BinaryOperator)
    left:
      self: cnt (Identifier)
    right:
      self: 10 (NumericLiteral)
",
        ),
        // ----- WINDOW clause -----
        TestCase::new(
            "\
SELECT *
FROM t
WINDOW
  a AS (PARTITION BY col1),
  b AS (a ORDER BY col2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
window:
  self: WINDOW (WindowClause)
  window_exprs:
  - self: a (WindowExpr)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
    window:
      self: ( (WindowSpecification)
      partitionby:
        self: PARTITION (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: col1 (Identifier)
      rparen:
        self: ) (Symbol)
  - self: b (WindowExpr)
    as:
      self: AS (Keyword)
    window:
      self: ( (WindowSpecification)
      name:
        self: a (Identifier)
      orderby:
        self: ORDER (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: col2 (Identifier)
      rparen:
        self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT *
FROM t
WINDOW
  a AS (PARTITION BY col1),
  b AS a
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: * (Symbol)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
window:
  self: WINDOW (WindowClause)
  window_exprs:
  - self: a (WindowExpr)
    as:
      self: AS (Keyword)
    comma:
      self: , (Symbol)
    window:
      self: ( (WindowSpecification)
      partitionby:
        self: PARTITION (XXXByExprs)
        by:
          self: BY (Keyword)
        exprs:
        - self: col1 (Identifier)
      rparen:
        self: ) (Symbol)
  - self: b (WindowExpr)
    as:
      self: AS (Keyword)
    window:
      self: a (Identifier)
",
        ),
        // ----- ORDER BY clause -----
        TestCase::new(
            "\
SELECT c1 FROM t ORDER BY c1 ASC, c2
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: c1 (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
orderby:
  self: ORDER (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: c1 (Identifier)
    comma:
      self: , (Symbol)
    order:
      self: ASC (Keyword)
  - self: c2 (Identifier)
",
        ),
        // ----- LIMIT clause -----
        TestCase::new(
            "\
SELECT c1 FROM t LIMIT 100
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: c1 (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
limit:
  self: LIMIT (LimitClause)
  expr:
    self: 100 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
SELECT c1 FROM t LIMIT 100 OFFSET 10
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: c1 (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t (Identifier)
limit:
  self: LIMIT (LimitClause)
  expr:
    self: 100 (NumericLiteral)
  offset:
    self: OFFSET (KeywordWithExpr)
    expr:
      self: 10 (NumericLiteral)
",
        ),
    ];
    for t in test_cases {
        t.test();
    }
}

