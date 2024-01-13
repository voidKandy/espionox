#[cfg(feature = "long_term_memory")]
#[derive(Clone, Debug)]
pub(crate) struct DatabaseSettings {
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database_name: String,
}

#[cfg(feature = "long_term_memory")]
impl std::fmt::Display for DatabaseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database Url: postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
