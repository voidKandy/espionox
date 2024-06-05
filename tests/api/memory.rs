#[cfg(test)]
mod tests {
    use espionox::{
        agents::memory::{Message, MessageRole, MessageStack, OtherRoleTo},
        language_models::completions::{
            anthropic::builder::AnthropicCompletionModel, inference::CompletionRequestBuilder,
        },
    };

    #[test]
    fn message_stack_filter_by_behavior() {
        let mut stack = MessageStack::new("SYSTEM");
        stack.push(Message::new_user("USER"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_assistant("ASS"));
        stack.push(Message::new_assistant("ASS1"));
        let mut clone = stack.clone();
        stack.mut_filter_by(MessageRole::System, true);
        clone.mut_filter_by(MessageRole::System, false);
        assert_eq!(1, stack.len());
        assert_eq!(5, clone.len());
        let clone_ref = clone.ref_filter_by(MessageRole::User, false);
        assert_eq!(2, clone_ref.len());
        let clone_ref = clone_ref.filter_by(MessageRole::User, true);
        assert_eq!(0, clone_ref.len());
    }

    #[test]
    fn message_stack_pop_behavior() {
        let mut stack = MessageStack::new("SYSTEM");
        stack.push(Message::new_user("USER"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_assistant("ASS"));
        stack.push(Message::new_assistant("ASS1"));
        assert_eq!("USE2", &stack.pop(Some(MessageRole::User)).unwrap().content);
        assert_eq!(
            "ASS1",
            &stack.pop(Some(MessageRole::Assistant)).unwrap().content
        );
        assert_eq!("ASS", &stack.pop(None).unwrap().content);
        let mut stack_ref = stack.ref_filter_by(MessageRole::User, true);
        let m = stack_ref.pop(None).unwrap();
        assert_eq!("USE1", m.content);
        println!("{:?}", stack_ref);
        let m = stack_ref.pop(Some(MessageRole::System));
        assert_eq!(None, m);
    }

    #[test]
    fn anthropic_agent_cache_to_json() {
        let mut stack = MessageStack::new("SYSTEM");
        stack.push(Message::new_user("USER"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_assistant("ASS"));
        stack.push(Message::new_assistant("ASS1"));
        stack.push(Message::new_user("USE1"));
        stack.push(Message::new_user("USE2"));
        stack.push(Message::new_other("some_other", "USE2", OtherRoleTo::User));
        stack.push(Message::new_other(
            "some_other",
            "ASS",
            OtherRoleTo::Assistant,
        ));
        let handler = AnthropicCompletionModel::default();
        let vals = handler.serialize_messages(&stack);
        println!("VALS: {:?}", vals);
        let stack: MessageStack =
            MessageStack::try_from(vals.as_array().unwrap().to_owned()).unwrap();
        assert_eq!(5, stack.len());
    }
}
