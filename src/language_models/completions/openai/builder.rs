use std::collections::HashMap;

use super::{
    super::inference::{CompletionRequest, CompletionRequestBuilder},
    requests::OpenAiIoRequest,
};
use crate::language_models::completions::{
    error::{CompletionError, CompletionResult},
    functions::{FunctionParam, ParamType},
    ModelParameters,
};
use anyhow::anyhow;
use reqwest::header::HeaderMap;
use serde::{de::Error, Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tracing::info;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum OpenAiCompletionModel {
    #[default]
    Gpt3,
    Gpt4,
}

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-0125";
const GPT4_MODEL_STR: &str = "gpt-4-0125-preview";

impl OpenAiCompletionModel {
    fn serialize_function_params(params: HashMap<String, FunctionParam>) -> Value {
        let mut all_params = Map::new();
        let mut req = vec![];
        for (name, param) in params.iter() {
            let mut current_param = Map::new();
            match &param.typ {
                ParamType::String => {
                    current_param.insert("type".to_owned(), "string".to_owned().into());
                }
                ParamType::Bool => {
                    current_param.insert("type".to_owned(), "boolean".to_owned().into());
                }
                ParamType::Integer => {
                    current_param.insert("type".to_owned(), "integer".to_owned().into());
                }
                ParamType::Enum(variants) => {
                    current_param.insert("type".to_owned(), "string".to_owned().into());
                    current_param.insert("enum".to_owned(), json!(variants));
                }
            }

            if let Some(desc) = &param.description {
                current_param.insert("description".to_owned(), desc.to_owned().into());
            }
            all_params.insert(name.to_owned(), json!(current_param));
            if param.required {
                req.push(name);
            }
        }
        json!({
            "type": "object",
            "properties": all_params,
            "required": json!(req),
        })
    }
}

impl CompletionRequestBuilder for OpenAiCompletionModel {
    fn model_str(&self) -> &str {
        match self {
            Self::Gpt3 => GPT3_MODEL_STR,
            Self::Gpt4 => GPT4_MODEL_STR,
        }
    }

    fn url_str(&self) -> &str {
        "https://api.openai.com/v1/chat/completions"
    }

    fn headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        map.insert("Content-Type", "application/json".parse().unwrap());
        map
    }

    fn serialize_messages(&self, stack: &crate::agents::memory::MessageStack) -> Value {
        stack
            .as_ref()
            .to_owned()
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<Value>>()
            .into()
    }

    fn into_io_req(
        &self,
        stack: &crate::agents::memory::MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(OpenAiIoRequest::new(stack, params, *self, false)))
    }
    fn into_stream_req(
        &self,
        stack: &crate::agents::memory::MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(OpenAiIoRequest::new(stack, params, *self, true)))
    }
    fn serialize_function(
        &self,
        stack: &crate::prelude::MessageStack,
        function: crate::language_models::completions::functions::Function,
    ) -> CompletionResult<Value> {
        let mut func_map = Map::new();
        let params = Self::serialize_function_params(function.params);
        func_map.insert("name".to_owned(), function.name.clone().into());
        func_map.insert("description".to_owned(), function.description.into());
        func_map.insert("parameters".to_owned(), json!(params));
        let func: Value = func_map.into();
        info!("function serialized: {:?}", func);

        Ok(json!({
            "model": self.model_str(),
            "messages": self.serialize_messages(stack),
            "functions": [func],
            "function_call": {"name": function.name}
        }))
    }

    fn process_function_response(&self, response_json: Value) -> CompletionResult<Value> {
        let fn_call = response_json
            .get("choices")
            .ok_or(serde_json::Error::missing_field("choices"))?[0]
            .get("message")
            .ok_or(serde_json::Error::missing_field("message"))?
            .get("function_call")
            .ok_or(serde_json::Error::missing_field("function_call"))?;

        let args_json = serde_json::from_str::<Value>(
            fn_call
                .get("arguments")
                .ok_or(serde_json::Error::missing_field("arguments"))?
                .as_str()
                .expect("why did args fail to be coerced to str?"),
        )?;

        tracing::info!("Args json: {:?}", args_json);
        let mut args_output: Value = json!({});
        if let Some(arguments) = args_json.as_object() {
            for (key, value) in arguments.iter() {
                args_output
                    .as_object_mut()
                    .ok_or(CompletionError::from(anyhow!(
                        "failed to get args output as mutable array"
                    )))?
                    .insert(key.to_string(), value.clone());
            }
        }
        tracing::info!("Args output: {:?}", args_output);
        Ok(args_output)
    }
}

mod tests {
    use std::collections::HashMap;

    use once_cell::sync::Lazy;
    use serde_json::json;

    use crate::{
        language_models::completions::{
            functions::{FunctionParam, ParamType},
            openai::builder::OpenAiCompletionModel,
        },
        telemetry::{get_subscriber, init_subscriber},
    };

    #[test]
    fn correctly_serialize_params() {
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
                typ: ParamType::Enum(vec![String::from("celcius"), String::from("fahrenheight")]),
                description: None,
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

        let expected = json!({
                "type": "object",
                "properties": {
                  "location": {
                    "type": "string",
                    "description": "the city and state, e.g. san francisco, ca"
                  },
                    "num_days": {
                    "type": "integer",
                    "description": "the number of days to forcast",
                    },
                  "format": {
                    "type": "string",
                    "enum": ["celcius", "fahrenheight"]
                  }
                },
                "required": ["num_days", "format"]}
        );

        let serialized = OpenAiCompletionModel::serialize_function_params(params);

        for (k, v) in expected["properties"].as_object().unwrap().into_iter() {
            assert_eq!(v, &serialized["properties"][k])
        }
        for r in expected["required"].as_array().unwrap() {
            assert!(serialized["required"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| *v == r)
                .is_some())
        }
    }
}
