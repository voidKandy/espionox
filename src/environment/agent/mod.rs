pub mod language_models;
pub mod memory;
pub mod utils;
use memory::{Message, MessageVector};
use uuid::Uuid;

pub use super::errors::AgentError;
use anyhow::anyhow;
use language_models::LanguageModel;
use serde_json::Value;

use crate::environment::{EnvMessageSender, EnvRequest};

use self::language_models::openai::gpt::GptResponse;

/// Agent struct for interracting with LLM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    /// Unique Identifier
    /// Memory contains cache and ltm
    pub cache: MessageVector,
    /// Language model defines which model to use for the given agent
    pub model: LanguageModel,
}

/// Handle for making requests to agents within Environment
#[derive(Debug, Clone)]
pub struct AgentHandle {
    /// Associated Agent's ID
    pub id: String,
    /// Connection to environment
    pub sender: EnvMessageSender,
}

impl From<(&str, EnvMessageSender)> for AgentHandle {
    fn from((id, sender): (&str, EnvMessageSender)) -> Self {
        Self {
            id: id.to_string(),
            sender,
        }
    }
}

impl Default for Agent {
    fn default() -> Self {
        let cache = crate::persistance::prompts::get_prompt_by_name("DEFAULT_INIT_PROMPT")
            .unwrap_or(MessageVector::init());
        tracing::info!("Default Agent initialized with cache: {:?}", cache);
        let model = LanguageModel::default_gpt();
        Agent { cache, model }
    }
}

impl Agent {
    pub fn new(init_prompt: &str, model: LanguageModel) -> Self {
        let cache = MessageVector::new(init_prompt);
        Agent {
            cache,
            model,
            ..Default::default()
        }
    }

    #[tracing::instrument(name = "Parse GptResponse and add token count")]
    pub fn handle_completion_response(
        &mut self,
        response: GptResponse,
    ) -> Result<String, AgentError> {
        let gpt = self.model.inner_mut_gpt().unwrap();
        gpt.token_count += response.usage.total_tokens;

        tracing::info!(
            "{} tokens added to model token count. Total count: {}",
            response.usage.total_tokens,
            gpt.token_count
        );

        let parsed_response = response
            .parse()
            .map_err(|err| AgentError::Undefined(anyhow!("Error parsing Gpt Reponse: {err:?}")))?;

        Ok(parsed_response)
    }
}

impl AgentHandle {
    /// Requests env for response, returns ticket number
    #[tracing::instrument(name = "Send request to prompt agent to env", skip(self))]
    pub async fn request_io_completion(&mut self, message: Message) -> Result<Uuid, AgentError> {
        let ticket = Uuid::new_v4();
        let env_req = EnvRequest::PromptAgent {
            ticket,
            agent_id: self.id.to_owned(),
            message,
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(env_req.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        tracing::info!("Requested an oi completion from the env");
        Ok(ticket)
    }

    /// Requests env for streamed response, returns ticket number
    #[tracing::instrument(name = "Send request for a stream handle to env", skip(self))]
    pub async fn request_stream_completion(
        &mut self,
        message: Message,
    ) -> Result<Uuid, AgentError> {
        let ticket = Uuid::new_v4();
        let env_req = EnvRequest::StreamPromptAgent {
            ticket,
            agent_id: self.id.to_owned(),
            message,
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(env_req.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        tracing::info!("Requested a stream completion from the env");
        Ok(ticket)
    }

    // Openai function calling completion
    // #[tracing::instrument(name = "Function prompt GPT API for response" skip(message, custom_function))]
    // pub async fn function_prompt(
    //     &mut self,
    //     custom_function: CustomFunction,
    //     message: Message,
    // ) -> Result<Value, AgentError> {
    //     self.cache.as_mut().push(message);
    //     let func = custom_function.function();
    //     let gpt = &self.model.inner_gpt().unwrap();
    //     let json_payload: Vec<Value> = (&self.cache).into();
    //     let function_response = gpt
    //         .function_completion(&json_payload, &func)
    //         .await
    //         .map_err(|err| AgentError::GptError(err))?;
    //     tracing::info!("Function response: {:?}", function_response);
    //     Ok(function_response
    //         .parse_fn()
    //         .expect("failed to parse response"))
    // }

    // Get streamed completion, this function returns a reciever which must be tried to recieve
    // tokens
    // #[tracing::instrument(name = "Prompt agent for stream response")]
    // pub async fn stream_prompt(
    //     &mut self,
    //     message: Message,
    // ) -> Result<CompletionReceiverHandler, AgentError> {
    //     self.cache.as_mut().push(message);
    //     let gpt = &self.model.inner_gpt().unwrap();
    //     let json_payload: Vec<Value> = (&self.cache).into();
    //     let response_stream = gpt.stream_completion(&json_payload).await?;
    //
    //     let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
    //         tokio::sync::mpsc::channel(50);
    //     CompletionStreamingThread::spawn_poll_stream_for_tokens(response_stream, tx);
    //     Ok(CompletionReceiverHandler::from(rx))
    // }

    //     #[cfg(feature = "long_term_memory")]
    //     pub fn vector_query_files(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
    //         match &self.memory.long_term {
    //             LongTermMemory::Some(mem) => {
    //                 let query_vector = embed(query).expect("Failed to embed query");
    //                 Some(File::get_from_embedding(query_vector.into(), &mem.pool()))
    //             }
    //             _ => None,
    //         }
    //     }
    //
    //     #[cfg(feature = "long_term_memory")]
    //     pub fn vector_query_chunks(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
    //         match &self.context.long_term {
    //             LongTermMemory::Some(mem) => {
    //                 let query_vector = embed(query).expect("Failed to embed query");
    //                 Some(FileChunk::get_from_embedding(
    //                     query_vector.into(),
    //                     &mem.pool(),
    //                 ))
    //             }
    //             _ => None,
    //         }
    //     }
}
