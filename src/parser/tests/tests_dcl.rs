use super::*;

#[test]
fn test_parse_code_other() {
    let test_cases = vec![
        // ----- GRANT statement -----
        TestCase::new(
            "\
GRANT `roles/bigquery.dataViewer`, `roles/bigquery.admin`
ON SCHEMA project_name.dataset_name
TO 'user:foo@example.com', 'user:bar@example.com'
",
            "\
self: GRANT (GrantStatement)
ident:
  self: . (DotOperator)
  left:
    self: project_name (Identifier)
  right:
    self: dataset_name (Identifier)
on:
  self: ON (Keyword)
resource_type:
  self: SCHEMA (Keyword)
roles:
- self: `roles/bigquery.dataViewer` (Identifier)
  comma:
    self: , (Symbol)
- self: `roles/bigquery.admin` (Identifier)
to:
  self: TO (KeywordWithExprs)
  exprs:
  - self: 'user:foo@example.com' (StringLiteral)
    comma:
      self: , (Symbol)
  - self: 'user:bar@example.com' (StringLiteral)
",
        ),
        // ----- REVOKE statement -----
        TestCase::new(
            "\
REVOKE `roles/bigquery.admin`
ON SCHEMA dataset_name
FROM 'user:foo@example.com', 'user:bar@example.com'
;
",
            "\
self: REVOKE (RevokeStatement)
from:
  self: FROM (KeywordWithExprs)
  exprs:
  - self: 'user:foo@example.com' (StringLiteral)
    comma:
      self: , (Symbol)
  - self: 'user:bar@example.com' (StringLiteral)
ident:
  self: dataset_name (Identifier)
on:
  self: ON (Keyword)
resource_type:
  self: SCHEMA (Keyword)
roles:
- self: `roles/bigquery.admin` (Identifier)
semicolon:
  self: ; (Symbol)
",
        ),
        // ----- Reservations statement -----
        // CREATE
        TestCase::new(
            "\
CREATE CAPACITY project.region.commitment_id
AS JSON '''
  'slot_count': 100,
  'plan': 'FLEX'
'''
",
            "\
self: CREATE (CreateReservationStatement)
as_json:
- self: AS (Keyword)
- self: JSON (Keyword)
ident:
  self: . (DotOperator)
  left:
    self: . (DotOperator)
    left:
      self: project (Identifier)
    right:
      self: region (Identifier)
  right:
    self: commitment_id (Identifier)
json:
  self: '''
  'slot_count': 100,
  'plan': 'FLEX'
''' (StringLiteral)
what:
  self: CAPACITY (Keyword)
",
        ),
        // DELETE
        TestCase::new(
            "\
DROP ASSIGNMENT IF EXISTS project.location.reservation.assignment
",
            "\
self: DROP (DropStatement)
ident:
  self: . (DotOperator)
  left:
    self: . (DotOperator)
    left:
      self: . (DotOperator)
      left:
        self: project (Identifier)
      right:
        self: location (Identifier)
    right:
      self: reservation (Identifier)
  right:
    self: assignment (Identifier)
if_exists:
- self: IF (Keyword)
- self: EXISTS (Keyword)
what:
  self: ASSIGNMENT (Keyword)
",
        ),
    ];
    for t in test_cases {
        t.test(0);
    }
}
