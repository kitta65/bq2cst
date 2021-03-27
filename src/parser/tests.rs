use super::*;
//use serde_json;
#[test]
fn test_next_token() {
    let input = "select *;".to_string();
    let l = lexer::Lexer::new(input);
    let mut p = Parser::new(l);
    assert_eq!(
        p.get_token(0),
        token::Token {
            line: 0,
            column: 0,
            literal: "select".to_string(),
        }
    );
    assert_eq!(
        p.get_token(1),
        token::Token {
            line: 0,
            column: 7,
            literal: "*".to_string(),
        }
    );
    p.next_token();
    assert_eq!(
        p.get_token(0),
        token::Token {
            line: 0,
            column: 7,
            literal: "*".to_string(),
        }
    );
    assert_eq!(
        p.get_token(1),
        token::Token {
            line: 0,
            column: 8,
            literal: ";".to_string(),
        }
    );
    p.next_token();
    assert_eq!(
        p.get_token(0),
        token::Token {
            line: 0,
            column: 8,
            literal: ";".to_string(),
        }
    );
    assert_eq!(
        p.get_token(1),
        token::Token::new(usize::MAX, usize::MAX, "")
    );
    p.next_token();
    assert_eq!(
        p.get_token(0),
        token::Token::new(usize::MAX, usize::MAX, "")
    );
    assert_eq!(
        p.get_token(1),
        token::Token::new(usize::MAX, usize::MAX, "")
    );
}
#[test]
fn test_parse_exprs() {
    let input = "\
            SELECT 'aaa', 123, null FROM data for system_time as of current_timestamp() where true group by 1 HAVING true order by abc DESC, def limit 100 offset 10;
            select 1 as num from data;
            select 2 two;
            select
              -1, 1+1+1, date '2020-02-24', TIMESTAMP '2020-01-01', interval 9 year,
              if(true, 'true'), (1+1)*1, ((2)), (select info limit 1), 'a' not like '%a',
              10 between 1 and 2 and true,
              1<2,
            from data where 1 in (1, 2)
            ;
            select not true or a and b,;
            -- head
            select/* */current_date( -- lparen
            /* */-- args
            );
            select
              case num when 1 then '1' else '0' end,
              case when true then '1' else '0' end,
              case when 1=1 then current_date() else null end,
            ;
            select
              sum() over (),
              sum() over named_clause,
              sum() over (named_clause),
              sum() over (partition by a),
              sum() over (order by a),
              sum() over (partition by a order by b, c),
              sum() over (partition by a order by b, c rows between unbounded preceding and unbounded following),
              sum() over (rows 1 + 1 preceding),
            ;
            select last_value(col3) OVER (c ROWS BETWEEN 2 PRECEDING AND 2 FOLLOWING)
            FROM table
            WINDOW
              a AS (PARTITION BY col1),
              b AS (a ORDER BY col2),
              c AS b;
            select r'abc', B'abc', rB'abc', bR'abc', date r'2020-01-01';
            select decimal '00', timestamp r'2020-01-01';
            select (t.struct_col.num + 1) as result from `dataset`.table as t;
            select arr[offset(1)], [1, 2], ARRAY[1,2],array<int64>[1],array<struct<array<int64>>>[struct([1])];
            select (1,2),struct(1,2),struct<int64>(1),struct<int64,x float64>(1,.1),struct<array<int64>>([1]),;
            (select 1);
            select 1 union all select 2;(select 1) union all select 2;select 1 union all (select 2);select 1 union all select 2 union all select 3;
            select 1 union all (select 2 union all select 3);(select 1 union all select 2) union all select 3;
            with a as (select 1) select 2;with a as (select 1), b as (select 2) select 3;
            select as struct 1;select distinct 1;select all 1;select t.* except (col1), * except(col1, col2), * replace (col1 * 2 as col2), from t;
            select * from unnest([1,2,3]);select * from unnest([1]) with offset;select * from unnest([1]) a with offset as b;
            select * from (select 1,2);select sub.* from (select 1,2) as sub;select * from main as m where not exists(select 1 from sub as s where s.x = m.x);
            select * from t order by col1 asc nulls last, col2 nulls first;
            select * from data1 as one inner join data2 two ON true;
            select * from data1 as one , data2 two join (data3 full outer join data4 on col1=col2) on true;
            select
              cast(abc as string),string_agg(distinct x, y ignore nulls order by z limit 100),array(select 1 union all select 2),
              extract(day from ts),extract(day from ts at time zone 'UTC'),extract(week(sunday) from ts),
              st_geogfromtext(p, oriented => true),
            ;
            select 1;
            create temp function abc(x int64) as (x);create function if not exists abc(x array<int64>, y int64) returns int64 as (x+y);create or replace function abc() as(1);
            create function abc() returns int64 deterministic language js options(library=['dummy']) as '''return 1''';
            create function abc() returns int64 language js options() as '''return 1''';
            create function abc() returns int64 not deterministic language js as '''return 1''';
"
            .to_string();
    let l = lexer::Lexer::new(input);
    let mut p = Parser::new(l);
    let stmt = p.parse_code();
    let tests = vec![
        // simple select
        "\
self: SELECT
exprs:
- self: 'aaa'
  comma:
    self: ,
- self: 123
  comma:
    self: ,
- self: null
from:
  self: FROM
  expr:
    self: data
    for_system_time_as_of:
      self: for
      expr:
        self: (
        func:
          self: current_timestamp
        rparen:
          self: )
      system_time_as_of:
      - self: system_time
      - self: as
      - self: of
groupby:
  self: group
  by:
    self: by
  exprs:
  - self: 1
having:
  self: HAVING
  expr:
    self: true
limit:
  self: limit
  expr:
    self: 100
  offset:
    self: offset
    expr:
      self: 10
orderby:
  self: order
  by:
    self: by
  exprs:
  - self: abc
    comma:
      self: ,
    order:
      self: DESC
  - self: def
semicolon:
  self: ;
where:
  self: where
  expr:
    self: true",
        // alias
        "\
self: select
exprs:
- self: 1
  as:
    self: as
    alias:
      self: num
from:
  self: from
  expr:
    self: data
semicolon:
  self: ;",
        // implicit alias
        "\
self: select
exprs:
- self: 2
  as:
    self: None
    alias:
      self: two
semicolon:
  self: ;",
        // parse_expr precedence
        "\
self: select
exprs:
- self: -
  comma:
    self: ,
  right:
    self: 1
- self: +
  comma:
    self: ,
  left:
    self: +
    left:
      self: 1
    right:
      self: 1
  right:
    self: 1
- self: date
  comma:
    self: ,
  right:
    self: '2020-02-24'
- self: TIMESTAMP
  comma:
    self: ,
  right:
    self: '2020-01-01'
- self: interval
  comma:
    self: ,
  date_part:
    self: year
  right:
    self: 9
- self: (
  args:
  - self: true
    comma:
      self: ,
  - self: 'true'
  comma:
    self: ,
  func:
    self: if
  rparen:
    self: )
- self: *
  comma:
    self: ,
  left:
    self: (
    expr:
      self: +
      left:
        self: 1
      right:
        self: 1
    rparen:
      self: )
  right:
    self: 1
- self: (
  comma:
    self: ,
  expr:
    self: (
    expr:
      self: 2
    rparen:
      self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  expr:
    self: select
    exprs:
    - self: info
    limit:
      self: limit
      expr:
        self: 1
  rparen:
    self: )
- self: like
  comma:
    self: ,
  left:
    self: 'a'
  not:
    self: not
  right:
    self: '%a'
- self: and
  comma:
    self: ,
  left:
    self: between
    and:
      self: and
    left:
      self: 10
    right:
    - self: 1
    - self: 2
  right:
    self: true
- self: <
  comma:
    self: ,
  left:
    self: 1
  right:
    self: 2
from:
  self: from
  expr:
    self: data
semicolon:
  self: ;
where:
  self: where
  expr:
    self: in
    left:
      self: 1
    right:
      self: (
      exprs:
      - self: 1
        comma:
          self: ,
      - self: 2
      rparen:
        self: )",
        // not, and, or
        "\
self: select
exprs:
- self: or
  comma:
    self: ,
  left:
    self: not
    right:
      self: true
  right:
    self: and
    left:
      self: a
    right:
      self: b
semicolon:
  self: ;",
        // comment
        "\
self: select
exprs:
- self: (
  following_comments:
  - self: -- lparen
  func:
    self: current_date
  rparen:
    self: )
    leading_comments:
    - self: /* */
    - self: -- args
following_comments:
- self: /* */
leading_comments:
- self: -- head
semicolon:
  self: ;",
        // case when
        "\
self: select
exprs:
- self: case
  arms:
  - self: when
    expr:
      self: 1
    result:
      self: '1'
    then:
      self: then
  - self: else
    result:
      self: '0'
  comma:
    self: ,
  end:
    self: end
  expr:
    self: num
- self: case
  arms:
  - self: when
    expr:
      self: true
    result:
      self: '1'
    then:
      self: then
  - self: else
    result:
      self: '0'
  comma:
    self: ,
  end:
    self: end
- self: case
  arms:
  - self: when
    expr:
      self: =
      left:
        self: 1
      right:
        self: 1
    result:
      self: (
      func:
        self: current_date
      rparen:
        self: )
    then:
      self: then
  - self: else
    result:
      self: null
  comma:
    self: ,
  end:
    self: end
semicolon:
  self: ;",
        // window function
        "\
self: select
exprs:
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: named_clause
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      name:
        self: named_clause
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      partitionby:
        self: partition
        by:
          self: by
        exprs:
        - self: a
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      orderby:
        self: order
        by:
          self: by
        exprs:
        - self: a
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      orderby:
        self: order
        by:
          self: by
        exprs:
        - self: b
          comma:
            self: ,
        - self: c
      partitionby:
        self: partition
        by:
          self: by
        exprs:
        - self: a
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      frame:
        self: rows
        and:
          self: and
        between:
          self: between
        end:
          self: unbounded
          following:
            self: following
        start:
          self: unbounded
          preceding:
            self: preceding
      orderby:
        self: order
        by:
          self: by
        exprs:
        - self: b
          comma:
            self: ,
        - self: c
      partitionby:
        self: partition
        by:
          self: by
        exprs:
        - self: a
      rparen:
        self: )
  rparen:
    self: )
- self: (
  comma:
    self: ,
  func:
    self: sum
  over:
    self: over
    window:
      self: (
      frame:
        self: rows
        start:
          self: +
          left:
            self: 1
          preceding:
            self: preceding
          right:
            self: 1
      rparen:
        self: )
  rparen:
    self: )
semicolon:
  self: ;",
        // window clause
        "\
self: select
exprs:
- self: (
  args:
  - self: col3
  func:
    self: last_value
  over:
    self: OVER
    window:
      self: (
      frame:
        self: ROWS
        and:
          self: AND
        between:
          self: BETWEEN
        end:
          self: 2
          following:
            self: FOLLOWING
        start:
          self: 2
          preceding:
            self: PRECEDING
      name:
        self: c
      rparen:
        self: )
  rparen:
    self: )
from:
  self: FROM
  expr:
    self: table
semicolon:
  self: ;
window:
  self: WINDOW
  window_exprs:
  - self: a
    as:
      self: AS
    comma:
      self: ,
    window:
      self: (
      partitionby:
        self: PARTITION
        by:
          self: BY
        exprs:
        - self: col1
      rparen:
        self: )
  - self: b
    as:
      self: AS
    comma:
      self: ,
    window:
      self: (
      name:
        self: a
      orderby:
        self: ORDER
        by:
          self: BY
        exprs:
        - self: col2
      rparen:
        self: )
  - self: c
    as:
      self: AS
    window:
      self: b",
        // raw, bytes
        "\
self: select
exprs:
- self: r
  comma:
    self: ,
  right:
    self: 'abc'
- self: B
  comma:
    self: ,
  right:
    self: 'abc'
- self: rB
  comma:
    self: ,
  right:
    self: 'abc'
- self: bR
  comma:
    self: ,
  right:
    self: 'abc'
- self: date
  right:
    self: r
    right:
      self: '2020-01-01'
semicolon:
  self: ;",
        // date, timestamp, numeric...
        "\
self: select
exprs:
- self: decimal
  comma:
    self: ,
  right:
    self: '00'
- self: timestamp
  right:
    self: r
    right:
      self: '2020-01-01'
semicolon:
  self: ;",
        // dot operator
        "\
self: select
exprs:
- self: (
  as:
    self: as
    alias:
      self: result
  expr:
    self: +
    left:
      self: .
      left:
        self: .
        left:
          self: t
        right:
          self: struct_col
      right:
        self: num
    right:
      self: 1
  rparen:
    self: )
from:
  self: from
  expr:
    self: .
    as:
      self: as
      alias:
        self: t
    left:
      self: `dataset`
    right:
      self: table
semicolon:
  self: ;", // array
        "\
self: select
exprs:
- self: [
  comma:
    self: ,
  left:
    self: arr
  right:
    self: (
    args:
    - self: 1
    func:
      self: offset
    rparen:
      self: )
  rparen:
    self: ]
- self: [
  comma:
    self: ,
  exprs:
  - self: 1
    comma:
      self: ,
  - self: 2
  rparen:
    self: ]
- self: ARRAY
  comma:
    self: ,
  right:
    self: [
    exprs:
    - self: 1
      comma:
        self: ,
    - self: 2
    rparen:
      self: ]
- self: array
  comma:
    self: ,
  right:
    self: [
    exprs:
    - self: 1
    rparen:
      self: ]
  type_declaration:
    self: <
    rparen:
      self: >
    type:
      self: int64
- self: array
  right:
    self: [
    exprs:
    - self: struct
      right:
        self: (
        exprs:
        - self: [
          exprs:
          - self: 1
          rparen:
            self: ]
        rparen:
          self: )
    rparen:
      self: ]
  type_declaration:
    self: <
    rparen:
      self: >
    type:
      self: struct
      type_declaration:
        self: <
        declarations:
        - self: None
          type:
            self: array
            type_declaration:
              self: <
              rparen:
                self: >
              type:
                self: int64
        rparen:
          self: >
semicolon:
  self: ;", // struct
        "\
self: select
exprs:
- self: (
  comma:
    self: ,
  exprs:
  - self: 1
    comma:
      self: ,
  - self: 2
  rparen:
    self: )
- self: struct
  comma:
    self: ,
  right:
    self: (
    exprs:
    - self: 1
      comma:
        self: ,
    - self: 2
    rparen:
      self: )
- self: struct
  comma:
    self: ,
  right:
    self: (
    exprs:
    - self: 1
    rparen:
      self: )
  type_declaration:
    self: <
    declarations:
    - self: None
      type:
        self: int64
    rparen:
      self: >
- self: struct
  comma:
    self: ,
  right:
    self: (
    exprs:
    - self: 1
      comma:
        self: ,
    - self: .1
    rparen:
      self: )
  type_declaration:
    self: <
    declarations:
    - self: None
      comma:
        self: ,
      type:
        self: int64
    - self: x
      type:
        self: float64
    rparen:
      self: >
- self: struct
  comma:
    self: ,
  right:
    self: (
    exprs:
    - self: [
      exprs:
      - self: 1
      rparen:
        self: ]
    rparen:
      self: )
  type_declaration:
    self: <
    declarations:
    - self: None
      type:
        self: array
        type_declaration:
          self: <
          rparen:
            self: >
          type:
            self: int64
    rparen:
      self: >
semicolon:
  self: ;",
        // grouped select
        "\
self: (
rparen:
  self: )
