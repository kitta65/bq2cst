use super::*;

struct TestCase {
    code: String,
    expected_tokens: Vec<Token>,
    result_tokens: Vec<Token>,
}

impl TestCase {
    fn new(code: &str, expected_tokens_without_eof: Vec<Token>) -> TestCase {
        let code = code.to_string();
        let mut l = Lexer::new(code.clone());
        l.tokenize_code();
        let result_tokens = l.tokens;
        let mut expected_tokens = expected_tokens_without_eof;
        expected_tokens.push(Token::eof());
        TestCase {
            code,
            expected_tokens,
            result_tokens,
        }
    }
    fn test(&self) {
        println!("testing: \n{:?}\n", self.code);
        assert_eq!(self.expected_tokens.len(), self.result_tokens.len());
        for i in 0..self.expected_tokens.len() {
            assert_eq!(self.expected_tokens[i], self.result_tokens[i]);
        }
    }
}

#[test]
fn test_tokenize_code() {
    let test_cases = vec![
        TestCase::new(
            "\
SELECT c1 FROM t WHERE true GROUP BY 1 ORDER BY 1;",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "c1"),
                Token::from_str(1, 11, "FROM"),
                Token::from_str(1, 16, "t"),
                Token::from_str(1, 18, "WHERE"),
                Token::from_str(1, 24, "true"),
                Token::from_str(1, 29, "GROUP"),
                Token::from_str(1, 35, "BY"),
                Token::from_str(1, 38, "1"),
                Token::from_str(1, 40, "ORDER"),
                Token::from_str(1, 46, "BY"),
                Token::from_str(1, 49, "1"),
                Token::from_str(1, 50, ";"),
            ],
        ),
        // comment
        TestCase::new(
            "\
#standardSQL
SELECT 1 /*
  comment
*/
; -- comment",
            vec![
                Token::from_str(1, 1, "#standardSQL"),
                Token::from_str(2, 1, "SELECT"),
                Token::from_str(2, 8, "1"),
                Token::from_str(2, 10, "/*\n  comment\n*/"),
                Token::from_str(5, 1, ";"),
                Token::from_str(5, 3, "-- comment"),
            ],
        ),
        // string literal
        TestCase::new(
            "\
SELECT
  'xxx',
  r'xxx',
  \"xxx\",
  '''
xxx
  ''',
  \"\"\"
xxx
  \"\"\",",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(2, 3, "'xxx'"),
                Token::from_str(2, 8, ","),
                Token::from_str(3, 3, "r"),
                Token::from_str(3, 4, "'xxx'"),
                Token::from_str(3, 9, ","),
                Token::from_str(4, 3, "\"xxx\""),
                Token::from_str(4, 8, ","),
                Token::from_str(5, 3, "'''\nxxx\n  '''"),
                Token::from_str(7, 6, ","),
                Token::from_str(8, 3, "\"\"\"\nxxx\n  \"\"\""),
                Token::from_str(10, 6, ","),
            ],
        ),
        // numeric literal
        TestCase::new(
            "\
SELECT 1, 01, 1.1, .1, 1.1e+1, 1.1E-1, .1e10",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "1"),
                Token::from_str(1, 9, ","),
                Token::from_str(1, 11, "01"),
                Token::from_str(1, 13, ","),
                Token::from_str(1, 15, "1.1"),
                Token::from_str(1, 18, ","),
                Token::from_str(1, 20, ".1"),
                Token::from_str(1, 22, ","),
                Token::from_str(1, 24, "1.1e+1"),
                Token::from_str(1, 30, ","),
                Token::from_str(1, 32, "1.1E-1"),
                Token::from_str(1, 38, ","),
                Token::from_str(1, 40, ".1e10"),
            ],
        ),
        // timestamp, date literal
        TestCase::new(
            "\
SELECT date '2000-01-01', timestamp '2000-01-01'",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "date"),
                Token::from_str(1, 13, "'2000-01-01'"),
                Token::from_str(1, 25, ","),
                Token::from_str(1, 27, "timestamp"),
                Token::from_str(1, 37, "'2000-01-01'"),
            ],
        ),
        // array literal
        TestCase::new(
            "\
SELECT
  ARRAY<INT64>[1],
  ARRAY<STRUCT<INT64,INT64>>[(0,0)]",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(2, 3, "ARRAY"),
                Token::from_str(2, 8, "<"),
                Token::from_str(2, 9, "INT64"),
                Token::from_str(2, 14, ">"),
                Token::from_str(2, 15, "["),
                Token::from_str(2, 16, "1"),
                Token::from_str(2, 17, "]"),
                Token::from_str(2, 18, ","),
                Token::from_str(3, 3, "ARRAY"),
                Token::from_str(3, 8, "<"),
                Token::from_str(3, 9, "STRUCT"),
                Token::from_str(3, 15, "<"),
                Token::from_str(3, 16, "INT64"),
                Token::from_str(3, 21, ","),
                Token::from_str(3, 22, "INT64"),
                Token::from_str(3, 27, ">"),
                Token::from_str(3, 28, ">"),
                Token::from_str(3, 29, "["),
                Token::from_str(3, 30, "("),
                Token::from_str(3, 31, "0"),
                Token::from_str(3, 32, ","),
                Token::from_str(3, 33, "0"),
                Token::from_str(3, 34, ")"),
                Token::from_str(3, 35, "]"),
            ],
        ),
        // identifier
        TestCase::new(
            "\
SELECT _c1, `c-1`
FROM t",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "_c1"),
                Token::from_str(1, 11, ","),
                Token::from_str(1, 13, "`c-1`"),
                Token::from_str(2, 1, "FROM"),
                Token::from_str(2, 6, "t"),
            ],
        ),
        // parameter
        TestCase::new(
            "\
SELECT ?;
SELECT @param;",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "?"),
                Token::from_str(1, 9, ";"),
                Token::from_str(2, 1, "SELECT"),
                Token::from_str(2, 8, "@param"),
                Token::from_str(2, 14, ";"),
            ],
        ),
        // operator
        TestCase::new(
            "\
SELECT
  1-1+2/2*3,
  'a'||'b',
  2>>1",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(2, 3, "1"),
                Token::from_str(2, 4, "-"),
                Token::from_str(2, 5, "1"),
                Token::from_str(2, 6, "+"),
                Token::from_str(2, 7, "2"),
                Token::from_str(2, 8, "/"),
                Token::from_str(2, 9, "2"),
                Token::from_str(2, 10, "*"),
                Token::from_str(2, 11, "3"),
                Token::from_str(2, 12, ","),
                Token::from_str(3, 3, "'a'"),
                Token::from_str(3, 6, "||"),
                Token::from_str(3, 8, "'b'"),
                Token::from_str(3, 11, ","),
                Token::from_str(4, 3, "2"),
                Token::from_str(4, 4, ">>"),
                Token::from_str(4, 6, "1"),
            ],
        ),
        // function
        TestCase::new(
            "\
SELECT f(a1, a2)",
            vec![
                Token::from_str(1, 1, "SELECT"),
                Token::from_str(1, 8, "f"),
                Token::from_str(1, 9, "("),
                Token::from_str(1, 10, "a1"),
                Token::from_str(1, 12, ","),
                Token::from_str(1, 14, "a2"),
                Token::from_str(1, 16, ")"),
            ],
        ),
    ];
    for t in test_cases {
        t.test();
    }
}
