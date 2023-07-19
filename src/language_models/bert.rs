use rust_bert::pipelines::sentence_embeddings::{
    Embedding, SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::error::Error;

pub fn embed(contents: &[&str]) -> Result<Vec<Embedding>, Box<dyn Error>> {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()?;

    Ok(model.encode(contents)?)
}
