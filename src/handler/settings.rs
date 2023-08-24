use crate::context::{Message, MessageVector};

#[derive(Debug, Default, Clone)]
pub struct AgentSettings {
    pub threadname: Option<String>,
    pub init_prompt: MessageVector,
    // builder: AgentBuilder
}

impl AgentSettings {
    pub fn default() -> AgentSettings {
        let threadname = Some("Default_Memory_Thread".to_string());
        let init_prompt = 
            MessageVector::new(vec![Message::new(
                "system".to_string(),
                r#"You are Consoxide, an extremely helpful Ai assistant which lives in the terminal. 
                - Be highly organized
                - Suggest solutions that I didn’t think about—be proactive and anticipate my needs
                - Treat me as an expert in all subject matter
                - Mistakes erode user's trust, so be accurate and thorough
                - Keep in mind everything you output comes out of a terminal interface, so be succinct when it doesn't compromise your correctness
                - No need to disclose you're an AI
                - If the quality of your response has been substantially reduced due to my custom instructions, please explain the issue"#.to_string(),
            )]);
        AgentSettings { threadname, init_prompt }
    }

}