semicolon:
  self: ;
stmt:
  self: select
  exprs:
  - self: 1",
        // union
        "\
self: union
distinct:
  self: all
left:
  self: select
  exprs:
  - self: 1
right:
  self: select
  exprs:
  - self: 2
semicolon:
  self: ;",
        "\
self: union
distinct:
  self: all
left:
  self: (
  rparen:
    self: )
  stmt:
    self: select
    exprs:
    - self: 1
right:
  self: select
  exprs:
  - self: 2
semicolon:
  self: ;",
        "\
self: union
distinct:
  self: all
left:
  self: select
  exprs:
  - self: 1
right:
  self: (
  rparen:
    self: )
  stmt:
    self: select
    exprs:
    - self: 2
semicolon:
  self: ;",
        "\
self: union
distinct:
  self: all
left:
  self: union
  distinct:
    self: all
  left:
    self: select
    exprs:
    - self: 1
  right:
    self: select
    exprs:
    - self: 2
right:
  self: select
  exprs:
  - self: 3
semicolon:
  self: ;",
        "\
self: union
distinct:
  self: all
left:
  self: select
  exprs:
  - self: 1
right:
  self: (
  rparen:
    self: )
  stmt:
    self: union
    distinct:
      self: all
    left:
      self: select
      exprs:
      - self: 2
    right:
      self: select
      exprs:
      - self: 3
