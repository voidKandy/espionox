use crate::lib::{
    agent::{
        config::memory::Memory,
        handler::context::{Context, Contextual},
    },
    io::file_interface::{Directory, File},
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
