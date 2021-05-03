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
        expected_tokens.push(Token::new(usize::MAX, usize::MAX, ""));
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
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "c1"),
                Token::new(1, 11, "FROM"),
                Token::new(1, 16, "t"),
                Token::new(1, 18, "WHERE"),
                Token::new(1, 24, "true"),
                Token::new(1, 29, "GROUP"),
                Token::new(1, 35, "BY"),
                Token::new(1, 38, "1"),
                Token::new(1, 40, "ORDER"),
                Token::new(1, 46, "BY"),
                Token::new(1, 49, "1"),
                Token::new(1, 50, ";"),
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
                Token::new(1, 1, "#standardSQL"),
                Token::new(2, 1, "SELECT"),
                Token::new(2, 8, "1"),
                Token::new(2, 10, "/*\n  comment\n*/"),
                Token::new(5, 1, ";"),
                Token::new(5, 3, "-- comment"),
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
                Token::new(1, 1, "SELECT"),
                Token::new(2, 3, "'xxx'"),
                Token::new(2, 8, ","),
                Token::new(3, 3, "r"),
                Token::new(3, 4, "'xxx'"),
                Token::new(3, 9, ","),
                Token::new(4, 3, "\"xxx\""),
                Token::new(4, 8, ","),
                Token::new(5, 3, "'''\nxxx\n  '''"),
                Token::new(7, 6, ","),
                Token::new(8, 3, "\"\"\"\nxxx\n  \"\"\""),
                Token::new(10, 6, ","),
            ],
        ),
        // numeric literal
        TestCase::new(
            "\
SELECT 1, 01, 1.1, .1, 1.1e+1, 1.1E-1, .1e10",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "1"),
                Token::new(1, 9, ","),
                Token::new(1, 11, "01"),
                Token::new(1, 13, ","),
                Token::new(1, 15, "1.1"),
                Token::new(1, 18, ","),
                Token::new(1, 20, ".1"),
                Token::new(1, 22, ","),
                Token::new(1, 24, "1.1e+1"),
                Token::new(1, 30, ","),
                Token::new(1, 32, "1.1E-1"),
                Token::new(1, 38, ","),
                Token::new(1, 40, ".1e10"),
            ],
        ),
        // timestamp, date literal
        TestCase::new(
            "\
SELECT date '2000-01-01', timestamp '2000-01-01'",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "date"),
                Token::new(1, 13, "'2000-01-01'"),
                Token::new(1, 25, ","),
                Token::new(1, 27, "timestamp"),
                Token::new(1, 37, "'2000-01-01'"),
            ],
        ),
        // array literal
        TestCase::new(
            "\
SELECT
  ARRAY<INT64>[1],
  ARRAY<STRUCT<INT64,INT64>>[(0,0)]",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(2, 3, "ARRAY"),
                Token::new(2, 8, "<"),
                Token::new(2, 9, "INT64"),
                Token::new(2, 14, ">"),
                Token::new(2, 15, "["),
                Token::new(2, 16, "1"),
                Token::new(2, 17, "]"),
                Token::new(2, 18, ","),
                Token::new(3, 3, "ARRAY"),
                Token::new(3, 8, "<"),
                Token::new(3, 9, "STRUCT"),
                Token::new(3, 15, "<"),
                Token::new(3, 16, "INT64"),
                Token::new(3, 21, ","),
                Token::new(3, 22, "INT64"),
                Token::new(3, 27, ">"),
                Token::new(3, 28, ">"),
                Token::new(3, 29, "["),
                Token::new(3, 30, "("),
                Token::new(3, 31, "0"),
                Token::new(3, 32, ","),
                Token::new(3, 33, "0"),
                Token::new(3, 34, ")"),
                Token::new(3, 35, "]"),
            ],
        ),
        // identifier
        TestCase::new(
            "\
SELECT _c1, `c-1`
FROM t",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "_c1"),
                Token::new(1, 11, ","),
                Token::new(1, 13, "`c-1`"),
                Token::new(2, 1, "FROM"),
                Token::new(2, 6, "t"),
            ],
        ),
        // parameter
        TestCase::new(
            "\
SELECT ?;
SELECT @param;",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "?"),
                Token::new(1, 9, ";"),
                Token::new(2, 1, "SELECT"),
                Token::new(2, 8, "@param"),
                Token::new(2, 14, ";"),
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
                Token::new(1, 1, "SELECT"),
                Token::new(2, 3, "1"),
                Token::new(2, 4, "-"),
                Token::new(2, 5, "1"),
                Token::new(2, 6, "+"),
                Token::new(2, 7, "2"),
                Token::new(2, 8, "/"),
                Token::new(2, 9, "2"),
                Token::new(2, 10, "*"),
                Token::new(2, 11, "3"),
                Token::new(2, 12, ","),
                Token::new(3, 3, "'a'"),
                Token::new(3, 6, "||"),
                Token::new(3, 8, "'b'"),
                Token::new(3, 11, ","),
                Token::new(4, 3, "2"),
                Token::new(4, 4, ">>"),
                Token::new(4, 6, "1"),
            ],
        ),
        // function
        TestCase::new(
            "\
SELECT f(a1, a2)",
            vec![
                Token::new(1, 1, "SELECT"),
                Token::new(1, 8, "f"),
                Token::new(1, 9, "("),
                Token::new(1, 10, "a1"),
                Token::new(1, 12, ","),
                Token::new(1, 14, "a2"),
                Token::new(1, 16, ")"),
            ],
        ),
    ];
    for t in test_cases {
        t.test();
    }
}
