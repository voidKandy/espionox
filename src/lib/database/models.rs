#[derive(sqlx::FromRow, Clone)]
pub struct ContextModelSql {
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct ContextParams {
    pub name: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct FileModelSql {
    pub id: String,
    pub context_id: String,
    pub filepath: String,
    pub parent_dir_path: String,
    pub summary: String,
    pub summary_embedding: pgvector::Vector,
}

pub struct GetFileParams {
    pub filepath: String,
}

pub struct CreateFileBody {
    pub context_id: String,
    pub filepath: String,
    pub parent_dir_path: String,
    pub summary: String,
    pub summary_embedding: pgvector::Vector,
}

pub struct DeleteFileParams {
    pub filepath: String,
}

pub struct CreateFileChunksBody {
    pub parent_file_id: String,
    pub index: u8,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}
