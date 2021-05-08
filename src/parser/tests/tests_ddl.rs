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
  self: . (Identifier)
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
  self: . (Identifier)
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
AS SELECT 1;
",
            "\
self: CREATE (CreateTableStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
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
    ];
    for t in test_cases {
        t.test();
    }
}

//            create temp function abc(x int64) as (x);create function if not exists abc(x array<int64>, y int64) returns int64 as (x+y);create or replace function abc() as(1);
//            create function abc() returns int64 deterministic language js options(library=['dummy']) as '''return 1''';
//            create function abc() returns int64 language js options() as '''return 1''';
//            create function abc() returns int64 not deterministic language js as '''return 1''';

//            create view dataset.new_table as select * from dataset.old_table;
//            create materialized view dataset.new_table options(dummy='dummy') as select count(*) from dataset.old_table;
//            CREATE EXTERNAL TABLE dataset.new_table
//            WITH PARTITION COLUMNS
//            OPTIONS (
//              uris=['dummy'],
//              format=csv
//            );
//            CREATE EXTERNAL TABLE dataset.new_table
//            WITH PARTITION COLUMNS (
//                col1 string
//            )
//            OPTIONS (
//              uris=['dummy'],
//              format=csv
//            );
//            CREATE PROCEDURE dataset.procede() BEGIN SELECT 1; END;
//            CREATE PROCEDURE dataset.procede(x int64, inout y int64) options(dummy='dummy') BEGIN SELECT 1; END;
//            alter table example set options(dummy='dummy');
//            alter view example set options(dummy='dummy',description='abc');
//            alter materialized view example set options(dummy='dummy');
//            alter table example add column x int64;
//            alter table example add column if not exists x int64 options(description='dummy'),add column y struct<z int64 not null>;
//            alter table example drop column if exists x,drop column y;
//            drop table example;drop external table if exists example;drop materialized view example;
//            drop schema dataset_name cascade;
