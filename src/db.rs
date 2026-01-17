use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
pub async fn create_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Error creating pool")
}