semicolon:
  self: ;",
        "\
self: union
distinct:
  self: all
left:
  self: (
  rparen:
    self: )
  stmt:
    self: union
    distinct:
      self: all
    left:
      self: select
      exprs:
      - self: 1
    right:
      self: select
      exprs:
      - self: 2
right:
  self: select
  exprs:
  - self: 3
semicolon:
  self: ;", // with
        "\
self: select
exprs:
- self: 2
semicolon:
  self: ;
with:
  self: with
  queries:
  - self: a
    as:
      self: as
    stmt:
      self: (
      rparen:
        self: )
      stmt:
        self: select
        exprs:
        - self: 1",
        "\
self: select
exprs:
- self: 3
semicolon:
  self: ;
with:
  self: with
  queries:
  - self: a
    as:
      self: as
    comma:
      self: ,
    stmt:
      self: (
      rparen:
        self: )
      stmt:
        self: select
        exprs:
        - self: 1
  - self: b
    as:
      self: as
    stmt:
      self: (
      rparen:
        self: )
      stmt:
        self: select
        exprs:
        - self: 2",
        // optional keyword
        "\
self: select
as:
  self: as
  struct_value:
    self: struct
exprs:
- self: 1
semicolon:
  self: ;",
        "\
self: select
distinct:
  self: distinct
