// REWRITE AS TEST OF SUM AGENT
// #[test]
// fn adding_files_to_memory_works() {
//     let files = vec![
//         File {
//             filepath: PathBuf::from("path/to/file1.txt").into(),
//             chunks: vec![],
//             summary: None,
//         },
//         File {
//             filepath: PathBuf::from("path/to/file2.txt").into(),
//             chunks: vec![],
//             summary: Some("Summary of file 2".to_string()),
//         },
//     ];
//
//     let mut context = Context::build(Memory::Forget);
//     context.push_to_buffer(
//         "user",
//         &format!(
//             "Relavent files: {:?}",
//             &files
//                 .into_iter()
//                 .map(|f| f.buffer_display())
//                 .collect::<Vec<String>>()
//         ),
//     );
//
//     let expected_content = r#"Relavent files: ["FilePath: path/to/file1.txt, Content: ", "FilePath: path/to/file2.txt, Content: , Summary: Summary of file 2"]"#;
//
//     println!("{}", context.buffer_as_string());
//     assert!(context.buffer_as_string().contains(expected_content));
// }
