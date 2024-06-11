mod cases;
use super::{lexer::*, parser::*, tokens::*, *};
use crate::telemetry::{get_subscriber, init_subscriber};
use cases::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[test]
fn lexes_correctly() {
    Lazy::force(&TRACING);
    let cases = all_test_cases();
    for case in cases.iter() {
        let mut l = Lexer::new(&case.input);
        let lexed = l.lex_input().unwrap();
        for i in 0..case.expected_tokens.len() {
            assert_eq!(lexed[i], case.expected_tokens[i]);
        }
    }
}

#[test]
fn parse_parameters_works() {
    Lazy::force(&TRACING);
    let cases = all_test_cases();
    for case in cases.into_iter() {
        let input = format!(
            "({}",
            case.input
                .split_once("where")
                .unwrap()
                .0
                .split_once("(")
                .unwrap()
                .1
        );
        let mut lexer = Lexer::new(&input);
        let stream = lexer.lex_input().unwrap();
        let mut parser = Parser::try_from(stream).unwrap();
        let map = parser.parse_parameter_list().unwrap();
        let mut expected = case.expected_function.params;
        for (_, v) in expected.iter_mut() {
            v.description = None;
        }

        assert_eq!(expected, map);
    }
}

#[test]
fn parses_to_function_correctly() {
    Lazy::force(&TRACING);
    let cases = all_test_cases();
    for case in cases.into_iter() {
        let mut l = Lexer::new(&case.input);
        let stream = l.lex_input().unwrap();
        let mut parser = Parser::try_from(stream).unwrap();
        let function = parser.parse_whole_function().unwrap();

        let expected = case.expected_function;
        for (k, param) in expected.params.iter() {
            assert_eq!(function.params.get(k), Some(param));
        }
    }
}
