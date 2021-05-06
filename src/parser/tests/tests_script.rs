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
        // check root argument
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
//            ;
//            set x=5;set (x,y)=(1,2);set (x,y)=(select as struct 1,2);
//            execute immediate 'select 1';execute immediate 'select ?,?' into x,y using 1,2;execute immediate 'select @x' into x using 1 as x;
//            begin select 1;select 2;end;begin select 1;exception when error then select 2;end;begin exception when error then end;
//            if true then end if;
//            if true then select 1; select 2;end if;
//            if true then select 1; elseif true then end if;
//            if true then elseif true then select 1; elseif true then select 2; select 3; else end if;
//            if true then else select 1; end if;
//            if true then else select 1;select 2; end if;
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
