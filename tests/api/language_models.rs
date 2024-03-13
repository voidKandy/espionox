use espionox::language_models::openai::endpoints::embeddings::{
    get_embedding, OpenAiEmbeddingModel,
};

use crate::init_test;

#[ignore]
#[tokio::test]
async fn openai_embedding_works() {
    init_test();
    dotenv::dotenv().ok();
    let text = "Heyyyy this is a test";
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let response = get_embedding(&client, &api_key, text, OpenAiEmbeddingModel::Small).await;
    assert!(response.is_ok())
}
