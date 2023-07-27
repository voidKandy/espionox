// ------ ERRORS ------ //
#[derive(sqlx::FromRow, Clone)]
pub struct ErrorModelSql {
    pub id: String,
    pub thread_id: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct CreateErrorBody {
    pub thread_id: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct GetErrorParams {
    pub thread_id: String,
}

pub struct DeleteErrorParams {
    pub id: String,
}
