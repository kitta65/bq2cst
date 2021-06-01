use super::*;

#[test]
fn test_parse_code_ddl() {
    let test_cases = vec![
        // ----- CREATE SCHEMA statement -----
        TestCase::new(
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
        ),
        TestCase::new(
            "\
CREATE SCHEMA IF NOT EXISTS project_name.dataset_name OPTIONS();
",
            "\
self: CREATE (CreateSchemaStatement)
ident:
  self: . (BinaryOperator)
  left:
    self: project_name (Identifier)
  right:
    self: dataset_name (Identifier)
if_not_exists:
- self: IF (Keyword)
- self: NOT (Keyword)
- self: EXISTS (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
  group:
    self: ( (GroupedExprs)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
what:
  self: SCHEMA (Keyword)
",
        ),
        // ----- CREATE TABLE statement -----
        TestCase::new(
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
        ),
        TestCase::new(
            "\
CREATE TEMP TABLE example (x INT64, y INT64);
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
      self: INT64 (Type)
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
        ),
        TestCase::new(
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
        self: OPTIONS (KeywordWithGroupedExprs)
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
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: example (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        TestCase::new(
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
        ),
        // EXTERNAL
        TestCase::new(
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
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: new_table (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        TestCase::new(
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
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: new_table (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ----- CREATE VIEW statement -----
        TestCase::new(
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
        self: . (BinaryOperator)
        left:
          self: dataset (Identifier)
        right:
          self: table_name (Identifier)
ident:
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: view_name (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: VIEW (Keyword)
",
        ),
        TestCase::new(
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
  self: . (BinaryOperator)
  left:
    self: dataset_name (Identifier)
  right:
    self: view_name (Identifier)
what:
  self: VIEW (Keyword)
",
        ),
        // MATERIALIZED
        TestCase::new(
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
        self: . (BinaryOperator)
        left:
          self: dataset (Identifier)
        right:
          self: table_name (Identifier)
ident:
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: view_name (Identifier)
materialized:
  self: MATERIALIZED (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ----- CREATE FUNCTION statement -----
        // sql function definition
        TestCase::new(
            "\
CREATE OR REPLACE FUNCTION abc() AS (1);
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedExpr)
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
        ),
        TestCase::new(
            "\
CREATE TEMP FUNCTION abc(x INT64) AS (x);
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedExpr)
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
        ),
        TestCase::new(
            "\
CREATE FUNCTION IF NOT EXISTS abc(x ARRAY<INT64>, y ANY TYPE)
RETURNS INT64
AS ('dummy');
",
            "\
self: CREATE (CreateFunctionStatement)
as:
  self: AS (KeywordWithGroupedExpr)
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
        ),
        // javascript function definition
        TestCase::new(
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
    self: js (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        TestCase::new(
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
    self: js (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        TestCase::new(
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
    self: js (Keyword)
returns:
  self: RETURNS (KeywordWithType)
  type:
    self: INT64 (Type)
semicolon:
  self: ; (Symbol)
what:
  self: FUNCTION (Keyword)
",
        ),
        // ----- CREATE PROCEDURE statement -----
        TestCase::new(
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
  self: . (BinaryOperator)
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
        ),
        TestCase::new(
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
  self: . (BinaryOperator)
  left:
    self: dataset (Identifier)
  right:
    self: procede (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ----- ALTER SCHEMA statement -----
        TestCase::new(
            "\
ALTER SCHEMA dataset_name SET OPTIONS();
",
            "\
self: ALTER (AlterSchemaStatement)
ident:
  self: dataset_name (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        TestCase::new(
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
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ----- ALTER TABLE statement -----
        // SET
        TestCase::new(
            "\
ALTER TABLE example SET OPTIONS(dummy='dummy');
",
            "\
self: ALTER (AlterTableStatement)
ident:
  self: example (Identifier)
options:
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ADD COLUMN
        TestCase::new(
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
    self: x (Identifier)
    type:
      self: INT64 (Type)
ident:
  self: example (Identifier)
semicolon:
  self: ; (Symbol)
what:
  self: TABLE (Keyword)
",
        ),
        TestCase::new(
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
    self: x (Identifier)
    type:
      self: INT64 (Type)
      options:
        self: OPTIONS (KeywordWithGroupedExprs)
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
    self: y (Identifier)
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
        ),
        // DROP
        TestCase::new(
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
        ),
        // ----- ALTER COLUMN statement -----
        TestCase::new(
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
        ),
        TestCase::new(
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
        ),
        // ----- ALTER VIEW statement -----
        TestCase::new(
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
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // MATERIALIZED
        TestCase::new(
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
  self: OPTIONS (KeywordWithGroupedExprs)
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
        ),
        // ----- DROP statement -----
        TestCase::new(
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
        ),
        TestCase::new(
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
        ),
        TestCase::new(
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
        ),
        TestCase::new(
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
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}

