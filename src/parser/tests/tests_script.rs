use super::*;

#[test]
fn test_parse_code_select() {
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
  idents:
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
  idents:
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
        // ----- BEGIN statement -----
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
exception_stmts:
- self: SELECT (SelectStatement)
  exprs:
  - self: 2 (NumericLiteral)
  semicolon:
    self: ; (Symbol)
exception_when_error_then:
- self: EXCEPTION (Keyword)
- self: WHEN (Keyword)
- self: ERROR (Keyword)
- self: THEN (Keyword)
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
BEGIN EXCEPTiON WHEN ERROR THEN END;
",
            "\
self: BEGIN (BeginStatement)
end:
  self: END (Keyword)
exception_when_error_then:
- self: EXCEPTiON (Keyword)
- self: WHEN (Keyword)
- self: ERROR (Keyword)
- self: THEN (Keyword)
semicolon:
  self: ; (Symbol)
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
- self: ELSEIF (Keyword)
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
- self: ELSEIF (Keyword)
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
- self: ELSEIF (Keyword)
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
    ];
    for t in test_cases {
        t.test();
    }
}

//            create temp function abc(x int64) as (x);create function if not exists abc(x array<int64>, y int64) returns int64 as (x+y);create or replace function abc() as(1);
//            create function abc() returns int64 deterministic language js options(library=['dummy']) as '''return 1''';
//            create function abc() returns int64 language js options() as '''return 1''';
//            create function abc() returns int64 not deterministic language js as '''return 1''';
//            insert into table values(1,2);insert table values(1),(2);insert table (col) select 1;
//            delete table where true;delete table t where true;delete from table as t where not exists (select * from t where true);
//            truncate table t;
//            update table t set col1=1,col2=2 where true;update table1 as one set one.value=two.value from table2 as two where one.id = two.id;
//            update t1 set t1.flg=true from t2 inner join t3 on t2.id=t3.id where t1.id=t3.id;
//            merge t using s on t.id=s.id when matched then delete;
//            merge dataset.t t using dataset.s s on t.id=s.id
//            when not matched then insert row
//            when not matched by target then insert (id,value) values (1,2)
//            when not matched by source then update set id=999
//            when not matched by source and true then update set id=999,value=999

//            loop select 1; end loop;loop select 1;break; end loop;
//            while true do select 1; end while;
//            while true do iterate;leave;continue; end while;
//            raise;raise using message = 'error';
//            begin
//              begin
//                select 1;
//              exception when error then
//                raise using message='error';
//              end;
//            exception when error then
//              select @@error.message;
//            end;
//            call mydataset.myprocedure(1);
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
