use crate::{
    configuration::ConfigEnv,
    context::memory::{Message, MessageVector},
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Prompt {
    pub name: String,
    pub messages: Vec<Message>,
}

type Prompts = Vec<Prompt>;

#[tracing::instrument(name = "Get prompts from prompts file")]
pub fn get_prompts_from_file() -> Result<Prompts, anyhow::Error> {
    let config_dir = ConfigEnv::config_dir_path();
    let prompt_yaml_file = config_dir.join("prompts.yaml").display().to_string();

    let yaml_data = fs::read_to_string(&prompt_yaml_file)?;
    tracing::info!("Yaml data from file:\n{:?}", yaml_data);
    Ok(serde_yaml::from_str::<Prompts>(&yaml_data)?)
}

pub fn get_prompt_by_name(name: &str) -> Option<MessageVector> {
    let prompts = get_prompts_from_file().expect("Failed to get prompts");
    match prompts.iter().find(|p| p.name == name).cloned() {
        Some(prompt) => Some(prompt.messages.into()),
        None => None,
    }
}

pub fn add_prompt_to_file(prompt: Prompt) -> Result<(), anyhow::Error> {
    let config_dir = ConfigEnv::config_dir_path();
    let prompt_yaml_file = config_dir.join("prompts.yaml");

    let mut prompts = get_prompts_from_file().unwrap_or_else(|_| Prompts::new());
    prompts.push(prompt);

    let serialized_prompts = serde_yaml::to_string(&prompts)?;

    fs::write(prompt_yaml_file, serialized_prompts)?;

    Ok(())
}
