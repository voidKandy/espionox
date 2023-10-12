pub mod huggingface;
pub mod openai;

pub use huggingface::sentence_embeddings::*;
use openai::gpt::Gpt;

use self::openai::gpt::GptModel;

#[derive(Debug)]
pub enum LanguageModel {
    Gpt(Gpt),
}

impl From<Gpt> for LanguageModel {
    fn from(value: Gpt) -> Self {
        LanguageModel::Gpt(value)
    }
}

impl LanguageModel {
    /// Probably should create an into impl trait for this once more models are supported
    pub fn inner_gpt(&self) -> Option<&Gpt> {
        match self {
            Self::Gpt(g) => Some(g),
            _ => None,
        }
    }
    pub fn default_gpt() -> Self {
        let gpt = Gpt::default();
        Self::Gpt(gpt)
    }
    pub fn new_gpt(model: GptModel) -> Self {
        let gpt = Gpt::new(model);
        Self::Gpt(gpt)
    }
}
