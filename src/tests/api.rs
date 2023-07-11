use crate::api::{bert, database::db};

#[cfg(test)]
#[tokio::test]
async fn test_create_pool() {
    match db::create_pool().await {
        Ok(_) => assert!(true),
        Err(err) => {
            panic!("Error: {err:?}");
        }
    };
}

#[ignore]
#[test]
fn test_embedding() {
    let embedding = bert::embed(&["nut"]);
    println!("{:?}", embedding);
    assert!(embedding.is_ok());
}
