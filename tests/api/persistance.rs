use espionox::persistance::prompts::{get_prompt_by_name, get_prompts_from_file};

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
