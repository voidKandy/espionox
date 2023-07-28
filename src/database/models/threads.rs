#[derive(sqlx::FromRow, Clone, Debug)]
pub struct ThreadModelSql {
    pub name: String,
}

impl ThreadModelSql {}
