use crate::db::create_pool;
use crate::models::PublicUser;
use crate::{PubAccount, Transaction};
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{self, Argon2, password_hash::PasswordHasher};
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::query;
use uuid::Uuid;
/// Створює нового користувача та хешує пароль
pub async fn create_user(name: &str, password: &str) -> Result<PublicUser, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // Хешуємо пароль
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Генеруємо UUID для користувача
    let user_id = Uuid::new_v4();

    let row = query!(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3) RETURNING id, username, password_hash",
        user_id,
        name,
        password_hash
    )
    .fetch_one(&pool)
    .await?;

    Ok(PublicUser {
        id: row.id,
        username: row.username,
    })
}

pub async fn get_user(user_id: Uuid) -> Result<PublicUser, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let row = query!("SELECT id, username FROM users WHERE id=$1", user_id)
        .fetch_one(&pool)
        .await?;

    Ok(PublicUser {
        id: row.id,
        username: row.username,
    })
}

pub async fn delete_user(user_id: Uuid) -> Result<u64, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // Спочатку видаляємо всі рахунки
    query!("DELETE FROM accounts WHERE user_id = $1", user_id)
        .execute(&pool)
        .await?;

    // Потім видаляємо користувача
    let result = query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn create_account(user_id: Uuid) -> Result<PubAccount, sqlx::Error> {
    let pool: PgPool = create_pool().await;
    let account_id = Uuid::new_v4();

    let row = query!(
        "INSERT INTO accounts (id, user_id, balance) VALUES ($1,$2,$3) RETURNING id, user_id, balance",
        account_id,
        user_id,
        Decimal::ZERO
    ).fetch_one(&pool).await?;

    Ok(PubAccount {
        id: row.id,
        user_id: row.user_id,
        balance: row.balance,
    })
}
pub async fn get_account(user_id: Uuid) -> Result<Vec<PubAccount>, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let rows = query!(
        "SELECT id, user_id, balance FROM accounts WHERE user_id=$1",
        user_id
    )
    .fetch_all(&pool)
    .await?;
    // Перетворюємо кожен рядок у PubAccount
    let accounts = rows
        .into_iter()
        .map(|row| PubAccount {
            id: row.id,
            user_id: row.user_id,
            balance: row.balance,
        })
        .collect();

    Ok(accounts)
}
pub async fn add_money(account_id: Uuid, money: Decimal) -> Result<PubAccount, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let row = query!(
        "UPDATE accounts SET balance=balance+$1 WHERE id=$2 RETURNING id, user_id, balance",
        money,
        account_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(PubAccount {
        id: row.id,
        user_id: row.user_id,
        balance: row.balance,
    })
}
pub async fn make_transaction(
    from_account: Uuid,
    to_account: Uuid,
    amount: Decimal,
) -> Result<Transaction, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // КРИТИЧНО: Починаємо SQL транзакцію
    let mut tx = pool.begin().await?;

    // 1. Перевіряємо баланс відправника
    let sender = query!(
        "SELECT balance FROM accounts WHERE id = $1 FOR UPDATE", // FOR UPDATE блокує рядок
        from_account
    )
    .fetch_one(&mut *tx)
    .await?;

    if sender.balance < amount {
        return Err(sqlx::Error::RowNotFound); // Недостатньо коштів
    }

    // 2. Знімаємо гроші з відправника
    query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount,
        from_account
    )
    .execute(&mut *tx)
    .await?;

    // 3. Додаємо гроші отримувачу
    query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        to_account
    )
    .execute(&mut *tx)
    .await?;

    // 4. Створюємо запис транзакції
    let trans_id = Uuid::new_v4();
    let transaction = query!(
        "INSERT INTO transactions (id, from_account, to_account, amount) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, from_account, to_account, amount, created_at",
        trans_id,
        from_account,
        to_account,
        amount
    )
    .fetch_one(&mut *tx)
    .await?;

    // 5. Підтверджуємо транзакцію (все або нічого!)
    tx.commit().await?;

    Ok(Transaction {
        id: transaction.id,
        from_account: transaction.from_account,
        to_account: transaction.to_account,
        amount: transaction.amount,
        created_at: transaction.created_at,
    })
}

pub async fn login_user(username: &str, password: &str) -> Result<PublicUser, String> {
    let pool: PgPool = create_pool().await;

    let user = query!(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| "User not found".to_string())?;

    let parsed_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| "Invalid password hash".to_string())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid password".to_string())?;

    Ok(PublicUser {
        id: user.id,
        username: user.username,
    })
}

pub async fn get_transaction_history(account_id: Uuid) -> Result<Vec<Transaction>, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let rows = query!(
        "SELECT id, from_account, to_account, amount, created_at 
         FROM transactions 
         WHERE from_account = $1 OR to_account = $1  -- ← ВИПРАВЛЕНО!
         ORDER BY created_at DESC",
        account_id
    )
    .fetch_all(&pool)
    .await?;

    let transactions = rows
        .into_iter()
        .map(|row| Transaction {
            id: row.id,
            from_account: row.from_account,
            to_account: row.to_account,
            amount: row.amount,
            created_at: row.created_at,
        })
        .collect();
    Ok(transactions)
}
