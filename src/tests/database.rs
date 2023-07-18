use crate::lib::database::{db, models};
use tokio;

// ------ CONTEXT ------ //
#[ignore]
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

// ------ FILES ------ //
#[ignore]
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

// ------ FILECHUNKS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_filechunks() {
    let pool = db::DbPool::init().await;
    let newchunk = models::CreateFileChunkBody {
        parent_file_id: "9999".to_string(),
        idx: 1 as i16,
        content: "chunk content".to_string(),
        content_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = pool.post_file_chunk(newchunk).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let gotchunk = pool
        .get_file_chunks(models::GetFileChunkParams {
            parent_file_id: "9999".to_string(),
        })
        .await;
    if let Err(e) = gotchunk {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("9999".to_string(), gotchunk.unwrap()[0].parent_file_id);
    assert!(pool
        .delete_file_chunk(models::DeleteFileChunkParams {
            parent_file_id: "9999".to_string(),
            idx: 1,
        })
        .await
        .is_ok());
}

// ------ ERRORS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_errors() {
    let pool = db::DbPool::init().await;
    let newerror = models::CreateErrorBody {
        context_id: "9999".to_string(),
        content: "error content".to_string(),
        content_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = pool.post_error(newerror).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let goterror = pool
        .get_errors(models::GetErrorParams {
            context_id: "9999".to_string(),
        })
        .await;
    if let Err(e) = goterror {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("9999".to_string(), goterror.unwrap()[0].context_id);
    assert!(pool
        .delete_error(models::DeleteErrorParams {
            id: "9999".to_string()
        })
        .await
        .is_ok());
}
