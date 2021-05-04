use super::*;
use difference::Changeset;

struct TestCase {
    code: String,
    expected_output: String,
}

impl TestCase {
    pub fn new(code: &str, expected_output: &str) -> TestCase {
        TestCase {
            code: code.to_string(),
            expected_output: expected_output.to_string(),
        }
    }
    pub fn test(&self) {
        let mut p = Parser::new(self.code.clone());
        let stmts = p.parse_code();
        println!(
            "========== testing ==========\n{}\n=============================\n",
            self.code.trim()
        );
        let result = stmts[0].to_string();
        let changeset = Changeset::new(self.expected_output.as_str(), result.as_str(), "\n");
        println!("{}\n", changeset.to_string());
        assert_eq!(2, stmts.len());
        assert_eq!(self.expected_output, result);
    }
    pub fn test_eof(&self) {
        let mut p = Parser::new(self.code.clone());
        let stmts = p.parse_code();
        println!(
            "========== testing ==========\n{}\n=============================\n",
            self.code.trim()
        );
        let result = stmts[1].to_string();
        let changeset = Changeset::new(self.expected_output.as_str(), result.as_str(), "\n");
        println!("{}\n", changeset.to_string());
        assert_eq!(2, stmts.len());
        assert_eq!(self.expected_output, result);
    }
}

