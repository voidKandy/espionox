#[derive(sqlx::FromRow, Clone)]
pub struct FileModelSql {
    pub id: String,
    pub thread_name: String,
    pub filepath: String,
    pub parent_dir_path: String,
    pub summary: String,
    pub summary_embedding: super::super::vector_embeddings::EmbeddingVector,
}

pub struct GetFileParams {
    pub filepath: String,
}

#[derive(Clone)]
pub struct CreateFileBody {
    pub id: String,
    pub thread_name: String,
    pub filepath: String,
    pub parent_dir_path: String,
    pub summary: String,
    pub summary_embedding: super::super::vector_embeddings::EmbeddingVector,
}

pub struct DeleteFileParams {
    pub filepath: String,
}
