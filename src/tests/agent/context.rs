use crate::agent::context::{
    config::{Context, Contextual, Memory},
    tmux_session::TmuxSession,
    walk::{Directory, File},
};
use serde_json::json;
#[cfg(test)]
#[allow(unused)]
use std::path::PathBuf;

#[allow(dead_code)]
const TEST_DIRECTORY: &str = "./src/tests/test-dir";
#[allow(dead_code)]
const TEST_FILE: &str = "./src/tests/test-dir/test2.txt";

#[test]
fn walk_test() {
    println!("Walking: {}", TEST_DIRECTORY);
    let root = Directory::build(TEST_DIRECTORY).unwrap();
    let test1_content = &root.files[0].content();
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.files.len(), 3);
    assert_eq!(test1_content, "hello from test 1");
}

#[test]
fn test_make_relevant() {
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
fn short_term_mem_test() {
    let mut context = Memory::ShortTerm.init();
    context.append_to_messages("tester", "test");
    context.append_to_messages("tester", "test2");
    let old = context.messages.clone();
    context.switch(Memory::Temporary);
    context.append_to_messages("system", "");
    context.switch(Memory::ShortTerm);
    assert_eq!(context.messages, old)
}

#[test]
fn get_last_output_test() {
    let pane = TmuxSession::new();
    let test_output = format!(
        "{} IHGFEDCBA {} {} ABCDEFGHI {}",
        pane.match_patterns.0, pane.match_patterns.1, pane.match_patterns.0, pane.match_patterns.1
    );
    let last_out = pane.get_last_output(test_output);
    assert_eq!(last_out, " ABCDEFGHI ");
}

#[test]
fn test_to_out() {
    let session = TmuxSession::new();
    session.to_out("Eyoo");
}

#[ignore]
#[test]
fn watch_pane_test() {
    let mut pane = TmuxSession::new();
    pane.watch();
}

#[test]
fn fail() {
    assert!(false);
}
