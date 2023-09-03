use consoxide::database::init::{DatabaseEnv, DbPool};
use tokio;

#[tokio::main]
async fn main() {
    assert!(DbPool::init_pool(DatabaseEnv::Default).await.is_ok());
    println!("Default database is good to go!");
    assert!(DbPool::init_pool(DatabaseEnv::Testing).await.is_ok());
    println!("Testing database is good to go!");
}