#[test]
fn test_parse_code() {
    let test_cases = vec![
        TestCase::new(
            "\
SELECT 1;
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 1 (Unknown)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- grouped statement -----
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
  - self: 1 (Unknown)
",
        ),
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
- self: 1 (Unknown)
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
        // ----- alias -----
        TestCase::new(
            "\
SELECT 1 AS one, 2 two
FROM t1 AS t
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: 1 (Unknown)
  alias:
    self: one (Identifier)
  as:
    self: AS (Keyword)
  comma:
    self: , (Symbol)
- self: 2 (Unknown)
  alias:
    self: two (Identifier)
from:
  self: FROM (KeywordWithExpr)
  expr:
    self: t1 (Unknown)
    alias:
      self: t (Identifier)
    as:
      self: AS (Keyword)
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
    self: 1 (Unknown)
- self: + (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: 1 (Unknown)
- self: r (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: 'xxx' (Unknown)
- self: DATE (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: '2020-01-01' (Unknown)
- self: TIMESTAMP (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: r (UnaryOperator)
    right:
      self: '2020-01-01' (Unknown)
- self: NOT (UnaryOperator)
  comma:
    self: , (Symbol)
  right:
    self: TRUE (Unknown)
",
        ),
        // ----- binary operator -----
        TestCase::new(
            "\
SELECT
  1 + 2,
  1 BETWEEN 0 AND 3,
  1 IN (1, 2, 3),
  'x' LIKE '%x%',
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: + (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (Unknown)
  right:
    self: 2 (Unknown)
- self: BETWEEN (BetweenOperator)
  and:
    self: AND (Keyword)
  comma:
    self: , (Symbol)
  left:
    self: 1 (Unknown)
  right:
  - self: 0 (Unknown)
  - self: 3 (Unknown)
- self: IN (InOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (Unknown)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 1 (Unknown)
      comma:
        self: , (Symbol)
    - self: 2 (Unknown)
      comma:
        self: , (Symbol)
    - self: 3 (Unknown)
    rparen:
      self: ) (Symbol)
- self: LIKE (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 'x' (Unknown)
  right:
    self: '%x%' (Unknown)
",
        ),
        TestCase::new(
            "\
SELECT
  1 NOT BETWEEN 0 AND 3,
  1 NOT IN (1, 2, 3),
  'x' NOT LIKE '%x%',
  TRUE IS NOT FALSE,
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
    self: 1 (Unknown)
  not:
    self: NOT (Keyword)
  right:
  - self: 0 (Unknown)
  - self: 3 (Unknown)
- self: IN (InOperator)
  comma:
    self: , (Symbol)
  left:
    self: 1 (Unknown)
  not:
    self: NOT (Keyword)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 1 (Unknown)
      comma:
        self: , (Symbol)
    - self: 2 (Unknown)
      comma:
        self: , (Symbol)
    - self: 3 (Unknown)
    rparen:
      self: ) (Symbol)
- self: LIKE (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: 'x' (Unknown)
  not:
    self: NOT (Keyword)
  right:
    self: '%x%' (Unknown)
- self: IS (BinaryOperator)
  comma:
    self: , (Symbol)
  left:
    self: TRUE (Unknown)
  not:
    self: NOT (Keyword)
  right:
    self: FALSE (Unknown)
",
        ),
        // ----- precedence -----
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
          self: 1 (Unknown)
        right:
          self: ( (GroupedExpr)
          expr:
            self: - (UnaryOperator)
            right:
              self: 2 (Unknown)
          rparen:
            self: ) (Symbol)
      rparen:
        self: ) (Symbol)
    right:
      self: 3 (Unknown)
  right:
    self: ( (GroupedExprs)
    exprs:
    - self: 9 (Unknown)
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
            self: 1 (Unknown)
          right:
            self: 2 (Unknown)
        rparen:
          self: ) (Symbol)
      right:
        self: 3 (Unknown)
    not:
      self: NOT (Keyword)
    right:
    - self: + (BinaryOperator)
      left:
        self: 10 (Unknown)
      right:
        self: 0 (Unknown)
    - self: + (BinaryOperator)
      left:
        self: 11 (Unknown)
      right:
        self: 2 (Unknown)
  right:
    self: TRUE (Unknown)
",
        ),
        // ----- case expr -----
        TestCase::new(
            "\
SELECT
  CASE c1 WHEN 1 THEN 'one' WHEN 2 THEN 'two' ELSE NULL END,
  CASE WHEN c1 = 1 THEN 'one' ELSE NULL END,
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: CASE (Unknown)
  arms:
  - self: WHEN (CaseArm)
    expr:
      self: 1 (Unknown)
    result:
      self: 'one' (Unknown)
    then:
      self: THEN (Keyword)
  - self: WHEN (CaseArm)
    expr:
      self: 2 (Unknown)
    result:
      self: 'two' (Unknown)
    then:
      self: THEN (Keyword)
  - self: ELSE (CaseArm)
    result:
      self: NULL (Unknown)
  comma:
    self: , (Symbol)
  end:
    self: END (Keyword)
  expr:
    self: c1 (Unknown)
- self: CASE (Unknown)
  arms:
  - self: WHEN (CaseArm)
    expr:
      self: = (BinaryOperator)
      left:
        self: c1 (Unknown)
      right:
        self: 1 (Unknown)
    result:
      self: 'one' (Unknown)
    then:
      self: THEN (Keyword)
  - self: ELSE (CaseArm)
    result:
      self: NULL (Unknown)
  comma:
    self: , (Symbol)
  end:
    self: END (Keyword)
",
        ),
        // ----- calling function -----
        TestCase::new(
            "\
SELECT f(c1, c2)
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: c1 (Unknown)
    comma:
      self: , (Symbol)
  - self: c2 (Unknown)
  func:
    self: f (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
        // ----- irregular function -----
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
      self: '1' (Unknown)
    cast_to:
      self: INT64 (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: CAST (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT
    EXTRACT(DAY FROM ts),
    EXTRACT(WEEK(SUNDAY) FROM ts),
    EXTRACT(DAY FROM ts AT TIME ZONE 'UTC'),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    extract_datepart:
      self: DAY (Unknown)
    extract_from:
      self: ts (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: EXTRACT (Unknown)
  rparen:
    self: ) (Symbol)
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    extract_datepart:
      self: ( (CallingFunction)
      args:
      - self: SUNDAY (Unknown)
      func:
        self: WEEK (Unknown)
      rparen:
        self: ) (Symbol)
    extract_from:
      self: ts (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: EXTRACT (Unknown)
  rparen:
    self: ) (Symbol)
- self: ( (CallingFunction)
  args:
  - self: FROM (ExtractArgument)
    at_time_zone:
    - self: AT (Keyword)
    - self: TIME (Keyword)
    - self: ZONE (Keyword)
    extract_datepart:
      self: DAY (Unknown)
    extract_from:
      self: ts (Unknown)
    time_zone:
      self: 'UTC' (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: EXTRACT (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT STRING_AGG(DISTINCT x, y IGNORE NULLS ORDER BY z LIMIT 100),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: x (Unknown)
    comma:
      self: , (Symbol)
  - self: y (Unknown)
  comma:
    self: , (Symbol)
  distinct:
    self: DISTINCT (Keyword)
  func:
    self: STRING_AGG (Unknown)
  ignore_nulls:
  - self: IGNORE (Keyword)
  - self: NULLS (Keyword)
  limit:
    self: LIMIT (KeywordWithExpr)
    expr:
      self: 100 (Unknown)
  orderby:
    self: ORDER (XXXByExprs)
    by:
      self: BY (Keyword)
    exprs:
    - self: z (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
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
      - self: 1 (Unknown)
    right:
      self: SELECT (SelectStatement)
      exprs:
      - self: 2 (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: ARRAY (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT ST_GEOGFROMTEXT(p, oriented => TRUE),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: p (Unknown)
    comma:
      self: , (Symbol)
  - self: => (BinaryOperator)
    left:
      self: oriented (Unknown)
    right:
      self: TRUE (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: ST_GEOGFROMTEXT (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
        TestCase::new(
            "\
SELECT DATE_ADD(dt, INTERVAL 1 + 1 DAY),
",
            "\
self: SELECT (SelectStatement)
exprs:
- self: ( (CallingFunction)
  args:
  - self: dt (Unknown)
    comma:
      self: , (Symbol)
  - self: INTERVAL (Unknown)
    date_part:
      self: DAY (Unknown)
    right:
      self: + (BinaryOperator)
      left:
        self: 1 (Unknown)
      right:
        self: 1 (Unknown)
  comma:
    self: , (Symbol)
  func:
    self: DATE_ADD (Unknown)
  rparen:
    self: ) (Symbol)
",
        ),
    ];
    for t in test_cases {
        t.test();
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
        t.test_eof();
    }
}

//#[test]
//fn test_parse_exprs() {
//    let input = "\
//            SELECT null FROM data for system_time as of current_timestamp() tablesample system (20 percent) where true group by 1 HAVING true order by abc DESC, def limit 100 offset 10;
//            select
//              sum() over (),
//              sum() over named_clause,
//              sum() over (named_clause),
//              sum() over (partition by a),
//              sum() over (order by a),
//              sum() over (partition by a order by b, c),
//              sum() over (partition by a order by b, c rows between unbounded preceding and unbounded following),
//              sum() over (rows 1 + 1 preceding),
//            select last_value(col3) OVER (c ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING)
//            FROM table
//            WINDOW
//              a AS (PARTITION BY col1),
//              b AS (a ORDER BY col2),
//              c AS b;
//            select (t.struct_col.num + 1) as result from `dataset`.table as t;
//            select arr[offset(1)], [1, 2], ARRAY[1,2],array<int64>[1],array<struct<array<int64>>>[struct([1])];
//            select (1,2),struct(1,2),struct<int64>(1),struct<int64,x float64>(1,.1),struct<array<int64>>([1]),;
//            (select 1);
//            select 1 union all select 2;(select 1) union all select 2;select 1 union all (select 2);select 1 union all select 2 union all select 3;
//            select 1 union all (select 2 union all select 3);(select 1 union all select 2) union all select 3;
//            with a as (select 1) select 2;with a as (select 1), b as (select 2 from data where true) select 3;
//            select as struct 1;select distinct 1;select all 1;select t.* except (col1), * except(col1, col2), * replace (col1 * 2 as col2), from t;
//            select * from unnest([1,2,3]);select * from unnest([1]) with offset;select * from unnest([1]) a with offset as b;
//            select * from (select 1,2);select sub.* from (select 1,2) as sub;select * from main as m where not exists(select 1 from sub as s where s.x = m.x);
//            select * from (select 1 from table1) inner join table2;
//            select * from t order by col1 asc nulls last, col2 nulls first;
//            select * from data1 as one inner join data2 two ON true;
//            select * from data1 as one inner join data2 two using(col) left outer join data3 on true;
//            select * from data1 as one , data2 two join (data3 full outer join data4 on col1=col2) on true;
//              cast(abc as string),string_agg(distinct x, y ignore nulls order by z limit 100),array(select 1 union all select 2),
//              extract(day from ts),extract(day from ts at time zone 'UTC'),extract(week(sunday) from ts),
//              st_geogfromtext(p, oriented => true),

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
//            declare x int64;declare x,y default 1;
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
//            -- end comment
//"