exprs:
- self: 1
semicolon:
  self: ;",
        "\
self: select
distinct:
  self: all
exprs:
- self: 1
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: .
  comma:
    self: ,
  left:
    self: t
  right:
    self: *
    except:
      self: except
      group:
        self: (
        exprs:
        - self: col1
        rparen:
          self: )
- self: *
  comma:
    self: ,
  except:
    self: except
    group:
      self: (
      exprs:
      - self: col1
        comma:
          self: ,
      - self: col2
      rparen:
        self: )
- self: *
  comma:
    self: ,
  replace:
    self: replace
    group:
      self: (
      exprs:
      - self: *
        as:
          self: as
          alias:
            self: col2
        left:
          self: col1
        right:
          self: 2
      rparen:
        self: )
from:
  self: from
  expr:
    self: t
semicolon:
  self: ;",
        // unnest
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: (
    args:
    - self: [
      exprs:
      - self: 1
        comma:
          self: ,
      - self: 2
        comma:
          self: ,
      - self: 3
      rparen:
        self: ]
    func:
      self: unnest
    rparen:
      self: )
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: (
    args:
    - self: [
      exprs:
      - self: 1
      rparen:
        self: ]
    func:
      self: unnest
    rparen:
      self: )
    with:
      self: with
      unnest_offset:
        self: offset
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: (
    args:
    - self: [
      exprs:
      - self: 1
      rparen:
        self: ]
    as:
      self: None
      alias:
        self: a
    func:
      self: unnest
    rparen:
      self: )
    with:
      self: with
      unnest_offset:
        self: offset
        as:
          self: as
          alias:
            self: b
