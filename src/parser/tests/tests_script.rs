use super::*;

#[test]
fn test_parse_code_script() {
    let test_cases = vec![
        // ----- DECLARE statement -----
        TestCase::new(
            "\
DECLARE x INT64;
",
            "\
self: DECLARE (DeclareStatement)
idents:
- self: x (Identifier)
semicolon:
  self: ; (Symbol)
variable_type:
  self: INT64 (Type)
",
        ),
        TestCase::new(
            "\
DECLARE x,y DEFAULT 1;
",
            "\
self: DECLARE (DeclareStatement)
default:
  self: DEFAULT (KeywordWithExpr)
  expr:
    self: 1 (NumericLiteral)
idents:
- self: x (Identifier)
  comma:
    self: , (Symbol)
- self: y (Identifier)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- SET statement -----
        TestCase::new(
            "\
SET x = 5
",
            "\
self: SET (SetStatement)
expr:
  self: = (BinaryOperator)
  left:
    self: x (Identifier)
  right:
    self: 5 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
SET (x,y) = (1,2)
",
            "\
self: SET (SetStatement)
expr:
  self: = (BinaryOperator)
  left:
    self: ( (StructLiteral)
    exprs:
    - self: x (Identifier)
      comma:
        self: , (Symbol)
    - self: y (Identifier)
    rparen:
      self: ) (Symbol)
  right:
    self: ( (StructLiteral)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SET (x, y) = (SELECT AS STRUCT 1,2)
",
            "\
self: SET (SetStatement)
expr:
  self: = (BinaryOperator)
  left:
    self: ( (StructLiteral)
    exprs:
    - self: x (Identifier)
      comma:
        self: , (Symbol)
    - self: y (Identifier)
    rparen:
      self: ) (Symbol)
  right:
    self: ( (GroupedStatement)
    rparen:
      self: ) (Symbol)
    stmt:
      self: SELECT (SelectStatement)
      as_struct_or_value:
      - self: AS (Keyword)
      - self: STRUCT (Keyword)
      exprs:
      - self: 1 (NumericLiteral)
        comma:
          self: , (Symbol)
      - self: 2 (NumericLiteral)
",
        ),
        // ----- EXECUTE statement -----
        TestCase::new(
            "\
EXECUTE IMMEDIATE 'SELECT 1'
",
            "\
self: EXECUTE (ExecuteStatement)
immediate:
  self: IMMEDIATE (Keyword)
sql_expr:
  self: 'SELECT 1' (StringLiteral)
",
        ),
        TestCase::new(
            "\
EXECUTE IMMEDIATE 'SELECT ?' USING 1;
",
            "\
self: EXECUTE (ExecuteStatement)
immediate:
  self: IMMEDIATE (Keyword)
semicolon:
  self: ; (Symbol)
sql_expr:
  self: 'SELECT ?' (StringLiteral)
using:
  self: USING (KeywordWithExprs)
  exprs:
  - self: 1 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
EXECUTE IMMEDIATE 'SELECT ?,?' INTO x, y USING 1, 2;
",
            "\
self: EXECUTE (ExecuteStatement)
immediate:
  self: IMMEDIATE (Keyword)
into:
  self: INTO (KeywordWithExprs)
  exprs:
  - self: x (Identifier)
    comma:
      self: , (Symbol)
  - self: y (Identifier)
semicolon:
  self: ; (Symbol)
sql_expr:
  self: 'SELECT ?,?' (StringLiteral)
using:
  self: USING (KeywordWithExprs)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
",
        ),
        TestCase::new(
            "\
EXECUTE IMMEDIATE 'SELECT @x' INTO x USING 1 AS x;
",
            "\
self: EXECUTE (ExecuteStatement)
immediate:
  self: IMMEDIATE (Keyword)
into:
  self: INTO (KeywordWithExprs)
  exprs:
  - self: x (Identifier)
semicolon:
  self: ; (Symbol)
sql_expr:
  self: 'SELECT @x' (StringLiteral)
using:
  self: USING (KeywordWithExprs)
  exprs:
  - self: 1 (NumericLiteral)
    alias:
      self: x (Identifier)
    as:
      self: AS (Keyword)
",
        ),
        // ----- BEGIN statement -----
        TestCase::new(
            "\
BEGIN
  SELECT 1;
  SELECT 2;
END;
",
            "\
self: BEGIN (BeginStatement)
end:
  self: END (Keyword)
semicolon:
  self: ; (Symbol)
stmts:
- self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
- self: SELECT (SelectStatement)
  exprs:
  - self: 2 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
BEGIN
  SELECT 1;
EXCEPTION WHEN ERROR THEN
  SELECT 2;
END;
",
            "\
self: BEGIN (BeginStatement)
end:
  self: END (Keyword)
exception_when_error:
- self: EXCEPTION (Keyword)
- self: WHEN (Keyword)
- self: ERROR (Keyword)
semicolon:
  self: ; (Symbol)
stmts:
- self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 2 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
BEGIN EXCEPTiON WHEN ERROR THEN END;
",
            "\
self: BEGIN (BeginStatement)
end:
  self: END (Keyword)
exception_when_error:
- self: EXCEPTiON (Keyword)
- self: WHEN (Keyword)
- self: ERROR (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
",
        ),
        // ----- IF statement -----
        TestCase::new(
            "\
IF TRUE THEN END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
",
        ),
        TestCase::new(
            "\
IF TRUE THEN
  SELECT 1;
  SELECT 2;
END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
  - self: SELECT (SelectStatement)
    exprs:
    - self: 2 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
IF TRUE THEN
  SELECT 1;
ELSEIF TRUE THEN
END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
elseifs:
- self: ELSEIF (ElseIfClause)
  condition:
    self: TRUE (BooleanLiteral)
  then:
    self: THEN (KeywordWithStatements)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
IF TRUE THEN
ELSEIF TRUE THEN
  SELECT 1;
ELSEIF TRUE THEN
  SELECT 2;
  SELECT 3;
ELSE
END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
else:
  self: ELSE (KeywordWithStatements)
elseifs:
- self: ELSEIF (ElseIfClause)
  condition:
    self: TRUE (BooleanLiteral)
  then:
    self: THEN (KeywordWithStatements)
    stmts:
    - self: SELECT (SelectStatement)
      exprs:
      - self: 1 (NumericLiteral)
      semicolon:
        self: ; (Symbol)
- self: ELSEIF (ElseIfClause)
  condition:
    self: TRUE (BooleanLiteral)
  then:
    self: THEN (KeywordWithStatements)
    stmts:
    - self: SELECT (SelectStatement)
      exprs:
      - self: 2 (NumericLiteral)
      semicolon:
        self: ; (Symbol)
    - self: SELECT (SelectStatement)
      exprs:
      - self: 3 (NumericLiteral)
      semicolon:
        self: ; (Symbol)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
",
        ),
        TestCase::new(
            "\
IF TRUE THEN
ELSE SELECT 1;
END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
else:
  self: ELSE (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
",
        ),
        TestCase::new(
            "\
IF TRUE THEN
ELSE
  SELECT 1;
  SELECT 2;
END IF;
",
            "\
self: IF (IfStatement)
condition:
  self: TRUE (BooleanLiteral)
else:
  self: ELSE (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
  - self: SELECT (SelectStatement)
    exprs:
    - self: 2 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
end_if:
- self: END (Keyword)
- self: IF (Keyword)
semicolon:
  self: ; (Symbol)
then:
  self: THEN (KeywordWithStatements)
",
        ),
        // ----- LOOP statement -----
        TestCase::new(
            "\
LOOP
  SELECT 1;
END LOOP;
",
            "\
self: LOOP (LoopStatement)
end_loop:
- self: END (Keyword)
- self: LOOP (Keyword)
semicolon:
  self: ; (Symbol)
stmts:
- self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
LOOP SELECT 1; BREAK; END LOOP;
",
            "\
self: LOOP (LoopStatement)
end_loop:
- self: END (Keyword)
- self: LOOP (Keyword)
semicolon:
  self: ; (Symbol)
stmts:
- self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
- self: BREAK (SingleTokenStatement)
  semicolon:
    self: ; (Symbol)
",
        ),
        // ----- WHILE statement -----
        TestCase::new(
            "\
WHILE TRUE DO
  SELECT 1;
END WHILE;
",
            "\
self: WHILE (WhileStatement)
condition:
  self: TRUE (BooleanLiteral)
do:
  self: DO (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
end_while:
- self: END (Keyword)
- self: WHILE (Keyword)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- WHILE statement -----
        TestCase::new(
            "\
WHILE TRUE DO
  ITERATE;
  LEAVE;
  CONTINUE;
END WHILE;
",
            "\
self: WHILE (WhileStatement)
condition:
  self: TRUE (BooleanLiteral)
do:
  self: DO (KeywordWithStatements)
  stmts:
  - self: ITERATE (SingleTokenStatement)
    semicolon:
      self: ; (Symbol)
  - self: LEAVE (SingleTokenStatement)
    semicolon:
      self: ; (Symbol)
  - self: CONTINUE (SingleTokenStatement)
    semicolon:
      self: ; (Symbol)
end_while:
- self: END (Keyword)
- self: WHILE (Keyword)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- WHILE statement -----
        TestCase::new(
            "\
RAISE;
",
            "\
self: RAISE (RaiseStatement)
semicolon:
  self: ; (Symbol)
",
        ),
        TestCase::new(
            "\
RAISE USING MESSAGE = 'error';
",
            "\
self: RAISE (RaiseStatement)
semicolon:
  self: ; (Symbol)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: = (BinaryOperator)
    left:
      self: MESSAGE (Identifier)
    right:
      self: 'error' (StringLiteral)
",
        ),
        // ----- CALL statement -----
        TestCase::new(
            "\
CALL mydataset.myprocedure(1);
",
            "\
self: CALL (CallStatement)
procedure:
  self: ( (CallingFunction)
  args:
  - self: 1 (NumericLiteral)
  func:
    self: . (DotOperator)
    left:
      self: mydataset (Identifier)
    right:
      self: myprocedure (Identifier)
  rparen:
    self: ) (Symbol)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- system variables (@@xxx) -----
        TestCase::new(
            "\
BEGIN
  BEGIN
    SELECT 1;
  EXCEPTION WHEN ERROR THEN
    RAISE USING MESSAGE = 'error';
  END;
EXCEPTION WHEN ERROR THEN
  SELECT @@error.message;
END;
",
            "\
self: BEGIN (BeginStatement)
end:
  self: END (Keyword)
exception_when_error:
- self: EXCEPTION (Keyword)
- self: WHEN (Keyword)
- self: ERROR (Keyword)
semicolon:
  self: ; (Symbol)
stmts:
- self: BEGIN (BeginStatement)
  end:
    self: END (Keyword)
  exception_when_error:
  - self: EXCEPTION (Keyword)
  - self: WHEN (Keyword)
  - self: ERROR (Keyword)
  semicolon:
    self: ; (Symbol)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
  then:
    self: THEN (KeywordWithStatements)
    stmts:
    - self: RAISE (RaiseStatement)
      semicolon:
        self: ; (Symbol)
      using:
        self: USING (KeywordWithExpr)
        expr:
          self: = (BinaryOperator)
          left:
            self: MESSAGE (Identifier)
          right:
            self: 'error' (StringLiteral)
then:
  self: THEN (KeywordWithStatements)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: . (DotOperator)
      left:
        self: @@error (Parameter)
      right:
        self: message (Identifier)
    semicolon:
      self: ; (Symbol)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}
