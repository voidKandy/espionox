pub mod agent;
pub mod config;

fn main() {
    let root = config::Directory::build("test-dir").unwrap();
    println!("{}", root);
    let rand_content = &root.files[0].content;
    println!("{rand_content}");
}
