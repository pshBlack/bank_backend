// db.rs
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

/// Vytvori connection pool pre PostgreSQL databazu
///
/// # Navratova hodnota
/// Vracia PgPool - pool spojeni s databazou
///
/// # Konfiguracnia
/// Citanie DATABASE_URL z .env suboru alebo systemovych premennych
///
/// # Panika
/// Funkcia zahlasi paniku ak:
/// - DATABASE_URL nie je nastavena
/// - Spojenie s databazou zlyhalo
///
/// # Priklad DATABASE_URL
/// ```
/// DATABASE_URL=postgresql://postgres:heslo@localhost/bank_db
/// ```
pub async fn create_pool() -> PgPool {
    // Nacitanie premennych z .env suboru (ak existuje)
    dotenv().ok();

    // Ziskanie DATABASE_URL z environmentu
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Vytvorenie connection pool
    PgPool::connect(&database_url)
        .await
        .expect("Error creating pool")
}
