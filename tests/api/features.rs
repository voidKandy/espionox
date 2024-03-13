use espionox::agents::{
    language_models::{
        openai::gpt::{Gpt, GptModel},
        LanguageModel,
    },
    memory::{Message, MessageStack},
    Agent,
};
use espionox::features::tools::vision::{message_vector_to_context_with_image, vision_completion};
use espionox::features::tools::websurf::Surfer;

use crate::{helpers, init_test, test_env};

#[tokio::test]
async fn vision_completion_works() {
    init_test();
    dotenv::dotenv().ok();
    let client = reqwest::Client::new();
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    tracing::info!("API KEY: {}", api_key);
    let image_url = "./tests/test-screenshot.png";
    let mut messages = MessageStack::new("You are an image looker-atter");
    messages.push(Message::new_user("What is this image?"));
    let gpt = Gpt::new(GptModel::Gpt4, 0.4);
    let model = LanguageModel::OpenAi(gpt);

    let context = message_vector_to_context_with_image(&mut messages, Some(image_url), None);
    tracing::info!("Getting completion response");
    let response = vision_completion(&client, &api_key, &context, &model)
        .await
        .unwrap();
    tracing::info!("{:?}", response);
}

#[tokio::test]
async fn surfer_get_screenshot_works() {
    init_test();
    let mut env = test_env();
    let agent = Agent::default();
    let handle = env.insert_agent(None, agent).await.unwrap();
    let mut surfer = Surfer::from(&handle);
    surfer.get_screenshot("https://docs.rs/headless_chrome/latest/headless_chrome/browser/tab/struct.Tab.html#method.print_to_pdf").unwrap();
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    let desc = surfer
        .description_of_current_screenshot(&api_key)
        .await
        .unwrap();
    tracing::info!("{:?}", desc);
}

#[tokio::test]
async fn surfer_listener_works() {
    init_test();
    let agent = Agent::default();
    let mut environment = helpers::test_env();
    let mut handle = environment.insert_agent(None, agent).await.unwrap();
    let surfer = Surfer::from(&handle);
    environment.insert_listener(surfer).await;
    environment.spawn().await.unwrap();
    let comp_ticket = handle
        .request_io_completion(Message::new_user("What is on the front page of wikipedia?"))
        .await
        .unwrap();
    environment.finalize_dispatch().await.unwrap();
    let comp_noti = environment
        .notifications
        .wait_for_notification(&comp_ticket)
        .await
        .unwrap();
    let response: &Message = comp_noti.extract_body().try_into().unwrap();
    println!("{}", response.content);
}
