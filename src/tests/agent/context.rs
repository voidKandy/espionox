use crate::lib::{
    agent::config::{
        context::{Context, Contextual},
        memory::Memory,
    },
    io::{
        tmux_session::TmuxSession,
        walk::{Directory, File},
    },
};
#[cfg(test)]
#[allow(unused)]
use std::path::PathBuf;

#[test]
fn adding_files_to_context_works() {
    let files = vec![
        File {
            filepath: PathBuf::from("path/to/file1.txt").into(),
            chunks: vec![],
            summary: "".to_string(),
            summary_embedding: vec![],
        },
        File {
            filepath: PathBuf::from("path/to/file2.txt").into(),
            chunks: vec![],
            summary: "Summary of file 2".to_string(),
            summary_embedding: vec![],
        },
    ];

    let mut context = Memory::ShortTerm.init();

    files.make_relevant(&mut context);

    assert_eq!(context.messages.len(), 1);
    let message = context.messages.first().unwrap();
    assert_eq!(message["role"], "system");
    assert_eq!(
        message["content"].as_str().unwrap(),
        "Relavent Files: [FilePath: path/to/file1.txt, Content: , FilePath: path/to/file2.txt, Content: , Summary: Summary of file 2]"
    );
}

#[test]
fn short_term_switch_works() {
    let mut context = Memory::ShortTerm.init();
    context.append_to_messages("tester", "test");
    context.append_to_messages("tester", "test2");
    let old = context.messages.clone();
    context.switch(Memory::Temporary);
    context.append_to_messages("system", "");
    context.switch(Memory::ShortTerm);
    assert_eq!(context.messages, old)
}

// #[test]
// fn parsing_tmux_output_works() {
//     let pane = TmuxSession::new();
//     let test_output = format!(
//         "{} IHGFEDCBA {} {} ABCDEFGHI {}",
//         pane.match_patterns.0, pane.match_patterns.1, pane.match_patterns.0, pane.match_patterns.1
//     );
//     let last_out = pane.get_last_output(test_output);
//     assert_eq!(last_out, " ABCDEFGHI ");
// }

#[test]
fn test_to_out() {
    let session = TmuxSession::new();
    session.to_out("Im going to be so annoyed if i can't see this entire message. I've worked so hard and yet I find myself having to deal with another dumbass problem.");
    assert!(false);
}

#[test]
fn fail() {
    assert!(false);
}
