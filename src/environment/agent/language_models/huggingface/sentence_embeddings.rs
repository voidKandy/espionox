use rust_bert::pipelines::sentence_embeddings::{
    Embedding, SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::{error::Error, thread};

pub fn embed(content: &str) -> Result<Embedding, anyhow::Error> {
    let contents = content.to_owned();
    // This operation needs to be done on a separate thread because it spawns a tokio runtime
    thread::spawn(move || {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()?;

        let embedding_vector = model.encode(&[contents])?;
        Ok(embedding_vector
            .get(0)
            .expect("Failed to get 0th embedding of vector of embeddings")
            .to_vec())
    })
    .join()
    .expect("Failed to run embedding thread")
}
