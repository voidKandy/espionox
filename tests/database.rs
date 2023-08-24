use consoxide::database::{
    self, handlers,
    init::{DatabaseEnv, DbPool},
    models::{
        error::{CreateErrorBody, DeleteErrorParams, GetErrorParams},
        file::{CreateFileBody, DeleteFileParams, GetFileParams},
        file_chunks::{CreateFileChunkBody, DeleteFileChunkParams, GetFileChunkParams},
    },
};
use tokio;

#[tokio::test]
async fn testing_pool_health_check() {
    assert!(DbPool::init_pool(DatabaseEnv::Testing).await.is_ok())
}

// ------ THREADS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_threads() {
    let pool = DbPool::init_pool(DatabaseEnv::Testing)
        .await
        .expect("failed to init testing pool");

    match handlers::threads::post_thread(&pool, "test").await {
        Ok(_) => {}
        Err(err) => panic!("Problem posting thread: {err:?}"),
    }

    let threads = handlers::threads::get_thread(&pool, "test")
        .await
        .expect("Couldn't get threads");
    assert_eq!("test".to_string(), threads.name);
    assert!(handlers::threads::delete_thread(&pool, "test")
        .await
        .is_ok());
}
//
// ------ FILES ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_file() {
    let pool = DbPool::init_pool(DatabaseEnv::Testing)
        .await
        .expect("failed to init testing pool");
    let newfile = CreateFileBody {
        id: "9999".to_string(),
        thread_name: "test".to_string(),
        filepath: "path/to/test/file".to_string(),
        parent_dir_path: "path/to/test".to_string(),
        summary: "Summary".to_string(),
        summary_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = handlers::file::post_file(&pool, newfile).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let gotfile = handlers::file::get_file(
        &pool,
        GetFileParams {
            filepath: "path/to/test/file".to_string(),
        },
    )
    .await;
    if let Err(e) = gotfile {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("test".to_string(), gotfile.unwrap().thread_name);
    assert!(handlers::file::delete_file(
        &pool,
        DeleteFileParams {
            filepath: "path/to/test/file".to_string()
        }
    )
    .await
    .is_ok());
}

// ------ FILECHUNKS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_filechunks() {
    let pool = DbPool::init_pool(DatabaseEnv::Testing)
        .await
        .expect("failed to init testing pool");
    let newchunk = CreateFileChunkBody {
        parent_file_id: "9999".to_string(),
        idx: 1 as i16,
        content: "chunk content".to_string(),
        content_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = handlers::file_chunks::post_file_chunk(&pool, newchunk).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let gotchunk = handlers::file_chunks::get_file_chunks(
        &pool,
        GetFileChunkParams {
            parent_file_id: "9999".to_string(),
        },
    )
    .await;
    if let Err(e) = gotchunk {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("9999".to_string(), gotchunk.unwrap()[0].parent_file_id);
    assert!(handlers::file_chunks::delete_file_chunk(
        &pool,
        DeleteFileChunkParams {
            parent_file_id: "9999".to_string(),
            idx: 1,
        }
    )
    .await
    .is_ok());
}

// ------ ERRORS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_errors() {
    let pool = DbPool::init_pool(DatabaseEnv::Testing)
        .await
        .expect("failed to init testing pool");
    let newerror = CreateErrorBody {
        thread_name: "test".to_string(),
        content: "error content".to_string(),
        content_embedding: pgvector::Vector::from(vec![0.0; 384]),
    };
    let res = handlers::error::post_error(&pool, newerror).await;
    if let Err(e) = res {
        panic!("Error posting file: {e:?}");
    }

    let goterror = handlers::error::get_errors(
        &pool,
        GetErrorParams {
            thread_name: "test".to_string(),
        },
    )
    .await;
    if let Err(e) = goterror {
        panic!("Error getting file: {e:?}");
    }
    assert_eq!("test".to_string(), goterror.unwrap()[0].thread_name);
    assert!(handlers::error::delete_error(
        &pool,
        DeleteErrorParams {
            id: "9999".to_string()
        }
    )
    .await
    .is_ok());
}
