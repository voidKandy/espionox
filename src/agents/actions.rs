use super::{error::AgentResult, Agent};
use crate::language_models::openai::completions::{
    functions::CustomFunction, streaming::StreamedCompletionHandler,
};
use serde_json::Value;

pub async fn get_embedding(agent: &mut Agent, text: &str) -> AgentResult<Vec<f32>> {
    Ok(agent.completion_handler.get_embedding(text).await?)
}

pub async fn io_completion(agent: &mut Agent, _: ()) -> AgentResult<String> {
    Ok(agent
        .completion_handler
        .get_io_completion(&agent.cache)
        .await?)
}

pub async fn stream_completion(agent: &mut Agent, _: ()) -> AgentResult<StreamedCompletionHandler> {
    let cs = agent
        .completion_handler
        .get_stream_completion(&agent.cache)
        .await?;

    Ok(cs.into())
}

pub async fn function_completion(
    agent: &mut Agent,
    function: CustomFunction,
) -> AgentResult<Value> {
    Ok(agent
        .completion_handler
        .get_fn_completion(&agent.cache, function.function())
        .await?)
}
