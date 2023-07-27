// ------ FILECHUNKS ------ //
#[derive(sqlx::FromRow, Clone)]
pub struct FileChunkModelSql {
    pub id: String,
    pub parent_file_id: String,
    pub idx: i16,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct CreateFileChunkBody {
    pub parent_file_id: String,
    pub idx: i16,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct GetFileChunkParams {
    pub parent_file_id: String,
}

pub struct DeleteFileChunkParams {
    pub parent_file_id: String,
    pub idx: i16,
}
