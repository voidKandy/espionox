#[derive(sqlx::FromRow, Clone)]
pub struct ThreadModelSql {
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct ThreadParams {
    pub name: String,
}
