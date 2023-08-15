use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::thread;
use tokio::runtime::Runtime;

#[derive(Clone, Debug)]
pub struct DbPool(sqlx::PgPool);

impl DbPool {
    async fn init_pool() -> Result<DbPool, Box<dyn Error + Send + Sync>> {
        dotenv().ok();
        let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        match sqlx::postgres::PgPool::connect(&url).await {
            Ok(pool) => Ok(DbPool(pool)),
            Err(err) => Err(format!("Error initializing DB pool: {:?}", err).into()),
        }
    }

    pub fn init_long_term() -> DbPool {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                DbPool::init_pool()
                    .await
                    .expect("Failed to init long term DB")
            })
        })
        .join()
        .expect("Failed to init long term DbPool")
    }
}

impl AsRef<sqlx::PgPool> for DbPool {
    fn as_ref(&self) -> &sqlx::PgPool {
        &self.0
    }
}
