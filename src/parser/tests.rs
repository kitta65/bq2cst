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
            SELECT 'aaa', 123 FROM data where true group by 1 HAVING true order by abc DESC, def limit 100;
            select 1 as num from data;
            select 2 two;
            select * from data1 as one inner join data2 two ON true;
            select -1, 1+1+1, date '2020-02-24', TIMESTAMP '2020-01-01', interval 9 year, if(true, 'true'), (1+1)*1, ((2)), (select info limit 1), 'a' not like '%a', 10 between 1 and 2 and true,
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
            select r'abc', B'abc', rB'abc', bR'abc', date r'2020-01-01';
            select decimal '00', timestamp r'2020-01-01';
            select (t.struct_col.num + 1) as result from `dataset`.table as t;
            select arr[offset(1)], [1, 2], ARRAY[1,2],array<int64>[1],array<struct<array<int64>>>[struct([1])];
            select (1,2),struct(1,2),struct<int64>(1),struct<int64,x float64>(1,.1),struct<array<int64>>([1]),;
            (select 1);
            select 1 union select 2;(select 1) union select 2;select 1 union (select 2);select 1 union select 2 union select 3;
            select 1 union (select 2 union select 3);(select 1 union select 2) union select 3;"
            .to_string();
    let l = lexer::Lexer::new(input);
    let mut p = Parser::new(l);
    let stmt = p.parse_code();
    let tests = vec![
        // simple select
        "\
self: SELECT
columns:
- self: 'aaa'
  comma:
    self: ,
- self: 123
from:
  self: FROM
  tables:
  - self: data
groupby:
  self: group
  by:
    self: by
  columns:
  - self: 1
having:
  self: HAVING
  expr:
    self: true
limit:
  self: limit
  expr:
    self: 100
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
columns:
- self: 1
  as:
    self: as
    alias:
      self: num
from:
  self: from
  tables:
  - self: data
semicolon:
  self: ;",
        // implicit alias
        "\
self: select
columns:
- self: 2
  as:
    self: None
    alias:
      self: two
semicolon:
  self: ;",
        // join
        "\
self: select
columns:
- self: *
from:
  self: from
  tables:
  - self: data1
    as:
      self: as
      alias:
        self: one
  - self: data2
    as:
      self: None
      alias:
        self: two
    join:
      self: join
      on:
        self: ON
        expr:
          self: true
      type:
        self: inner
semicolon:
  self: ;",
        // parse_expr precedence
        "\
self: select
columns:
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
    columns:
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
from:
  self: from
  tables:
  - self: data
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
columns:
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
columns:
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
columns:
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
        // window clause
        "\
self: select
columns:
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
      partition:
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
      order:
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
      order:
        self: order
        by:
          self: by
        exprs:
        - self: b
          comma:
            self: ,
        - self: c
      partition:
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
        between:
          self: between
          and:
            self: and
        end:
          self: unbounded
          following:
            self: following
        start:
          self: unbounded
          preceding:
            self: preceding
      order:
        self: order
        by:
          self: by
        exprs:
        - self: b
          comma:
            self: ,
        - self: c
      partition:
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
        // raw, bytes
        "\
self: select
columns:
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
columns:
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
columns:
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
  tables:
  - self: .
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
columns:
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
columns:
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
  columns:
  - self: 1",
        // union
        "\
self: union
left:
  self: select
  columns:
  - self: 1
right:
  self: select
  columns:
  - self: 2
semicolon:
  self: ;",
        "\
self: union
left:
  self: (
  rparen:
    self: )
  stmt:
    self: select
    columns:
    - self: 1
right:
  self: select
  columns:
  - self: 2
semicolon:
  self: ;",
        "\
self: union
left:
  self: select
  columns:
  - self: 1
right:
  self: (
  rparen:
    self: )
  stmt:
    self: select
    columns:
    - self: 2
semicolon:
  self: ;",
        "\
self: union
left:
  self: union
  left:
    self: select
    columns:
    - self: 1
  right:
    self: select
    columns:
    - self: 2
right:
  self: select
  columns:
  - self: 3
semicolon:
  self: ;",
        "\
self: union
left:
  self: select
  columns:
  - self: 1
right:
  self: (
  rparen:
    self: )
  stmt:
    self: union
    left:
      self: select
      columns:
      - self: 2
    right:
      self: select
      columns:
      - self: 3
semicolon:
  self: ;",
        "\
self: union
left:
  self: (
  rparen:
    self: )
  stmt:
    self: union
    left:
      self: select
      columns:
      - self: 1
    right:
      self: select
      columns:
      - self: 2
right:
  self: select
  columns:
  - self: 3
semicolon:
  self: ;",
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
