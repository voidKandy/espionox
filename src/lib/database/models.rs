// ------ CONTEXTS ------ //
#[derive(sqlx::FromRow, Clone)]
pub struct ContextModelSql {
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct ContextParams {
    pub name: String,
}

// ------ FILES ------ //
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

// ------ ERRORS ------ //
#[derive(sqlx::FromRow, Clone)]
pub struct ErrorModelSql {
    pub id: String,
    pub context_id: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct CreateErrorBody {
    pub context_id: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct GetErrorParams {
    pub context_id: String,
}

pub struct DeleteErrorParams {
    pub id: String,
}
