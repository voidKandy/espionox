use crate::agents::{
    language_models::{error::ModelEndpointError, openai::gpt::models::GptResponse, LanguageModel},
    memory::MessageVector,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::fs::File;
use std::io::Read;

/// Builds context JsonValue needed for vision endpoint
/// Accepts either local or web image path  

#[tracing::instrument(
    name = "Converts message vector and image data into digestible JSON",
    skip(image_buffer)
)]
pub fn message_vector_to_context_with_image(
    vec: &mut MessageVector,
    image_path: Option<&str>,
    image_buffer: Option<Vec<u8>>,
) -> Vec<Value> {
    let mut return_vec = vec![];
    let mut image_url = String::new();
    if let Some(path) = image_path {
        image_url = match path.find("https://") {
            Some(_) => path.to_string(),
            None => {
                let mut file = File::open(path).expect("Unable to open file");
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).expect("Unable to read file");
                let base64_encoded = base64::encode(&buffer);
                format!("data:image/png;base64,{}", base64_encoded)
            }
        };
    } else if let Some(buf) = image_buffer {
        let base64_encoded = base64::encode(&buf);
        image_url = format!("data:image/png;base64,{}", base64_encoded)
    }
    let last = vec.as_mut().pop().unwrap();
    vec.as_ref().into_iter().for_each(|m| {
        return_vec.push(json!({
            "role": m.role.to_string(),
            "content": [{
                "type": "text",
                "text": m.content
            }]
        }));
    });
    return_vec.push(json!({
            "role": last.role.to_string(),
            "content": [
            { "type": "text", "text": last.content },
            {
                "type": "image_url",
                "image_url": {
                    "url": image_url
                }
            }
            ]
    }));
    return_vec
}

#[tracing::instrument(name = "Get vision completion", skip(client, api_key, model, context))]
pub async fn vision_completion(
    client: &Client,
    api_key: &str,
    context: &Vec<Value>,
    model: &LanguageModel,
) -> Result<GptResponse, ModelEndpointError> {
    let gpt = model.inner_gpt().unwrap();
    let temperature = (gpt.temperature * 10.0).round() / 10.0;
    let payload = json!({"model": "gpt-4-vision-preview", "messages": context, "temperature": temperature, "max_tokens": 1000});
    let request = client
        .post(model.completion_url())
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload);
    tracing::info!("REQUEST: {:?}", request);

    let response = request.send().await?;
    tracing::info!("RESPONSE: {:?}", response);
    let gpt_response = response.json().await?;
    Ok(gpt_response)
}
