// crud.rs
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

/// Vytvori noveho pouzivatela a zahashuje heslo
///
/// # Parametre
/// - name: pouzivatelske meno (musi byt unikatne)
/// - password: heslo v plain texte (bude zahashovane pomocou Argon2)
///
/// # Navratova hodnota
/// Vracia PublicUser (bez hesla) alebo chybu ak pouzivatel uz existuje
///
/// # Bezpecnost
/// Heslo je zahashovane pomocou Argon2 s nahodnou solu pred ulozenim do databazy
pub async fn create_user(name: &str, password: &str) -> Result<PublicUser, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // Hashovanie hesla pomocou Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Generovanie UUID pre noveho pouzivatela
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

/// Ziska pouzivatela podla jeho ID
///
/// # Parametre
/// - user_id: UUID pouzivatela
///
/// # Navratova hodnota
/// Vracia PublicUser alebo chybu ak pouzivatel neexistuje
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

/// Zmaze pouzivatela a vsetky jeho ucty
///
/// # Parametre
/// - user_id: UUID pouzivatela na zmazanie
///
/// # Navratova hodnota
/// Vracia pocet zmazanych riadkov (0 ak pouzivatel neexistoval)
///
/// # Poznamka
/// Najprv su zmazane vsetky ucty pouzivatela, potom samotny pouzivatel
pub async fn delete_user(user_id: Uuid) -> Result<u64, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // Najprv zmazeme vsetky ucty pouzivatela
    query!("DELETE FROM accounts WHERE user_id = $1", user_id)
        .execute(&pool)
        .await?;

    // Potom zmazeme samotneho pouzivatela
    let result = query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await?;

    Ok(result.rows_affected())
}

/// Vytvori novy bankovy ucet pre pouzivatela
///
/// # Parametre
/// - user_id: UUID pouzivatela, pre ktoreho sa ma ucet vytvorit
///
/// # Navratova hodnota
/// Vracia PubAccount s nulovou pociatocnou bilanciou
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

/// Ziska vsetky ucty pouzivatela
///
/// # Parametre
/// - user_id: UUID pouzivatela
///
/// # Navratova hodnota
/// Vracia zoznam vsetkych uctov pouzivatela (moze byt prazdny)
pub async fn get_account(user_id: Uuid) -> Result<Vec<PubAccount>, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let rows = query!(
        "SELECT id, user_id, balance FROM accounts WHERE user_id=$1",
        user_id
    )
    .fetch_all(&pool)
    .await?;

    // Konvertovanie riadkov z databazy na PubAccount struktury
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

/// Prida peniaze na ucet
///
/// # Parametre
/// - account_id: UUID uctu
/// - money: suma na pridanie (musi byt kladna)
///
/// # Navratova hodnota
/// Vracia aktualizovany PubAccount s novou bilanciou
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

/// Vytvori transakciu - prevod penazi medzi dvoma uctami
///
/// # Parametre
/// - from_account: UUID uctu odosielatela
/// - to_account: UUID uctu prijemcu
/// - amount: suma prevodu
///
/// # Navratova hodnota
/// Vracia Transaction objekt alebo chybu
///
/// # Bezpecnost a validacia
/// - Pouziva databazovu transakciu (BEGIN/COMMIT) pre ACID vlastnosti
/// - Overuje ci ma odosielatel dostatocny zostatok
/// - Pouziva FOR UPDATE zamok pre zabranenie race conditions
/// - Ak akakolvek operacia zlyhava, vsetky zmeny su automaticky stornovane (ROLLBACK)
///
/// # Chyby
/// - sqlx::Error::RowNotFound: nedostatocny zostatok na ucte odosielatela
/// - Ine sqlx::Error: problemy s databazou alebo neexistujuce ucty
pub async fn make_transaction(
    from_account: Uuid,
    to_account: Uuid,
    amount: Decimal,
) -> Result<Transaction, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    // Zacatie databazovej transakcie - zabezpecuje atomicitu operacie
    let mut tx = pool.begin().await?;

    // Kontrola zostatku odosielatela a zablokovanie riadku (FOR UPDATE)
    let sender = query!(
        "SELECT balance FROM accounts WHERE id = $1 FOR UPDATE",
        from_account
    )
    .fetch_one(&mut *tx)
    .await?;

    // Validacia - overenie dostatocneho zostatku
    if sender.balance < amount {
        return Err(sqlx::Error::RowNotFound);
    }

    // Odcitanie penazi z uctu odosielatela
    query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount,
        from_account
    )
    .execute(&mut *tx)
    .await?;

    // Pripocitanie penazi na ucet prijemcu
    query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        to_account
    )
    .execute(&mut *tx)
    .await?;

    // Vytvorenie zaznamu transakcie v tabulke
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

    // Potvrdenie transakcie - vsetky zmeny su trvale ulozene
    // Ak nedojde k commit(), zmeny sa automaticky stornuju
    tx.commit().await?;

    Ok(Transaction {
        id: transaction.id,
        from_account: transaction.from_account,
        to_account: transaction.to_account,
        amount: transaction.amount,
        created_at: transaction.created_at,
    })
}

/// Prihlasenie pouzivatela pomocou mena a hesla
///
/// # Parametre
/// - username: pouzivatelske meno
/// - password: heslo v plain texte
///
/// # Navratova hodnota
/// Vracia PublicUser alebo String s chybovou spravou
///
/// # Bezpecnost
/// - Heslo je overovane pomocou Argon2 verify funkcie
/// - Nehashuje sa znovu, len sa porovna s ulozenim hashom
///
/// # Chyby
/// - "User not found": pouzivatel s danym menom neexistuje
/// - "Invalid password hash": chyba pri parsovani hashu z databazy
/// - "Invalid password": heslo sa nezhoduje
pub async fn login_user(username: &str, password: &str) -> Result<PublicUser, String> {
    let pool: PgPool = create_pool().await;

    // Ziskanie pouzivatela z databazy
    let user = query!(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| "User not found".to_string())?;

    // Parsovanie hashu hesla z databazy
    let parsed_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| "Invalid password hash".to_string())?;

    // Overenie hesla pomocou Argon2
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid password".to_string())?;

    Ok(PublicUser {
        id: user.id,
        username: user.username,
    })
}

/// Ziska historiu vsetkych transakci pre dany ucet
///
/// # Parametre
/// - account_id: UUID uctu
///
/// # Navratova hodnota
/// Vracia zoznam vsetkych transakci (odoslanych aj prijatych) zoradeny podla casu
///
/// # Poznamka
/// Transakcie su zoradene zostupne podla created_at (najnovsie prvy)
pub async fn get_transaction_history(account_id: Uuid) -> Result<Vec<Transaction>, sqlx::Error> {
    let pool: PgPool = create_pool().await;

    let rows = query!(
        "SELECT id, from_account, to_account, amount, created_at 
         FROM transactions 
         WHERE from_account = $1 OR to_account = $1
         ORDER BY created_at DESC",
        account_id
    )
    .fetch_all(&pool)
    .await?;

    // Konvertovanie riadkov z databazy na Transaction struktury
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
