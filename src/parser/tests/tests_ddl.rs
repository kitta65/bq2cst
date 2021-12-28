use super::*;

#[test]
fn test_parse_code_ddl() {
    let test_cases: Vec<Box<dyn TestCase>> = vec![
        // ----- CREATE SCHEMA statement -----
        Box::new(SuccessTestCase::new(
            "\
CREATE SCHEMA dataset_name;
",
            "\
self: CREATE (CreateSchemaStatement)
ident:
  self: dataset_name (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: SCHEMA (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE SCHEMA IF NOT EXISTS project_name.dataset_name OPTIONS();
",
            "\
self: CREATE (CreateSchemaStatement)
ident:
  self: . (DotOperator)
  left:
    self: project_name (Identifier)
  right:
    self: dataset_name (Identifier)
if_not_exists:
- self: IF (Keyword)
- self: NOT (Keyword)
- self: EXISTS (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: SCHEMA (Keyword)
",
            0,
        )),
        Box::new(ErrorTestCase::new(
            "\
CREATE SCHEEMAA IF NOT EXISTS dataset_name;
",
            1,
            1,
        )),
        // ----- CREATE TABLE statement -----
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE example (x int64);
",
            "\
self: CREATE (CreateTableStatement)
column_schema_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: int64 (Type)
  rparen:
    self: ) (Symbol)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE this-is-project-name.dataset.example (x int64);
",
            "\
self: CREATE (CreateTableStatement)
column_schema_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: int64 (Type)
  rparen:
    self: ) (Symbol)
ident:
  self: . (DotOperator)
  left:
    self: . (DotOperator)
    left:
      self: - (MultiTokenIdentifier)
      left:
        self: - (MultiTokenIdentifier)
        left:
          self: - (MultiTokenIdentifier)
          left:
            self: this (Identifier)
          right:
            self: is (Identifier)
        right:
          self: project (Identifier)
      right:
        self: name (Identifier)
    right:
      self: dataset (Identifier)
  right:
    self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE TEMP TABLE example (x INT64, y STRING(10));
",
            "\
self: CREATE (CreateTableStatement)
column_schema_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    comma:
      self: , (Symbol)
    type:
      self: INT64 (Type)
  - self: y (TypeDeclaration)
    type:
      self: STRING (Type)
      parameter:
        self: ( (GroupedExprs)
        exprs:
        - self: 10 (NumericLiteral)
        rparen:
          self: ) (Symbol)
  rparen:
    self: ) (Symbol)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
temp:
  self: TEMP (Keyword)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE OR REPLACE TABLE dataset.example(x INT64 OPTIONS(description = 'dummy'))
PARTITION BY _PARTITIONDATE
OPTIONS(partition_expiration_days = 1);
",
            "\
self: CREATE (CreateTableStatement)
column_schema_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
      options:
        self: OPTIONS (KeywordWithGroupedXXX)
        group:
          self: ( (GroupedExprs)
          exprs:
          - self: = (BinaryOperator)
            left:
              self: description (Identifier)
            right:
              self: 'dummy' (StringLiteral)
          rparen:
            self: ) (Symbol)
  rparen:
    self: ) (Symbol)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: example (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: partition_expiration_days (Identifier)
      right:
        self: 1 (NumericLiteral)
    rparen:
      self: ) (Symbol)
or_replace:
- self: OR (Keyword)
- self: REPLACE (Keyword)
partitionby:
  self: PARTITION (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: _PARTITIONDATE (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE IF NOT EXISTS example (x INT64 NOT NULL)
CLUSTER BY x
AS SELECT 1 UNION ALL SELECT 2;
",
            "\
self: CREATE (CreateTableStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
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
clusterby:
  self: CLUSTER (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: x (Identifier)
column_schema_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
      not_null:
      - self: NOT (Keyword)
      - self: NULL (Keyword)
  rparen:
    self: ) (Symbol)
ident:
  self: example (Identifier)
if_not_exists:
- self: IF (Keyword)
- self: NOT (Keyword)
- self: EXISTS (Keyword)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // LIKE
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE new_table LIKE source_table
",
            "\
self: CREATE (CreateTableStatement)
ident:
  self: new_table (Identifier)
like_or_copy:
  self: LIKE (Keyword)
source_table:
  self: source_table (Identifier)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // COPY
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE new_table COPY source_table
",
            "\
self: CREATE (CreateTableStatement)
ident:
  self: new_table (Identifier)
like_or_copy:
  self: COPY (Keyword)
source_table:
  self: source_table (Identifier)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // CLONE
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE from_snap CLONE snap
",
            "\
self: CREATE (CreateTableStatement)
clone:
  self: CLONE (KeywordWithExpr)
  expr:
    self: snap (Identifier)
ident:
  self: from_snap (Identifier)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // SNAPSHOT
        Box::new(SuccessTestCase::new(
            "\
CREATE SNAPSHOT TABLE snap
CLONE source_table
",
            "\
self: CREATE (CreateTableStatement)
clone:
  self: CLONE (KeywordWithExpr)
  expr:
    self: source_table (Identifier)
ident:
  self: snap (Identifier)
snapshot:
  self: SNAPSHOT (Keyword)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE SNAPSHOT TABLE snap
CLONE dataset.source_table FOR SYSTEM_TIME AS OF CURRENT_TIMESTAMP()
OPTIONS ()
",
            "\
self: CREATE (CreateTableStatement)
clone:
  self: CLONE (KeywordWithExpr)
  expr:
    self: . (DotOperator)
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
    left:
      self: dataset (Identifier)
    right:
      self: source_table (Identifier)
ident:
  self: snap (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
snapshot:
  self: SNAPSHOT (Keyword)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // EXTERNAL
        Box::new(SuccessTestCase::new(
            "\
CREATE EXTERNAL TABLE dataset.new_table
WITH PARTITION COLUMNS
OPTIONS (
  uris = ['dummy'],
  format = csv
);
",
            "\
self: CREATE (CreateTableStatement)
external:
  self: EXTERNAL (Keyword)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: new_table (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      comma:
        self: , (Symbol)
      left:
        self: uris (Identifier)
      right:
        self: [ (ArrayLiteral)
        exprs:
        - self: 'dummy' (StringLiteral)
        rparen:
          self: ] (Symbol)
    - self: = (BinaryOperator)
      left:
        self: format (Identifier)
      right:
        self: csv (Identifier)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
with_partition_columns:
  self: WITH (WithPartitionColumnsClause)
  partition_columns:
  - self: PARTITION (Keyword)
  - self: COLUMNS (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE EXTERNAL TABLE dataset.new_table
WITH PARTITION COLUMNS (
    col1 string
)
OPTIONS (
  uris = ['dummy'],
  format = csv
);
",
            "\
self: CREATE (CreateTableStatement)
external:
  self: EXTERNAL (Keyword)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: new_table (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      comma:
        self: , (Symbol)
      left:
        self: uris (Identifier)
      right:
        self: [ (ArrayLiteral)
        exprs:
        - self: 'dummy' (StringLiteral)
        rparen:
          self: ] (Symbol)
    - self: = (BinaryOperator)
      left:
        self: format (Identifier)
      right:
        self: csv (Identifier)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
with_partition_columns:
  self: WITH (WithPartitionColumnsClause)
  column_schema_group:
    self: ( (GroupedTypeDeclarations)
    declarations:
    - self: col1 (TypeDeclaration)
      type:
        self: string (Type)
    rparen:
      self: ) (Symbol)
  partition_columns:
  - self: PARTITION (Keyword)
  - self: COLUMNS (Keyword)
",
            0,
        )),
        // ----- CREATE VIEW statement -----
        Box::new(SuccessTestCase::new(
            "\
CREATE VIEW dataset.view_name
AS
  SELECT *
  FROM dataset.table_name
;
",
            "\
self: CREATE (CreateViewStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: * (Asterisk)
    from:
      self: FROM (KeywordWithExpr)
      expr:
        self: . (DotOperator)
        left:
          self: dataset (Identifier)
        right:
          self: table_name (Identifier)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: view_name (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE VIEW dataset_name.view_name(uno, dos)
AS SELECT 1 ONE, 2 TWO
",
            "\
self: CREATE (CreateViewStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
      alias:
        self: ONE (Identifier)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
      alias:
        self: TWO (Identifier)
column_name_list:
  self: ( (GroupedExprs)
  exprs:
  - self: uno (Identifier)
    comma:
      self: , (Symbol)
  - self: dos (Identifier)
  rparen:
    self: ) (Symbol)
ident:
  self: . (DotOperator)
  left:
    self: dataset_name (Identifier)
  right:
    self: view_name (Identifier)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        // MATERIALIZED
        Box::new(SuccessTestCase::new(
            "\
CREATE MATERIALIZED VIEW dataset.view_name
OPTIONS(dummy = 'dummy')
AS
    SELECT COUNT(*)
    FROM dataset.table_name
;
",
            "\
self: CREATE (CreateViewStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: ( (CallingFunction)
      args:
      - self: * (Asterisk)
      func:
        self: COUNT (Identifier)
      rparen:
        self: ) (Symbol)
    from:
      self: FROM (KeywordWithExpr)
      expr:
        self: . (DotOperator)
        left:
          self: dataset (Identifier)
        right:
          self: table_name (Identifier)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: view_name (Identifier)
materialized:
  self: MATERIALIZED (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        // ----- CREATE FUNCTION statement -----
        // sql function definition
        Box::new(SuccessTestCase::new(
            "\
CREATE OR REPLACE FUNCTION abc() AS (1);
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExpr)
    expr:
      self: 1 (NumericLiteral)
    rparen:
      self: ) (Symbol)
group:
  self: ( (GroupedTypeDeclarations)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
or_replace:
- self: OR (Keyword)
- self: REPLACE (Keyword)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE TEMP FUNCTION abc(x INT64) AS (x);
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExpr)
    expr:
      self: x (Identifier)
    rparen:
      self: ) (Symbol)
group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
semicolon:
  self: ; (Symbol)
temp:
  self: TEMP (Keyword)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE FUNCTION IF NOT EXISTS abc(x ARRAY<INT64>, y ANY TYPE)
RETURNS INT64
AS ('dummy');
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExpr)
    expr:
      self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
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
  - self: y (TypeDeclaration)
    type:
      self: ANY (Type)
      type:
        self: TYPE (Keyword)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
if_not_exists:
- self: IF (Keyword)
- self: NOT (Keyword)
- self: EXISTS (Keyword)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: INT64 (Type)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        // javascript function definition
        Box::new(SuccessTestCase::new(
            "\
CREATE FUNCTION abc() RETURNS INT64 LAGUAGE js
OPTIONS()
AS '''return 1''';
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithExpr)
  expr:
    self: '''return 1''' (StringLiteral)
group:
  self: ( (GroupedTypeDeclarations)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
language:
  self: LAGUAGE (LanguageSpecifier)
  language:
    self: js (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: INT64 (Type)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE FUNCTION abc() RETURNS INT64 DETERMINISTIC LANGUAGE js
OPTIONS(library = ['dummy'])
AS '''return 1''';
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithExpr)
  expr:
    self: '''return 1''' (StringLiteral)
determinism:
- self: DETERMINISTIC (Keyword)
group:
  self: ( (GroupedTypeDeclarations)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
language:
  self: LANGUAGE (LanguageSpecifier)
  language:
    self: js (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: library (Identifier)
      right:
        self: [ (ArrayLiteral)
        exprs:
        - self: 'dummy' (StringLiteral)
        rparen:
          self: ] (Symbol)
    rparen:
      self: ) (Symbol)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: INT64 (Type)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE FUNCTION abc() RETURNS INT64 NOT DETERMINISTIC LANGUAGE js
AS '''return 1''';
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithExpr)
  expr:
    self: '''return 1''' (StringLiteral)
determinism:
- self: NOT (Keyword)
- self: DETERMINISTIC (Keyword)
group:
  self: ( (GroupedTypeDeclarations)
  rparen:
    self: ) (Symbol)
ident:
  self: abc (Identifier)
language:
  self: LANGUAGE (LanguageSpecifier)
  language:
    self: js (Identifier)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: INT64 (Type)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        // TVF
        Box::new(SuccessTestCase::new(
            "\
CREATE TABLE FUNCTION one(x INT64)
RETURNS TABLE<one INT64>
AS SELECT 1 AS one
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
      alias:
        self: one (Identifier)
      as:
        self: AS (Keyword)
group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
  rparen:
    self: ) (Symbol)
ident:
  self: one (Identifier)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: TABLE (Type)
    type_declaration:
      self: < (GroupedTypeDeclarations)
      declarations:
      - self: one (TypeDeclaration)
        type:
          self: INT64 (Type)
      rparen:
        self: > (Symbol)
table:
  self: TABLE (Keyword)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        // ----- CREATE PROCEDURE statement -----
        Box::new(SuccessTestCase::new(
            "\
CREATE PROCEDURE dataset.procede() BEGIN SELECT 1; END;
",
            "\
self: CREATE (CreateProcedureStatement)
group:
  self: ( (GroupedTypeDeclarations)
  rparen:
    self: ) (Symbol)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: procede (Identifier)
semicolon:
  self: ; (Symbol)
stmt:
  self: BEGIN (BeginStatement)
  end:
    self: END (Keyword)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
what:
  self: PROCEDURE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE PROCEDURE dataset.procede(x INT64, INOUT y INT64)
OPTIONS(dummy = 'dummy')
BEGIN SELECT 1; END;
",
            "\
self: CREATE (CreateProcedureStatement)
group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: x (TypeDeclaration)
    comma:
      self: , (Symbol)
    type:
      self: INT64 (Type)
  - self: y (TypeDeclaration)
    in_out:
      self: INOUT (Keyword)
    type:
      self: INT64 (Type)
  rparen:
    self: ) (Symbol)
ident:
  self: . (DotOperator)
  left:
    self: dataset (Identifier)
  right:
    self: procede (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
stmt:
  self: BEGIN (BeginStatement)
  end:
    self: END (Keyword)
  stmts:
  - self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
    semicolon:
      self: ; (Symbol)
what:
  self: PROCEDURE (Keyword)
",
            0,
        )),
        // ----- CREATE ROW ACCESS POLICY statement -----
        Box::new(SuccessTestCase::new(
            "\
CREATE ROW ACCESS POLICY new_filter
ON tablename
FILTER USING (TRUE)
",
            "\
self: CREATE (CreateRowAccessPolicyStatement)
filter:
  self: FILTER (Keyword)
ident:
  self: new_filter (Identifier)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: tablename (Identifier)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: ( (GroupedExpr)
    expr:
      self: TRUE (BooleanLiteral)
    rparen:
      self: ) (Symbol)
what:
- self: ROW (Keyword)
- self: ACCESS (Keyword)
- self: POLICY (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
CREATE OR REPLACE ROW ACCESS POLICY IF NOT EXISTS new_filter
ON tablename
GRANT TO ('a.example.com', 'b.example.com')
FILTER USING (email = SESSION_USER())
;
",
            "\
self: CREATE (CreateRowAccessPolicyStatement)
filter:
  self: FILTER (Keyword)
grant:
  self: GRANT (Keyword)
ident:
  self: new_filter (Identifier)
if_not_exists:
- self: IF (Keyword)
- self: NOT (Keyword)
- self: EXISTS (Keyword)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: tablename (Identifier)
or_replace:
- self: OR (Keyword)
- self: REPLACE (Keyword)
semicolon:
  self: ; (Symbol)
to:
  self: TO (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: 'a.example.com' (StringLiteral)
      comma:
        self: , (Symbol)
    - self: 'b.example.com' (StringLiteral)
    rparen:
      self: ) (Symbol)
using:
  self: USING (KeywordWithExpr)
  expr:
    self: ( (GroupedExpr)
    expr:
      self: = (BinaryOperator)
      left:
        self: email (Identifier)
      right:
        self: ( (CallingFunction)
        func:
          self: SESSION_USER (Identifier)
        rparen:
          self: ) (Symbol)
    rparen:
      self: ) (Symbol)
what:
- self: ROW (Keyword)
- self: ACCESS (Keyword)
- self: POLICY (Keyword)
",
            0,
        )),
        // ----- ALTER SCHEMA statement -----
        Box::new(SuccessTestCase::new(
            "\
ALTER SCHEMA dataset_name SET OPTIONS();
",
            "\
self: ALTER (AlterSchemaStatement)
ident:
  self: dataset_name (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
set:
  self: SET (Keyword)
what:
  self: SCHEMA (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
ALTER SCHEMA IF EXISTS dataset_name SET OPTIONS(dummy = 'dummy');
",
            "\
self: ALTER (AlterSchemaStatement)
ident:
  self: dataset_name (Identifier)
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
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
set:
  self: SET (Keyword)
what:
  self: SCHEMA (Keyword)
",
            0,
        )),
        // ----- ALTER TABLE statement -----
        // SET
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE example SET OPTIONS(dummy='dummy');
",
            "\
self: ALTER (AlterTableStatement)
ident:
  self: example (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
set:
  self: SET (Keyword)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // ADD COLUMN
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE example
ADD COLUMN x INT64;
",
            "\
self: ALTER (AlterTableStatement)
add_columns:
- self: ADD (AddColumnClause)
  column:
    self: COLUMN (Keyword)
  type_declaration:
    self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE example
ADD COLUMN IF NOT EXISTS x INT64 OPTIONS(description = 'dummy'),
ADD COLUMN y STRUCT<z INT64 NOT NULL>;
",
            "\
self: ALTER (AlterTableStatement)
add_columns:
- self: ADD (AddColumnClause)
  column:
    self: COLUMN (Keyword)
  comma:
    self: , (Symbol)
  if_not_exists:
  - self: IF (Keyword)
  - self: NOT (Keyword)
  - self: EXISTS (Keyword)
  type_declaration:
    self: x (TypeDeclaration)
    type:
      self: INT64 (Type)
      options:
        self: OPTIONS (KeywordWithGroupedXXX)
        group:
          self: ( (GroupedExprs)
          exprs:
          - self: = (BinaryOperator)
            left:
              self: description (Identifier)
            right:
              self: 'dummy' (StringLiteral)
          rparen:
            self: ) (Symbol)
- self: ADD (AddColumnClause)
  column:
    self: COLUMN (Keyword)
  type_declaration:
    self: y (TypeDeclaration)
    type:
      self: STRUCT (Type)
      type_declaration:
        self: < (GroupedTypeDeclarations)
        declarations:
        - self: z (TypeDeclaration)
          type:
            self: INT64 (Type)
            not_null:
            - self: NOT (Keyword)
            - self: NULL (Keyword)
        rparen:
          self: > (Symbol)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // RENAME
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE dataset_name.t
RENAME TO u;
",
            "\
self: ALTER (AlterTableStatement)
ident:
  self: . (DotOperator)
  left:
    self: dataset_name (Identifier)
  right:
    self: t (Identifier)
rename:
  self: RENAME (Keyword)
semicolon:
  self: ; (Symbol)
to:
  self: TO (KeywordWithExpr)
  expr:
    self: u (Identifier)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // DROP
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE example
DROP COLUMN IF EXISTS x,
DROP COLUMN y
",
            "\
self: ALTER (AlterTableStatement)
drop_columns:
- self: DROP (DropColumnClause)
  column:
    self: COLUMN (Keyword)
  comma:
    self: , (Symbol)
  ident:
    self: x (Identifier)
  if_exists:
  - self: IF (Keyword)
  - self: EXISTS (Keyword)
- self: DROP (DropColumnClause)
  column:
    self: COLUMN (Keyword)
  ident:
    self: y (Identifier)
ident:
  self: example (Identifier)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // ----- ALTER COLUMN statement -----
        // DROP
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE t
ALTER COLUMN c DROP NOT NULL;
",
            "\
self: ALTER (AlterTableStatement)
alter_column_stmt:
  self: ALTER (AlterColumnStatement)
  drop_not_null:
  - self: DROP (Keyword)
  - self: NOT (Keyword)
  - self: NULL (Keyword)
  ident:
    self: c (Identifier)
  what:
    self: COLUMN (Keyword)
ident:
  self: t (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE IF EXISTS t
ALTER COLUMN IF EXISTS c DROP NOT NULL
",
            "\
self: ALTER (AlterTableStatement)
alter_column_stmt:
  self: ALTER (AlterColumnStatement)
  drop_not_null:
  - self: DROP (Keyword)
  - self: NOT (Keyword)
  - self: NULL (Keyword)
  ident:
    self: c (Identifier)
  if_exists:
  - self: IF (Keyword)
  - self: EXISTS (Keyword)
  what:
    self: COLUMN (Keyword)
ident:
  self: t (Identifier)
if_exists:
- self: IF (Keyword)
- self: EXISTS (Keyword)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // SET OPTIONS
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE t
ALTER COLUMN c SET OPTIONS(description = 'abc');
",
            "\
self: ALTER (AlterTableStatement)
alter_column_stmt:
  self: ALTER (AlterColumnStatement)
  ident:
    self: c (Identifier)
  options:
    self: OPTIONS (KeywordWithGroupedXXX)
    group:
      self: ( (GroupedExprs)
      exprs:
      - self: = (BinaryOperator)
        left:
          self: description (Identifier)
        right:
          self: 'abc' (StringLiteral)
      rparen:
        self: ) (Symbol)
  set:
    self: SET (Keyword)
  what:
    self: COLUMN (Keyword)
ident:
  self: t (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // SET DATA TYPE
        Box::new(SuccessTestCase::new(
            "\
ALTER TABLE t ALTER COLUMN int
SET DATA TYPE NUMERIC;
",
            "\
self: ALTER (AlterTableStatement)
alter_column_stmt:
  self: ALTER (AlterColumnStatement)
  data_type:
  - self: DATA (Keyword)
  - self: TYPE (Keyword)
  ident:
    self: int (Identifier)
  set:
    self: SET (Keyword)
  type:
    self: NUMERIC (Type)
  what:
    self: COLUMN (Keyword)
ident:
  self: t (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        // ----- ALTER VIEW statement -----
        Box::new(SuccessTestCase::new(
            "\
ALTER VIEW example SET OPTIONS(
  dummy = 'dummy',
  description = 'abc'
);
",
            "\
self: ALTER (AlterViewStatement)
ident:
  self: example (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      comma:
        self: , (Symbol)
      left:
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    - self: = (BinaryOperator)
      left:
        self: description (Identifier)
      right:
        self: 'abc' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
set:
  self: SET (Keyword)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        // MATERIALIZED
        Box::new(SuccessTestCase::new(
            "\
ALTER MATERIALIZED VIEW example SET OPTIONS(dummy = 'dummy');
",
            "\
self: ALTER (AlterViewStatement)
ident:
  self: example (Identifier)
materialized:
  self: MATERIALIZED (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      left:
        self: dummy (Identifier)
      right:
        self: 'dummy' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
set:
  self: SET (Keyword)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        // ----- DROP statement -----
        // general
        Box::new(SuccessTestCase::new(
            "\
DROP TABLE example;
",
            "\
self: DROP (DropStatement)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP EXTERNAL TABLE IF EXISTS example;
",
            "\
self: DROP (DropStatement)
external:
  self: EXTERNAL (Keyword)
ident:
  self: example (Identifier)
if_exists:
- self: IF (Keyword)
- self: EXISTS (Keyword)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP MATERIALIZED VIEW example;
",
            "\
self: DROP (DropStatement)
ident:
  self: example (Identifier)
materialized:
  self: MATERIALIZED (Keyword)
semicolon:
  self: ; (Symbol)
what:
  self: VIEW (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP SCHEMA example CASCADE;
",
            "\
self: DROP (DropStatement)
cascade_or_restrict:
  self: CASCADE (Keyword)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: SCHEMA (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP TABLE FUNCTION ident;
",
            "\
self: DROP (DropStatement)
ident:
  self: ident (Identifier)
semicolon:
  self: ; (Symbol)
table:
  self: TABLE (Keyword)
what:
  self: FUNCTION (Keyword)
",
            0,
        )),
        // row access policy
        Box::new(SuccessTestCase::new(
            "\
DROP ROW ACCESS POLICY ident ON tablename;
",
            "\
self: DROP (DropRowAccessPolicyStatement)
ident:
  self: ident (Identifier)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: tablename (Identifier)
semicolon:
  self: ; (Symbol)
what:
- self: ROW (Keyword)
- self: ACCESS (Keyword)
- self: POLICY (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP ROW ACCESS POLICY IF EXISTS ident ON tablename;
",
            "\
self: DROP (DropRowAccessPolicyStatement)
ident:
  self: ident (Identifier)
if_exists:
- self: IF (Keyword)
- self: EXISTS (Keyword)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: tablename (Identifier)
semicolon:
  self: ; (Symbol)
what:
- self: ROW (Keyword)
- self: ACCESS (Keyword)
- self: POLICY (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
DROP ALL ROW ACCESS POLICIES ON tablename;
",
            "\
self: DROP (DropRowAccessPolicyStatement)
on:
  self: ON (KeywordWithExpr)
  expr:
    self: tablename (Identifier)
semicolon:
  self: ; (Symbol)
what:
- self: ALL (Keyword)
- self: ROW (Keyword)
- self: ACCESS (Keyword)
- self: POLICIES (Keyword)
",
            0,
        )),
    ];
    for t in test_cases {
        t.test();
    }
}
