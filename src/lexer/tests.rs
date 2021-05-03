use super::*;

//#[test]
//fn test_tokenize_code_old() {
//    let input = "#standardSQL
//SELECT 10, 1.1, 'aaa' || \"bbb\", .9, 1-1+2/2*3, date '2000-01-01', timestamp '2000-01-01',col1,date_add(col1, interval 9 hour),.1E4,?,@@param,'''abc''',arr[offset(1)],ARRAY<INT64>[1],
//From `data`; -- comment
//-- 
///*
//e
//o
//f
//*/select '\\'','''\\'''',\"\\x00\"".to_string();
//    let mut l = Lexer::new(input);
//    let tokens = l.tokenize_code();
//    let expected_tokens: Vec<Token> = vec![
//        // line 0
//        Token {
//            line: 0,
//            column: 0,
//            literal: "#standardSQL".to_string(),
//        },
//        // line 1
//        Token {
//            line: 1,
//            column: 0,
//            literal: "SELECT".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 7,
//            literal: "10".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 9,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 11,
//            literal: "1.1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 14,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 16,
//            literal: "'aaa'".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 22,
//            literal: "||".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 25,
//            literal: "\"bbb\"".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 30,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 32,
//            literal: ".9".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 34,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 36,
//            literal: "1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 37,
//            literal: "-".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 38,
//            literal: "1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 39,
//            literal: "+".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 40,
//            literal: "2".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 41,
//            literal: "/".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 42,
//            literal: "2".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 43,
//            literal: "*".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 44,
//            literal: "3".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 45,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 47,
//            literal: "date".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 52,
//            literal: "'2000-01-01'".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 64,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 66,
//            literal: "timestamp".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 76,
//            literal: "'2000-01-01'".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 88,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 89,
//            literal: "col1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 93,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 94,
//            literal: "date_add".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 102,
//            literal: "(".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 103,
//            literal: "col1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 107,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 109,
//            literal: "interval".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 118,
//            literal: "9".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 120,
//            literal: "hour".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 124,
//            literal: ")".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 125,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 126,
//            literal: ".1E4".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 130,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 131,
//            literal: "?".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 132,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 133,
//            literal: "@@param".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 140,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 141,
//            literal: "'''abc'''".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 150,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 151,
//            literal: "arr".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 154,
//            literal: "[".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 155,
//            literal: "offset".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 161,
//            literal: "(".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 162,
//            literal: "1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 163,
//            literal: ")".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 164,
//            literal: "]".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 165,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 166,
//            literal: "ARRAY".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 171,
//            literal: "<".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 172,
//            literal: "INT64".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 177,
//            literal: ">".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 178,
//            literal: "[".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 179,
//            literal: "1".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 180,
//            literal: "]".to_string(),
//        },
//        Token {
//            line: 1,
//            column: 181,
//            literal: ",".to_string(),
//        },
//        // line2
//        Token {
//            line: 2,
//            column: 0,
//            literal: "From".to_string(),
//        },
//        Token {
//            line: 2,
//            column: 5,
//            literal: "`data`".to_string(),
//        },
//        Token {
//            line: 2,
//            column: 11,
//            literal: ";".to_string(),
//        },
//        Token {
//            line: 2,
//            column: 13,
//            literal: "-- comment".to_string(),
//        },
//        // line3
//        Token {
//            line: 3,
//            column: 0,
//            literal: "-- ".to_string(),
//        },
//        // line4
//        Token {
//            line: 4,
//            column: 0,
//            literal: "/*
//e
//o
//f
//*/"
//            .to_string(),
//        },
//        Token {
//            line: 8,
//            column: 2,
//            literal: "select".to_string(),
//        },
//        Token {
//            line: 8,
//            column: 9,
//            literal: "'\\''".to_string(),
//        },
//        Token {
//            line: 8,
//            column: 13,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 8,
//            column: 14,
//            literal: "'''\\''''".to_string(),
//        },
//        Token {
//            line: 8,
//            column: 22,
//            literal: ",".to_string(),
//        },
//        Token {
//            line: 8,
//            column: 23,
//            literal: "\"\\x00\"".to_string(),
//        },
//        Token::new(usize::MAX, usize::MAX, ""),
//    ];
//    for (i, t) in expected_tokens.iter().enumerate() {
//        assert_eq!(tokens[i], *t);
//    }
//    assert_eq!(tokens.len(), expected_tokens.len());
//}

struct TestCase {
    result_tokens: Vec<Token>,
    expected_tokens: Vec<Token>,
}

impl TestCase {
    fn new(code: &str, expected_tokens_without_eof: Vec<Token>) -> TestCase {
        let mut l = Lexer::new(code.to_string());
        l.tokenize_code();
        let result_tokens = l.tokens;
        let mut expected_tokens = expected_tokens_without_eof;
        expected_tokens.push(Token::new(usize::MAX,usize::MAX,""));
        TestCase {
            result_tokens,
            expected_tokens,
        }
    }
    fn test(&self) {
        assert_eq!(self.result_tokens.len(), self.expected_tokens.len());
        for i in 0..self.result_tokens.len() {
            assert_eq!(self.result_tokens[i], self.expected_tokens[i]);
        }

    }
}

#[test]
fn test_tokenize_code() {
    let test_cases = vec![TestCase::new(
        "\
select 1;",
        vec![
            Token::new(1, 1, "select"),
            Token::new(1, 8, "1"),
            Token::new(1, 9, ";"),
        ],
    )];
    for t in test_cases {
        t.test();
    }
}
