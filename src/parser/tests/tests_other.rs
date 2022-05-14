use super::*;

#[test]
fn test_parse_code_other() {
    let test_cases = vec![
        // ----- EXPORT statement -----
        Box::new(SuccessTestCase::new(
            "\
EXPORT DATA OPTIONS(
  uri = 'gs://bucket/folder/*.csv',
  format = 'CSV'
) AS SELECT 1;
",
            "\
self: EXPORT (ExportStatement)
as:
  self: AS (KeywordWithStatement)
  stmt:
    self: SELECT (SelectStatement)
    exprs:
    - self: 1 (NumericLiteral)
data:
  self: DATA (Keyword)
options:
  self: OPTIONS (KeywordWithGroupedXXX)
  group:
    self: ( (GroupedExprs)
    exprs:
    - self: = (BinaryOperator)
      comma:
        self: , (Symbol)
      left:
        self: uri (Identifier)
      right:
        self: 'gs://bucket/folder/*.csv' (StringLiteral)
    - self: = (BinaryOperator)
      left:
        self: format (Identifier)
      right:
        self: 'CSV' (StringLiteral)
    rparen:
      self: ) (Symbol)
semicolon:
  self: ; (Symbol)
",
            0,
        )),
        // ----- LOAD statement -----
        Box::new(SuccessTestCase::new(
            "\
LOAD DATA INTO `mydataset.tablename`
FROM FILES (
  uris = ['azure://sample.com/sample.parquet'],
  format = 'PARQUET'
)
WITH CONNECTION `dummy.connection`
",
            "\
self: LOAD (LoadStatement)
connection:
  self: CONNECTION (Keyword)
connection_name:
  self: `dummy.connection` (Identifier)
data:
  self: DATA (Keyword)
files:
  self: FILES (Keyword)
from:
  self: FROM (Keyword)
from_files:
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
      - self: 'azure://sample.com/sample.parquet' (StringLiteral)
      rparen:
        self: ] (Symbol)
  - self: = (BinaryOperator)
    left:
      self: format (Identifier)
    right:
      self: 'PARQUET' (StringLiteral)
  rparen:
    self: ) (Symbol)
ident:
  self: `mydataset.tablename` (Identifier)
into:
  self: INTO (Keyword)
with:
  self: WITH (Keyword)
",
            0,
        )),
        Box::new(SuccessTestCase::new(
            "\
LOAD DATA INTO `ident` (dt date, s STRING)
PARTITION BY dt
CLUSTER BY s
OPTIONS (dummy = 'dummy')
FROM FILES (dummy = 'dummy')
WITH CONNECTION `dummy.connection`
",
            "\
self: LOAD (LoadStatement)
clusterby:
  self: CLUSTER (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: s (Identifier)
column_group:
  self: ( (GroupedTypeDeclarations)
  declarations:
  - self: dt (TypeDeclaration)
    comma:
      self: , (Symbol)
    type:
      self: date (Type)
  - self: s (TypeDeclaration)
    type:
      self: STRING (Type)
  rparen:
    self: ) (Symbol)
connection:
  self: CONNECTION (Keyword)
connection_name:
  self: `dummy.connection` (Identifier)
data:
  self: DATA (Keyword)
files:
  self: FILES (Keyword)
from:
  self: FROM (Keyword)
from_files:
  self: ( (GroupedExprs)
  exprs:
  - self: = (BinaryOperator)
    left:
      self: dummy (Identifier)
    right:
      self: 'dummy' (StringLiteral)
  rparen:
    self: ) (Symbol)
ident:
  self: `ident` (Identifier)
into:
  self: INTO (Keyword)
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
partitionby:
  self: PARTITION (XXXByExprs)
  by:
    self: BY (Keyword)
  exprs:
  - self: dt (Identifier)
with:
  self: WITH (Keyword)
",
            0,
        )),
    ];
    for t in test_cases {
        t.test();
    }
}
