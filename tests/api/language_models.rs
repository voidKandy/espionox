use espionox::{
    agents::{actions::io_completion, Agent},
    language_models::completions::{
        openai::requests::{Choice, GptMessage, OpenAiResponse, OpenAiSuccess, OpenAiUsage},
        CompletionModel,
    },
};
use serde_json::json;

use crate::init_test;

#[test]
fn openai_response_parsed_correctly() {
    init_test();
    let value = json!(
    {
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1677858242,
        "model": "gpt-3.5-turbo-0613",
        "usage": {
            "prompt_tokens": 13,
            "completion_tokens": 7,
            "total_tokens": 20
        },
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "\n\nThis is a test!"
                },
                "logprobs": null,
                "finish_reason": "stop",
                "index": 0
            }
        ]
    });

    let res: OpenAiResponse = serde_json::from_value(value).unwrap();
    let expected = OpenAiResponse::Success(OpenAiSuccess {
        usage: OpenAiUsage {
            prompt_tokens: 13,
            completion_tokens: Some(7),
            total_tokens: 20,
        },
        choices: vec![{
            Choice {
                message: GptMessage {
                    role: "assistant".to_string(),
                    content: Some("\n\nThis is a test!".to_string()),
                    function_call: None,
                },
            }
        }],
    });
    assert_eq!(res, expected);
}

#[tokio::test]
async fn failed_request_does_not_overflow_stack() {
    init_test();
    let llm = CompletionModel::default_anthropic("invalid_key");
    let mut a = Agent::new(None, llm);

    let res = io_completion(&mut a, ()).await;

    println!("{:?}", res);
    assert!(res.is_ok());
}
