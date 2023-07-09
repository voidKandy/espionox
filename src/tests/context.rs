#[cfg(test)]
#[allow(unused)]
use crate::agent::context::{
    config::Context,
    tmux_session::Pane,
    walk::{Directory, File},
};

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

// #[ignore]
// #[test]
// fn update_test() {
//     let mut pane = Pane::new();
//     pane.update(None);
//     // println!("{:?}", &pane);
//     // assert_eq!(pane.content.lines().count().clone(), window_size as usize);
//     let response = pane.write_to(TEST_FILE);
//     assert!(response.is_ok())
// }
//
// #[test]
// fn parse_pane_test() {
//     let parse_test_string = String::from(
//         r#"===START====chunk1===END===
//         ===START====chunk2===END===
//         ===START====chunk3===END==="#,
//     );
//     let parsed = Pane::parse_pane(parse_test_string);
//     println!("Parsed pane: {:?}", parsed);
//     assert_eq!(parsed, vec!["chunk1", "chunk2", "chunk3"]);
// }
//
#[test]
fn make_relevant_test() {
    let mut context = Context::new(None);
    let root = Directory::build(TEST_DIRECTORY).unwrap();
    context.make_relevant(Some(&vec![root.clone()]), Some(&root.files));
    assert_eq!(context.directories.len(), 1);
    assert_eq!(context.files.len(), root.files.len());
}

// #[ignore]
#[test]
fn watch_pane_test() {
    let mut pane = Pane::new();
    pane.watch();
}

#[ignore]
#[tokio::test]
async fn summarize_file_test() {
    let mut root = Directory::build(TEST_DIRECTORY).unwrap();
    let test_file = &mut root.files[0];
    test_file.summarize().await.unwrap();
    assert_ne!("".to_string(), test_file.summary);
}
