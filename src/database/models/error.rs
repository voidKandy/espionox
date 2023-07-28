// ------ ERRORS ------ //
#[derive(sqlx::FromRow, Clone)]
pub struct ErrorModelSql {
    pub id: String,
    pub thread_name: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct CreateErrorBody {
    pub thread_name: String,
    pub content: String,
    pub content_embedding: pgvector::Vector,
}

pub struct GetErrorParams {
    pub thread_name: String,
}

pub struct DeleteErrorParams {
    pub id: String,
}
