use crate::agent::context::{
    config::{Context, Contextual},
    tmux_session::Pane,
    walk::{Directory, File},
};
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
    let test1_content = &root.files[0].content;
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.files.len(), 3);
    assert_eq!(test1_content, "hello from test 1\n")
}

#[test]
fn test_make_relevant() {
    let files = vec![
        File {
            filepath: PathBuf::from("path/to/file1.txt").into(),
            content_embedding: vec![],
            content: "File 1 content".to_string(),
            summary: "".to_string(),
            summary_embedding: vec![],
        },
        File {
            filepath: PathBuf::from("path/to/file2.txt").into(),
            content_embedding: vec![],
            content: "File 2 content".to_string(),
            summary: "Summary of file 2".to_string(),
            summary_embedding: vec![],
        },
    ];

    let mut context = Context::new("test", None);

    files.make_relevant(&mut context);

    let messages = context.current_messages();
    assert_eq!(messages.len(), 1);
    let message = messages.first().unwrap();
    assert_eq!(message["role"], "system");
    assert_eq!(
            message["content"].as_str().unwrap(),
            "Relavent Files: [FilePath: path/to/file1.txt, Content: File 1 content, FilePath: path/to/file2.txt, Content: File 2 content, Summary: Summary of file 2]"
        );
}

#[test]
fn test_chunking_files() {
    let test_file = File::build("./src/tests/agent/context.rs");
    test_file.chunkify();
    assert!(false);
}

#[ignore]
#[test]
fn watch_pane_test() {
    let mut pane = Pane::new();
    pane.watch();
}

#[test]
fn fail() {
    assert!(false);
}

// #[ignore]
// #[tokio::test]
// async fn summarize_file_test() {
//     let mut root = Directory::build(TEST_DIRECTORY).unwrap();
//     let test_file = &mut root.files[0];
//     test_file.summarize().await.unwrap();
//     assert_ne!("".to_string(), test_file.summary);
// }
