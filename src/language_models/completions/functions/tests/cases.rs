use super::*;

pub struct TestCase {
    pub input: String,
    pub expected_tokens: Vec<Token>,
    pub expected_function: Function,
}

pub fn all_test_cases() -> Vec<TestCase> {
    vec![weather_func_test_case(), enough_context_func_test_case()]
}

fn weather_func_test_case() -> TestCase {
    let expected_tokens = vec![
        Token::new_identifier("get_n_day_weather_forecast"),
        Token::try_from(LP).unwrap(),
        Token::new_identifier("location"),
        Token::try_from(COLON).unwrap(),
        Token::try_from(STRING).unwrap(),
        Token::try_from(COMMA).unwrap(),
        Token::new_identifier("format"),
        Token::try_from(BANG).unwrap(),
        Token::try_from(COLON).unwrap(),
        Token::try_from(ENUM).unwrap(),
        Token::try_from(LP).unwrap(),
        Token::new_str_literal("celcius"),
        Token::try_from(PIPE).unwrap(),
        Token::new_str_literal("farenheight"),
        Token::try_from(RP).unwrap(),
        Token::try_from(COMMA).unwrap(),
        Token::new_identifier("num_days"),
        Token::try_from(BANG).unwrap(),
        Token::try_from(COLON).unwrap(),
        Token::try_from(INTEGER).unwrap(),
        Token::try_from(RP).unwrap(),
        Token::try_from(WHERE).unwrap(),
        Token::try_from(I).unwrap(),
        Token::try_from(AM).unwrap(),
        Token::new_str_literal("get an n-day weather forecast"),
        Token::new_identifier("location"),
        Token::try_from(IS).unwrap(),
        Token::new_str_literal("the city and state, e.g. san francisco, ca"),
        Token::new_identifier("format"),
        Token::try_from(IS).unwrap(),
        Token::new_str_literal("the temperature unit to use. infer this from the users location."),
        Token::new_identifier("num_days"),
        Token::try_from(IS).unwrap(),
        Token::new_str_literal("the number of days to forcast"),
        Token::end(),
    ];

    let mut params = HashMap::new();
    params.insert(
        String::from("location"),
        FunctionParam {
            required: false,
            typ: ParamType::String,
            description: Some("the city and state, e.g. san francisco, ca".to_owned()),
        },
    );
    params.insert(
        String::from("format"),
        FunctionParam {
            required: true,
            typ: ParamType::Enum(vec![String::from("celcius"), String::from("farenheight")]),
            description: Some(
                "the temperature unit to use. infer this from the users location.".to_owned(),
            ),
        },
    );
    params.insert(
        String::from("num_days"),
        FunctionParam {
            required: true,
            typ: ParamType::Integer,
            description: Some("the number of days to forcast".to_owned()),
        },
    );
    let expected_function = Function {
        name: "get_n_day_weather_forecast".to_owned(),
        description: "get an n-day weather forecast".to_owned(),
        params,
    };

    let input = r#"get_n_day_weather_forecast(location: string, format!: enum('celcius' | 'farenheight'), num_days!: integer)
        where 
            i am 'get an n-day weather forecast'
            location is 'the city and state, e.g. san francisco, ca'
            format is 'the temperature unit to use. infer this from the users location.'
            num_days is 'the number of days to forcast'
        "#.to_owned();

    TestCase {
        input,
        expected_tokens,
        expected_function,
    }
}

fn enough_context_func_test_case() -> TestCase {
    let expected_tokens = vec![
        Token::new_identifier("has_enough_context"),
        Token::try_from(LP).unwrap(),
        Token::new_identifier("context"),
        Token::try_from(BANG).unwrap(),
        Token::try_from(COLON).unwrap(),
        Token::try_from(STRING).unwrap(),
        Token::try_from(COMMA).unwrap(),
        Token::new_identifier("question"),
        Token::try_from(BANG).unwrap(),
        Token::try_from(COLON).unwrap(),
        Token::try_from(STRING).unwrap(),
        Token::try_from(COMMA).unwrap(),
        Token::new_identifier("enough"),
        Token::try_from(BANG).unwrap(),
        Token::try_from(COLON).unwrap(),
        Token::try_from(BOOL).unwrap(),
        Token::try_from(RP).unwrap(),
        Token::try_from(WHERE).unwrap(),
        Token::try_from(I).unwrap(),
        Token::try_from(AM).unwrap(),
        Token::new_str_literal(
            "tell if context given contains enough information to answer the question",
        ),
        Token::new_identifier("enough"),
        Token::try_from(IS).unwrap(),
        Token::new_str_literal(
            "true if context is enough, false otherwise. infer this from the context and question params",
        ),
        Token::end(),
    ];

    let mut params = HashMap::new();
    params.insert(
        String::from("context"),
        FunctionParam {
            required: true,
            typ: ParamType::String,
            description: None,
        },
    );
    params.insert(
        String::from("question"),
        FunctionParam {
            required: true,
            typ: ParamType::String,
            description: None,
        },
    );
    params.insert(
        String::from("enough"),
        FunctionParam {
            required: true,
            typ: ParamType::Bool,
            description: Some(
            "true if context is enough, false otherwise. infer this from the context and question params".to_owned(),
            ),
        },
    );
    let expected_function = Function {
        name: "has_enough_context".to_owned(),
        description: "tell if context given contains enough information to answer the question"
            .to_owned(),
        params,
    };

    let input = r#"has_enough_context(context!: string, question!: string, enough!: bool)
        where 
            i am 'tell if context given contains enough information to answer the question'
            enough is 'true if context is enough, false otherwise. infer this from the context and question params'
            "#
    .to_owned();

    TestCase {
        input,
        expected_tokens,
        expected_function,
    }
}
