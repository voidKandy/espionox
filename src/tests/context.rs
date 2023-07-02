#[cfg(test)]
use crate::agent::context::tmux_session::Pane;
use crate::agent::context::walk::Directory;

#[allow(dead_code)]
const TEST_DIRECTORY: &str = "./src/tests/test-dir";

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
    let response = pane.write_to(&format!("{}/test2.txt", TEST_DIRECTORY));
    assert!(response.is_ok())
}
