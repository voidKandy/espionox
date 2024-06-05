use super::{
    super::{
        error::CompletionResult,
        inference::{CompletionRequest, CompletionRequestBuilder},
        ModelParameters,
    },
    requests::AnthropicIoRequest,
};
use crate::agents::memory::{Message, MessageStack};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum AnthropicCompletionModel {
    #[default]
    Opus,
    Sonnet,
    Haiku,
}

const OPUS_MODEL_STR: &str = "claude-3-opus-20240229";
const SONNET_MODEL_STR: &str = "claude-3-sonnet-20240229";
const HAIKU_MODEL_STR: &str = "claude-3-haiku-20240307";

impl CompletionRequestBuilder for AnthropicCompletionModel {
    fn model_str(&self) -> &str {
        match self {
            Self::Opus => OPUS_MODEL_STR,
            Self::Sonnet => SONNET_MODEL_STR,
            Self::Haiku => HAIKU_MODEL_STR,
        }
    }

    fn url_str(&self) -> &str {
        "https://api.anthropic.com/v1/messages"
    }

    fn headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("x-api-key", format!("{}", api_key).parse().unwrap());
        map.insert("anthropic-version", "2023-06-01".parse().unwrap());
        map.insert("content-type", "application/json".parse().unwrap());
        map
    }

    fn serialize_messages(&self, stack: &MessageStack) -> Value {
        // Anthropic model requires that messages alternate from User to assistant. So we'll
        // concatenate all adjacent messages to one
        let mut val_vec: Vec<Value> = vec![];
        let mut last_message: Option<Message> = None;
        for message in stack.clone().into_iter() {
            match last_message.take() {
                Some(mut m) => {
                    if message.role.to_string() == m.role.to_string() {
                        m.content = format!("{}. {}", m.content, message.content);
                        last_message = Some(m);
                    } else {
                        let val: Value = m.into();
                        val_vec.push(val);
                        last_message = Some(message);
                    }
                }
                None => last_message = Some(message),
            }
        }
        if let Some(m) = last_message {
            let val: Value = m.into();
            val_vec.push(val);
        }
        val_vec.into()
    }

    fn into_io_req(
        &self,
        stack: &MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(AnthropicIoRequest::new(
            stack, params, *self, false,
        )))
    }

    fn into_stream_req(
        &self,
        stack: &MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(AnthropicIoRequest::new(
            stack, params, *self, true,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::memory::OtherRoleTo;
    #[test]
    fn anthropic_agent_cache_to_json() {
        let mut stack = MessageStack::new("SYSTEM");
        stack.push(Message::new_user("USER"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_assistant("ASS"));
        stack.push(Message::new_assistant("ASS1"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_other("some_other", "USE2", OtherRoleTo::User));
        stack.push(Message::new_other(
            "some_other",
            "ASS",
            OtherRoleTo::Assistant,
        ));
        let handler = AnthropicCompletionModel::default();
        let vals = handler.serialize_messages(&stack);
        println!("VALS: {:?}", vals);
        let stack: MessageStack =
            MessageStack::try_from(vals.as_array().unwrap().to_owned()).unwrap();
        assert_eq!(5, stack.len());
    }
}
