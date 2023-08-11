use consoxide::telemetry::{get_subscriber, init_subscriber};
use consoxide::{
    context::{memory::Memory, Context},
    core::file_interface::{File, Memorable},
};
use once_cell::sync::Lazy;
#[cfg(test)]
#[allow(unused)]
use std::path::PathBuf;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    }
});

#[test]
fn adding_files_to_memory_works() {
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

    let mut context = Context::build(Memory::Forget);
    context.push_to_buffer(
        "user",
        &format!(
            "Relavent files: {:?}",
            &files
                .into_iter()
                .map(|f| f.memorize())
                .collect::<Vec<String>>()
        ),
    );
    println!("{}", context.buffer_as_string());
    assert!(
        context.buffer_as_string().contains(
        "Relavent files: [\\\"FilePath: path/to/file1.txt, Content: \\\", \\\"FilePath: path/to/file2.txt, Content: , Summary: Summary of file 2\\\"]")
    );
}
