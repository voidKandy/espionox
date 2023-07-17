use crate::lib::{
    database::{db, models},
    io::walk,
};
use tokio;

#[tokio::test]
async fn post_get_delete_context() {
    let pool = db::DbPool::init().await;

    let new_context = models::ContextParams {
        name: "Test".to_string(),
    };
    let res = pool.post_context(new_context.clone()).await;
    assert!(res.is_ok());

    let context = pool
        .get_context(new_context.clone())
        .await
        .expect("Couldn't get context");
    assert_eq!("Test".to_string(), context.name);
    assert!(pool.delete_context(new_context).await.is_ok());
}

#[tokio::test]
async fn post_get_delete_file() {
    let pool = db::DbPool::init().await;
    let newfile = models::CreateFileBody {
        context_id: "9999".to_string(),
        filepath: "path/to/test/file".to_string(),
        parent_dir_path: "path/to/test".to_string(),
        summary: "Summary".to_string(),
        summary_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = pool.post_file(newfile).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let gotfile = pool
        .get_file(models::GetFileParams {
            filepath: "path/to/test/file".to_string(),
        })
        .await;
    if let Err(e) = gotfile {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("9999".to_string(), gotfile.unwrap().context_id);
    assert!(pool
        .delete_file(models::DeleteFileParams {
            filepath: "path/to/test/file".to_string()
        })
        .await
        .is_ok());
}
// #[tokio::test]
// async fn post_file_adds_to_database() {
//     let tempfile = walk::File::build("./src/lib/start.sh");
//
//     let pool = db::create_pool().await.expect("Problem creating db pool");
//     db::post_file(file, &pool)
// }
