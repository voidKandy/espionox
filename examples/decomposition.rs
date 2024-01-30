use futures_util::Future;
use std::pin::Pin;

use espionox::{
    environment::{
        agent::{
            language_models::LanguageModel,
            memory::{messages::MessageRole, Message},
        },
        dispatch::{Dispatch, EnvListener, EnvMessage, EnvRequest, ListenerMethodReturn},
        errors::DispatchError,
        Environment,
    },
    Agent,
};

#[derive(Debug)]
pub struct Decomposition {
    decomposed_text: Option<String>,
    watched_id: String,
    decomposer_id: String,
    times_decomposed: u32,
    max_decompositions: u32,
}

impl Decomposition {
    fn new(watched: &str, decomposer: &str, max_decompositions: u32) -> Self {
        Self {
            decomposed_text: None,
            watched_id: watched.to_string(),
            decomposer_id: decomposer.to_string(),
            times_decomposed: 0,
            max_decompositions,
        }
    }
}

impl EnvListener for Decomposition {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        if let EnvMessage::Request(req) = env_message {
            match req {
                EnvRequest::PushToCache { agent_id, message } => {
                    if message.role == MessageRole::User && agent_id == &self.watched_id {
                        return Some(env_message);
                    }
                }
                _ => return None,
            }
        }
        None
    }
    fn method<'l>(
        &'l mut self,
        trigger_message: &'l EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let agent = dispatch.get_agent_mut(&self.decomposer_id).unwrap();
            if let EnvMessage::Request(req) = trigger_message {
                if let EnvRequest::PushToCache { message, .. } = req {
                    agent.cache.push(message.clone());
                }
            }
            let counter: u32 = self.times_decomposed.clone().into();
            if counter >= self.max_decompositions {
                agent.cache.reset_to_system_prompt();
            }
            let model = agent.model.clone();
            let cache = agent.cache.clone();
            let client = &dispatch.client;
            let api_key = dispatch.api_key().unwrap();

            let decomp = model.io_completion_fn()(&client, &api_key, &(&cache).into(), &model)
                .await
                .unwrap();
            let decomp_agent = dispatch.get_agent_mut(&self.decomposer_id).unwrap();
            let text = decomp_agent.handle_completion_response(decomp).unwrap();
            decomp_agent.cache.push(Message::new_assistant(&text));
            self.decomposed_text = Some(text);
            Ok(())
        })
    }
    fn mutate<'l>(&'l mut self, origin: EnvMessage) -> EnvMessage {
        if let Some(text) = &self.decomposed_text {
            if let EnvMessage::Request(ref req) = origin {
                if let EnvRequest::PushToCache { agent_id, .. } = req {
                    let message = Message::new_user(&text);
                    self.times_decomposed += 1;
                    return EnvRequest::PushToCache {
                        agent_id: agent_id.to_string(),
                        message,
                    }
                    .into();
                }
            }
        }
        origin
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    let mut env = Environment::new(Some("testing"), Some(&api_key));
    let jerry = Agent::new("You are jerry!!", LanguageModel::default_gpt());
    let mut j_handle = env.insert_agent(Some("jerry"), jerry).await.unwrap();
    let decomp_agent = Agent::new(
        "You are watching the messages incoming to Jerry. \
        Jerry is a great guy, but he's not very smart. \
        Make sure anything being said to Jerry is brought\
        down to his level.",
        LanguageModel::default_gpt(),
    );

    let d_agent = env
        .insert_agent(Some("decomp"), decomp_agent)
        .await
        .unwrap();

    let decomp = Decomposition::new("jerry", "decomp", 5);
    env.insert_listener(decomp).await;
    env.spawn().await.unwrap();
    let message = Message::new_user("Can you explain an inverse square??");
    let ticket = j_handle.request_io_completion(message).await.unwrap();

    let noti = env
        .notifications
        .wait_for_notification(&ticket)
        .await
        .unwrap();
    println!("Noti: {:?}", noti);
    env.finalize_dispatch().await.unwrap();
    let stack = env.notifications.0.read().await;
    println!("\nStack: {:?}", stack);
    let dispatch = env.dispatch.read().await;
    let d_agent = dispatch.get_agent_ref(&d_agent.id).unwrap();
    println!("\nDecomp agent cache: {:?}", d_agent.cache);
}
