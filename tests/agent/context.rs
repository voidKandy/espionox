use consoxide::telemetry::{get_subscriber, init_subscriber};
use consoxide::{
    agent::config::{
        context::{Context, Contextual},
        memory::{LoadedMemory, Memory},
    },
    core::file_interface::{Directory, File},
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

    let mut context = Context::build(&Memory::Forget);

    files.make_relevant(&mut context);

    assert_eq!(context.buffer.len(), 1);
    let message = context.buffer.first().unwrap();
    assert_eq!(message["role"], "system");
    assert_eq!(
        message["content"].as_str().unwrap(),
        "Relavent Files: [FilePath: path/to/file1.txt, Content: , FilePath: path/to/file2.txt, Content: , Summary: Summary of file 2]"
    );
}

#[test]
fn short_term_switch_works() {
    Lazy::force(&TRACING);
    let mut context = Context::build(&Memory::Remember(LoadedMemory::Cache));
    context.push_to_buffer("tester", "test");
    context.push_to_buffer("tester", "test2");
    let old = context.buffer.clone();
    context.switch_mem(Memory::Forget);
    context.push_to_buffer("system", "");
    context.switch_mem(Memory::Remember(LoadedMemory::Cache));
    assert_eq!(context.buffer, old)
}