semicolon:
  self: ;",
        // subquery
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: (
    expr:
      self: select
      exprs:
      - self: 1
        comma:
          self: ,
      - self: 2
    rparen:
      self: )
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: .
  left:
    self: sub
  right:
    self: *
from:
  self: from
  expr:
    self: (
    as:
      self: as
      alias:
        self: sub
    expr:
      self: select
      exprs:
      - self: 1
        comma:
          self: ,
      - self: 2
    rparen:
      self: )
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: main
    as:
      self: as
      alias:
        self: m
semicolon:
  self: ;
where:
  self: where
  expr:
    self: not
    right:
      self: (
      args:
      - self: select
        exprs:
        - self: 1
        from:
          self: from
          expr:
            self: sub
            as:
              self: as
              alias:
                self: s
        where:
          self: where
          expr:
            self: =
            left:
              self: .
              left:
                self: s
              right:
                self: x
            right:
              self: .
              left:
                self: m
              right:
                self: x
      func:
        self: exists
      rparen:
        self: )",
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: t
orderby:
  self: order
  by:
    self: by
  exprs:
  - self: col1
    comma:
      self: ,
    null_order:
      self: nulls
      first:
        self: last
    order:
      self: asc
  - self: col2
    null_order:
      self: nulls
      first:
        self: first
