use crate::helpers;
use consoxide::{
    configuration::ConfigEnv,
    core::File,
    database::{
        api::{vector_query_file_chunks, vector_query_files, CreateFileChunksVector},
        handlers,
        init::DbPool,
        models::{
            error::{CreateErrorBody, DeleteErrorParams, GetErrorParams},
            file::{CreateFileBody, DeleteFileParams, GetFileParams},
            file_chunks::{CreateFileChunkBody, DeleteFileChunkParams, GetFileChunkParams},
        },
    },
};
use rust_bert::pipelines::sentence_embeddings::Embedding;
use tokio;

#[tokio::test]
async fn testing_pool_health_check() {
    assert!(DbPool::init_pool(ConfigEnv::Testing).await.is_ok());
}

#[tokio::test]
async fn nearest_vectors_works() {
    let pool = DbPool::init_pool(ConfigEnv::Testing)
        .await
        .expect("Failed to init testing pool");
    // let mut rng = rand::thread_rng();

    // let vector: Embedding = (0..384).map(|_| rng.gen::<f32>()).collect();
    let vector = filepath_to_database().await;
    assert!(vector_query_file_chunks(&pool, vector.clone(), 5)
        .await
        .is_ok());
    assert!(vector_query_files(&pool, vector, 5).await.is_ok());
}

async fn filepath_to_database() -> Embedding {
    let pool = DbPool::init_pool(ConfigEnv::Testing)
        .await
        .expect("Failed to init testing pool");
    let settings = helpers::test_settings();
    let mut f = File::from("./src/main.rs");
    let file_chunks = f.chunks.clone();
    let file =
        CreateFileBody::build_from(&mut f, &settings.memory().unwrap().threadname().unwrap())
            .expect("Failed to build create file sql body");

    let ret = file.summary_embedding.to_vec().clone();

    let chunks = CreateFileChunksVector::build_from(file_chunks, &file.id)
        .expect("Failed to build create file chunks sql body");
    assert!(handlers::file::post_file(&pool, file).await.is_ok());
    for chunk in chunks.as_ref().iter() {
        assert!(handlers::file_chunks::post_file_chunk(&pool, chunk.clone())
            .await
            .is_ok());
    }
    ret
}

// ------ THREADS ------ //
#[ignore]
#[tokio::test]
async fn post_get_delete_threads() {
    let pool = DbPool::init_pool(ConfigEnv::Testing)
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
    let pool = DbPool::init_pool(ConfigEnv::Testing)
        .await
        .expect("failed to init testing pool");
    let newfile = CreateFileBody {
        id: uuid::Uuid::new_v4().to_string(),
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
    let pool = DbPool::init_pool(ConfigEnv::Testing)
        .await
        .expect("failed to init testing pool");
    let newchunk = CreateFileChunkBody {
        parent_file_id: "9999".to_string(),
        parent_filepath: ".".to_string(),
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
    let pool = DbPool::init_pool(ConfigEnv::Testing)
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
