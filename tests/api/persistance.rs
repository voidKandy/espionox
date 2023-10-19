use espionox::{
    context::memory::Message,
    persistance::prompts::{add_prompt_to_file, get_prompt_by_name, get_prompts_from_file, Prompt},
};

#[test]
fn get_prompts_from_file_works() {
    let prompts = get_prompts_from_file();
    tracing::info!("Prompts:\n{:?}", prompts);
    assert!(prompts.is_ok());
}

#[test]
fn get_default_init_prompt_works() {
    if let Some(prompt) = get_prompt_by_name("DEFAULT_INIT_PROMPT") {
        println!("INIT_PROMPT:\n{:?}", prompt);
        assert!(true);
        return;
    }
    assert!(false);
}

#[test]
fn add_prompt_to_file_works() {
    let prompt_to_add = Prompt {
        name: "test".to_string(),
        messages: vec![Message::new_standard(
            espionox::context::memory::MessageRole::System,
            "this is a test",
        )],
    };
    let len_before = get_prompts_from_file().unwrap().len();
    add_prompt_to_file(prompt_to_add).unwrap();
    let len_after = get_prompts_from_file().unwrap().len();
    assert_eq!(len_after - len_before, 1);
}
