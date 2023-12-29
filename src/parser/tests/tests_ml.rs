use super::*;

#[test]
fn test_parse_code_ml() {
    let test_cases = vec![Box::new(SuccessTestCase::new(
        "\
select 1
",
        "\
self: select (SelectStatement)
exprs:
- self: 1 (NumericLiteral)
",
        0,
    ))];
    for t in test_cases {
        t.test();
    }
}
