#[cfg(test)]
use crate::agent::context::tmux_session::Pane;
use crate::agent::context::walk::{Directory, File};

#[allow(dead_code)]
const TEST_DIRECTORY: &str = "./src/tests/test-dir";
const TEST_FILE: &str = "./src/tests/test-dir/test2.txt";

#[test]
fn walk_test() {
    println!("{TEST_DIRECTORY}");
    let root = Directory::build(TEST_DIRECTORY).unwrap();
    let test1_content = &root.files[0].content;
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.files.len(), 3);
    assert_eq!(test1_content, "hello from test 1\n")
}

#[test]
fn capture_pane_test() {
    let pane = Pane::capture();
    let response = pane.write_to(TEST_FILE);
    assert!(response.is_ok())
}

#[ignore]
#[tokio::test]
async fn summarize_file_test() {
    let mut root = Directory::build(TEST_DIRECTORY).unwrap();
    let test_file = &mut root.files[0];
    test_file.summarize().await.unwrap();
    assert_ne!("".to_string(), test_file.summary);
}
