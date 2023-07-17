use crate::lib::{
    database::{db, models},
    io::walk,
};
use tokio;

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

#[tokio::test]
async fn post_get_delete_context() {
    let new_context = models::ContextParams {
        name: "Test".to_string(),
    };

    let pool = db::create_pool().await.expect("Problem creating db pool");
    let res = db::post_context(new_context.clone(), &pool).await;
    assert!(res.is_ok());
    let context = db::get_context(new_context.clone(), &pool)
        .await
        .expect("Couldn't get context");
    assert_eq!("Test".to_string(), context.name);
    assert!(db::delete_context(new_context, &pool).await.is_ok());
}

// #[tokio::test]
// async fn post_file_adds_to_database() {
//     let tempfile = walk::File::build("./src/lib/start.sh");
//
//     let pool = db::create_pool().await.expect("Problem creating db pool");
//     db::post_file(file, &pool)
// }
