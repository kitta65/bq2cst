use super::*;

#[test]
fn test_parse_code_dml() {
    let test_cases = vec![
        // ----- INSERT statement -----
        TestCase::new(
            "\
INSERT INTO TABLE VALUES(1,2);
",
            "\
self: INSERT (InsertStatement)
input:
  self: VALUES (KeywordWithExprs)
  exprs:
  - self: ( (GroupedExprs)
    exprs:
    - self: 1 (NumericLiteral)
      comma:
        self: , (Symbol)
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
into:
  self: INTO (Keyword)
semicolon:
  self: ; (Symbol)
target_name:
  self: TABLE (Identifier)
",
        ),
        TestCase::new(
            "\
INSERT table_name (col) VALUES(1),(2);
",
            "\
self: INSERT (InsertStatement)
columns:
  self: ( (GroupedExprs)
  exprs:
  - self: col (Identifier)
  rparen:
    self: ) (Symbol)
input:
  self: VALUES (KeywordWithExprs)
  exprs:
  - self: ( (GroupedExprs)
    comma:
      self: , (Symbol)
    exprs:
    - self: 1 (NumericLiteral)
    rparen:
      self: ) (Symbol)
  - self: ( (GroupedExprs)
    exprs:
    - self: 2 (NumericLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
target_name:
  self: table_name (Identifier)
",
        ),
        TestCase::new(
            "\
INSERT table_name (col1, col2) SELECT 1, 2;
",
            "\
self: INSERT (InsertStatement)
columns:
  self: ( (GroupedExprs)
  exprs:
  - self: col1 (Identifier)
    comma:
      self: , (Symbol)
  - self: col2 (Identifier)
  rparen:
    self: ) (Symbol)
input:
  self: SELECT (SelectStatement)
  exprs:
  - self: 1 (NumericLiteral)
    comma:
      self: , (Symbol)
  - self: 2 (NumericLiteral)
semicolon:
  self: ; (Symbol)
target_name:
  self: table_name (Identifier)
",
        ),
        // ----- DELETE statement -----
        TestCase::new(
            "\
DELETE table_name WHERE TRUE;
",
            "\
self: DELETE (DeleteStatement)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: TRUE (BooleanLiteral)
",
        ),
        TestCase::new(
            "\
DELETE table_name t WHERE TRUE;
",
            "\
self: DELETE (DeleteStatement)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
  alias:
    self: t (Identifier)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: TRUE (BooleanLiteral)
",
        ),
        TestCase::new(
            "\
DELETE FROM table_name AS t
WHERE NOT EXISTS (SELECT * FROM t WHERE TRUE);
",
            "\
self: DELETE (DeleteStatement)
from:
  self: FROM (Keyword)
semicolon:
  self: ; (Symbol)
table_name:
  self: table_name (Identifier)
  alias:
    self: t (Identifier)
  as:
    self: AS (Keyword)
where:
  self: WHERE (KeywordWithExpr)
  expr:
    self: NOT (UnaryOperator)
    right:
      self: ( (CallingFunction)
      args:
      - self: SELECT (SelectStatement)
        exprs:
        - self: * (Symbol)
        from:
          self: FROM (KeywordWithExpr)
          expr:
            self: t (Identifier)
        where:
          self: WHERE (KeywordWithExpr)
          expr:
            self: TRUE (BooleanLiteral)
      func:
        self: EXISTS (Identifier)
      rparen:
        self: ) (Symbol)
",
        ),
        // ----- TRUNCATE statement -----
        TestCase::new(
            "\
TRUNCATE table_name t;
",
            "\
self: TRUNCATE (TruncateStatement)
semicolon:
  self: ; (Symbol)
table:
  self: table_name (Keyword)
table_name:
  self: t (Identifier)
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

//            update table t set col1=1,col2=2 where true;update table1 as one set one.value=two.value from table2 as two where one.id = two.id;
//            update t1 set t1.flg=true from t2 inner join t3 on t2.id=t3.id where t1.id=t3.id;
//            merge t using s on t.id=s.id when matched then delete;
//            merge dataset.t t using dataset.s s on t.id=s.id
//            when not matched then insert row
//            when not matched by target then insert (id,value) values (1,2)
//            when not matched by source then update set id=999
//            when not matched by source and true then update set id=999,value=999

//            create table example (x int64);create temp table example (x int64, y int64);
//            CREATE  or replace TABLE dataset.example(x INT64 OPTIONS(description='dummy'))
//            PARTITION BY _PARTITIONDATE OPTIONS(partition_expiration_days=1);
//            create table if not exists example (x int64 not null) cluster by x as select 1;
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
//            create schema dataset_name;create schema if not exists project_name.dataset_name options();
//            alter table example set options(dummy='dummy');
//            alter view example set options(dummy='dummy',description='abc');
//            alter materialized view example set options(dummy='dummy');
//            alter table example add column x int64;
//            alter table example add column if not exists x int64 options(description='dummy'),add column y struct<z int64 not null>;
//            alter table example drop column if exists x,drop column y;
//            drop table example;drop external table if exists example;drop materialized view example;
//            drop schema dataset_name cascade;
