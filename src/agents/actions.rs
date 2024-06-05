use super::{error::AgentResult, Agent};
use crate::language_models::completions::streaming::ProviderStreamHandler;
use serde_json::Value;

pub async fn io_completion(agent: &mut Agent, _: ()) -> AgentResult<String> {
    Ok(agent
        .completion_model
        .get_io_completion(&agent.cache)
        .await?)
}

pub async fn stream_completion(agent: &mut Agent, _: ()) -> AgentResult<ProviderStreamHandler> {
    let cs = agent
        .completion_model
        .get_stream_completion(&agent.cache)
        .await?;

    Ok(cs.into())
}

pub async fn function_completion(agent: &mut Agent, function: (Value, &str)) -> AgentResult<Value> {
    Ok(agent
        .completion_model
        .get_fn_completion(&agent.cache, function.0, function.1)
        .await?)
}
