pub mod huggingface;
pub mod openai;

pub use huggingface::sentence_embeddings::*;
use openai::gpt::Gpt;
use serde::{Deserialize, Serialize};

use self::openai::gpt::GptModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageModel {
    Gpt(Gpt),
}

impl From<Gpt> for LanguageModel {
    fn from(value: Gpt) -> Self {
        LanguageModel::Gpt(value)
    }
}

impl LanguageModel {
    // Probably should create an into impl trait for this once more models are supported
    /// return a reference to the inner Gpt model struct
    pub fn inner_gpt(&self) -> Option<&Gpt> {
        match self {
            Self::Gpt(g) => Some(g),
            _ => None,
        }
    }
    /// Creates LanguageModel with default gpt settings
    pub fn default_gpt() -> Self {
        let gpt = Gpt::default();
        Self::Gpt(gpt)
    }

    /// Creates LanguageModel given gpt model override and temperature
    pub fn new_gpt(model: GptModel, temperature: f32) -> Self {
        let gpt = Gpt::new(model, temperature);
        Self::Gpt(gpt)
    }
}