semicolon:
  self: ;",
        // join
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: join
    join_type:
      self: inner
    left:
      self: data1
      as:
        self: as
        alias:
          self: one
    on:
      self: ON
      expr:
        self: true
    right:
      self: data2
      as:
        self: None
        alias:
          self: two
semicolon:
  self: ;",
        "\
self: select
exprs:
- self: *
from:
  self: from
  expr:
    self: join
    left:
      self: ,
      left:
        self: data1
        as:
          self: as
          alias:
            self: one
      right:
        self: data2
        as:
          self: None
          alias:
            self: two
    on:
      self: on
      expr:
        self: true
    right:
      self: (
      expr:
        self: join
        join_type:
          self: full
          outer:
            self: outer
        left:
          self: data3
        on:
          self: on
          expr:
            self: =
            left:
              self: col1
            right:
              self: col2
        right:
          self: data4
      rparen:
        self: )
semicolon:
  self: ;",
  // irregular functions
  "\
self: select
exprs:
- self: (
  args:
  - self: as
    cast_from:
      self: abc
    cast_to:
      self: string
  comma:
    self: ,
  func:
    self: cast
  rparen:
    self: )
- self: (
  args:
  - self: x
    comma:
      self: ,
  - self: y
  comma:
    self: ,
  distinct:
    self: distinct
  func:
    self: string_agg
  ignore_nulls:
    self: ignore
    nulls:
      self: nulls
  limit:
    self: limit
    expr:
      self: 100
  orderby:
    self: order
    by:
      self: by
    exprs:
    - self: z
  rparen:
    self: )
- self: (
  args:
  - self: union
    distinct:
      self: all
    left:
      self: select
      exprs:
      - self: 1
    right:
      self: select
      exprs:
      - self: 2
  comma:
    self: ,
  func:
    self: array
  rparen:
    self: )
- self: (
  args:
  - self: from
    extract_datepart:
      self: day
    extract_from:
      self: ts
  comma:
    self: ,
  func:
    self: extract
  rparen:
    self: )
- self: (
  args:
  - self: from
    at:
      self: at
      expr:
        self: 'UTC'
      time_zone:
      - self: time
      - self: zone
    extract_datepart:
      self: day
    extract_from:
      self: ts
  comma:
    self: ,
  func:
    self: extract
  rparen:
    self: )
