#[derive(Clone)]
pub struct ContextParams {
    pub name: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct ContextModelSql {
    pub id: String,
    pub name: String,
}

pub struct InsertFileBody {
    pub context_id: String,
    pub filepath: String,
    pub parent_dir_path: String,
    pub summary: String,
    pub summary_embedding: Vec<f32>,
}

pub struct InsertFileChunksBody {
    pub parent_file_id: String,
    pub index: u8,
    pub content: String,
    pub content_embedding: Vec<f32>,
}
