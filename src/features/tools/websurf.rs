use headless_chrome::{
    self,
    protocol::cdp::{Page::CaptureScreenshotFormatOption, Target::CreateTarget},
    Browser, LaunchOptions,
};
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};

use crate::{
    agents::{
        language_models::{
            openai::{
                functions::{CustomFunction, Property, PropertyInfo},
                gpt::{Gpt, GptModel},
            },
            LanguageModel,
        },
        memory::{Message, MessageRole, MessageStack},
    },
    environment::{
        agent_handle::AgentHandle,
        dispatch::{listeners::ListenerMethodReturn, EnvListener, EnvMessage, EnvRequest},
        ListenerError,
    },
};

use super::vision::{message_vector_to_context_with_image, vision_completion};
use std::fmt;

#[derive(Debug, Deserialize)]
struct SurferFunctionOutput {
    requires_browse: bool,
    url: String,
}

#[derive(Debug)]
struct SurferListener {
    agent_id: String,
    client: reqwest::Client,
    fn_out: Option<SurferFunctionOutput>,
}

pub struct Surfer {
    browser: headless_chrome::browser::Browser,
    current_screenshot: Option<Vec<u8>>,
    listener: SurferListener,
}

impl fmt::Debug for Surfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Surfer")
            .field("current_screenshot", &self.current_screenshot)
            .finish()
    }
}

impl From<&AgentHandle> for Surfer {
    fn from(handle: &AgentHandle) -> Self {
        let options = LaunchOptions::default();
        let browser = Browser::new(options).unwrap();
        Self {
            browser,
            current_screenshot: None,
            listener: SurferListener {
                agent_id: handle.id.clone(),
                client: Client::new(),
                fn_out: None,
            },
        }
    }
}

impl SurferListener {
    fn discern_browse_request() -> CustomFunction {
        let url_info = PropertyInfo::new(
            "url",
            json!("A URL either provided by or implied by the user, must be a valid url"),
        );
        let url_prop = Property::build_from("url")
            .return_type("string")
            .add_info(url_info)
            .finished();

        let browse_info =PropertyInfo::new("requires_browse", json!("True if the user is asking for information that requires the model to use it's browsing capabilities"));
        let request_to_browse_prop = Property::build_from("requires_browse")
            .return_type("boolean")
            .add_info(browse_info)
            .finished();

        CustomFunction::build_from("discern_browse_request")
            .description(
                "Discern whether the given body of text is asking for the model to get information that requires it to go somewhere on the internet",
            )
            .add_property(request_to_browse_prop, true)
            .add_property(url_prop, true)
            .finished()
    }
}

impl Surfer {
    pub fn get_screenshot(&mut self, url: &str) -> Result<(), anyhow::Error> {
        let tab = self.browser.new_tab_with_options(CreateTarget {
            url: url.to_string(),
            width: Some(720),
            height: Some(400),
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: None,
            background: None,
        })?;
        let png_data =
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)?;
        self.current_screenshot = Some(png_data);
        tab.close_with_unload().unwrap();
        Ok(())
    }

    pub async fn description_of_current_screenshot(
        &self,
        api_key: &str,
    ) -> Result<String, anyhow::Error> {
        let mut messages = MessageStack::new(
            "Your job is to give detailed descriptions of webpages based on screenshots",
        );
        messages.push(Message::new_user("Describe this webpage"));

        let screenshot = self.current_screenshot.clone().unwrap();
        let context = message_vector_to_context_with_image(&mut messages, None, Some(screenshot));
        let client = reqwest::Client::new();
        //// BAD!!!
        // let api_key = std::env::var("TESTING_API_KEY").unwrap();
        let gpt = Gpt::new(GptModel::Gpt4, 0.4);
        let model = LanguageModel::OpenAi(gpt);

        let response = vision_completion(&client, api_key, &context, &model).await?;
        response.parse()
    }
}

impl EnvListener for Surfer {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        self.listener.trigger(env_message)
    }
    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut crate::environment::dispatch::Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let trigger_message = self.listener.method(trigger_message, dispatch).await?;
            let trigger_message: EnvRequest = trigger_message.try_into().unwrap();
            tracing::info!("Within Surfer method");
            if let Some(fn_out) = &self.listener.fn_out.take() {
                if let EnvRequest::PushToCache { message, agent_id } = trigger_message {
                    tracing::info!("Surfer is surfing");
                    self.get_screenshot(&fn_out.url)
                        .map_err(|e| ListenerError::Undefined(e.into()))?;
                    let screenshot_desc = self
                        .description_of_current_screenshot(&dispatch.api_key().map_err(|_| {
                            ListenerError::Other("NO API KEY IN DISPATCH".to_owned())
                        })?)
                        .await
                        .map_err(|e| ListenerError::Undefined(e.into()))?;
                    tracing::info!("Surfer got screenshot, desc: {}", screenshot_desc);
                    let message_to_replace = Message::new_system(&format!(
                        "CONTEXT: {}QUERY: {}",
                        screenshot_desc, message.content
                    ));
                    let req_to_replace = EnvRequest::PushToCache {
                        agent_id,
                        message: message_to_replace,
                    };
                    return Ok(req_to_replace.into());
                }
                unreachable!()
            }

            tracing::warn!("Surfer listener found no need to use browser");
            return Ok(trigger_message.into());
        })
    }
}

impl EnvListener for SurferListener {
    fn trigger<'l>(
        &self,
        env_message: &'l crate::environment::dispatch::EnvMessage,
    ) -> Option<&'l crate::environment::dispatch::EnvMessage> {
        if let EnvMessage::Request(req) = env_message {
            if let EnvRequest::PushToCache { agent_id, message } = req {
                if agent_id == &self.agent_id && message.role == MessageRole::User {
                    tracing::info!("Surfer listener should trigger");
                    return Some(env_message);
                }
            }
        }
        return None;
    }
    fn method<'l>(
        &'l mut self,
        trigger_message: crate::environment::dispatch::EnvMessage,
        dispatch: &'l mut crate::environment::dispatch::Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let req: EnvRequest = trigger_message.try_into().unwrap();
            if let EnvRequest::PushToCache { message, agent_id } = req {
                let function = Self::discern_browse_request();
                let model = LanguageModel::default_gpt();
                let user_mes_val: Value = message.clone().into();
                let api_key = dispatch
                    .api_key()
                    .map_err(|e| ListenerError::Undefined(e.into()))?;
                let response = model.function_completion_fn()(
                    &self.client,
                    &api_key,
                    &vec![user_mes_val],
                    &model,
                    &function.function(),
                )
                .await
                .map_err(|e| ListenerError::Undefined(e.into()))?;
                let fn_res = response
                    .parse_fn()
                    .map_err(|e| ListenerError::Undefined(e.into()))?;
                let fn_res = serde_json::from_value::<SurferFunctionOutput>(fn_res)
                    .map_err(|e| ListenerError::Undefined(e.into()))?;
                tracing::info!("Function response: {:?}", fn_res);
                if fn_res.requires_browse {
                    self.fn_out = Some(fn_res);
                    let return_message = EnvRequest::PushToCache { agent_id, message };
                    return Ok(return_message.into());
                }
            }
            Err(ListenerError::Other("No recent user message".to_string()))
        })
    }
}