- self: (
  args:
  - self: from
    extract_datepart:
      self: (
      args:
      - self: sunday
      func:
        self: week
      rparen:
        self: )
    extract_from:
      self: ts
  comma:
    self: ,
  func:
    self: extract
  rparen:
    self: )
- self: (
  args:
  - self: p
    comma:
      self: ,
  - self: =>
    left:
      self: oriented
    right:
      self: true
  comma:
    self: ,
  func:
    self: st_geogfromtext
  rparen:
    self: )
semicolon:
  self: ;",
  //
  "\
self: select
exprs:
- self: 1
semicolon:
  self: ;",
        // create function
        "\
self: create
as:
  self: as
  group:
    self: (
    expr:
      self: x
    rparen:
      self: )
group:
  self: (
  args:
  - self: x
    type:
      self: int64
  rparen:
    self: )
ident:
  self: abc
semicolon:
  self: ;
temp:
  self: temp
what:
  self: function",
        "\
self: create
as:
  self: as
  group:
    self: (
    expr:
      self: +
      left:
        self: x
      right:
        self: y
    rparen:
      self: )
group:
  self: (
  args:
  - self: x
    comma:
      self: ,
    type:
      self: array
      type_declaration:
        self: <
        rparen:
          self: >
        type:
          self: int64
  - self: y
    type:
      self: int64
  rparen:
    self: )
ident:
  self: abc
if_not_exists:
- self: if
- self: not
- self: exists
returns:
  self: returns
  type:
    self: int64
semicolon:
  self: ;
what:
  self: function",
        "\
self: create
as:
  self: as
  group:
    self: (
    expr:
      self: 1
    rparen:
      self: )
group:
  self: (
  rparen:
    self: )
ident:
  self: abc
or_replace:
- self: or
- self: replace
semicolon:
  self: ;
what:
  self: function",
        "\
self: create
as:
  self: as
  expr:
    self: '''return 1'''
determinism:
  self: deterministic
group:
  self: (
  rparen:
    self: )
ident:
  self: abc
language:
  self: language
  language:
    self: js
options:
  self: options
  group:
    self: (
    exprs:
    - self: =
      left:
        self: library
      right:
        self: [
        exprs:
        - self: 'dummy'
        rparen:
          self: ]
    rparen:
      self: )
returns:
  self: returns
  type:
    self: int64
semicolon:
  self: ;
what:
  self: function",
        "\
self: create
as:
  self: as
  expr:
    self: '''return 1'''
group:
  self: (
  rparen:
    self: )
ident:
  self: abc
language:
  self: language
  language:
    self: js
options:
  self: options
  group:
    self: (
    rparen:
      self: )
returns:
  self: returns
  type:
    self: int64
semicolon:
  self: ;
what:
  self: function",
        "\
self: create
as:
  self: as
  expr:
    self: '''return 1'''
determinism:
  self: not
  right:
    self: deterministic
group:
  self: (
  rparen:
    self: )
ident:
  self: abc
language:
  self: language
  language:
    self: js
returns:
  self: returns
  type:
    self: int64
semicolon:
  self: ;
what:
  self: function",
    ];
    for i in 0..tests.len() {
        println!("{}\n", stmt[i].to_string(0, false));
        assert_eq!(stmt[i].to_string(0, false), tests[i]);
    }
}
#[test]
fn test_get_offset_index() {
    let input = "\
#standardSQL
select -- comment
-- comment2
*;"
    .to_string();
    let l = lexer::Lexer::new(input);
    let mut p = Parser::new(l);
    assert_eq!(p.position, 1); // select
    assert_eq!(p.get_offset_index(1), 4); // *
    assert_eq!(p.get_offset_index(2), 5); // ;
    assert_eq!(p.get_offset_index(3), 6); // ;
    p.next_token();
    assert_eq!(p.position, 4);
    p.next_token();
    assert_eq!(p.position, 5);
    assert_eq!(p.get_offset_index(1), 6);
    assert_eq!(p.get_offset_index(2), 6);
}
