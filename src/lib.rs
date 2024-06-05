pub mod agents;
pub mod errors;
pub mod language_models;
pub mod telemetry;
#[cfg(feature = "tools")]
pub mod tools;

pub mod prelude {
    pub use crate::{
        agents::{
            actions::{function_completion, io_completion, stream_completion},
            error::AgentResult,
            listeners::{AgentListener, ListenerCallReturn, ListenerTrigger},
            memory::{Message, MessageRole, MessageStack},
            Agent,
        },
        language_models::completions::{CompletionModel, CompletionProvider, ModelParameters},
    };
}
