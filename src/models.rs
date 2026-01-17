use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

/// Verejne udaje pouzivatela (bez hesla)
/// Pouziva sa pri vrateni informacii o pouzivatelovi cez API
#[derive(Debug, Clone, Serialize)]
pub struct PublicUser {
    /// Unikatny identifikator pouzivatela
    pub id: Uuid,
    /// Pouzivatelske meno
    pub username: String,
}

/// Verejne udaje bankoveho uctu
/// Obsahuje informacie o zostatku a vlastnikovi uctu
#[derive(Debug, Serialize, Deserialize)]
pub struct PubAccount {
    /// Unikatny identifikator uctu
    pub id: Uuid,
    /// Identifikator vlastnika uctu (vzah k PublicUser)
    pub user_id: Uuid,
    /// Zostatok na ucte (presne desatinne cislo)
    pub balance: Decimal,
}

/// Poziadavka na vytvorenie noveho bankoveho uctu
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    /// Identifikator pouzivatela, pre ktoreho sa ma vytvorit ucet
    pub user_id: Uuid,
}

/// Poziadavka na registraciu noveho pouzivatela
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Pozadovane pouzivatelske meno (musi byt unikatne)
    pub username: String,
    /// Heslo v plain texte (bude zahashovane pred ulozenim)
    pub password: String,
}

/// Poziadavka na prihlasenie pouzivatela
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Pouzivatelske meno
    pub username: String,
    /// Heslo v plain texte
    pub password: String,
}

/// Transakcia medzi dvoma uctami
/// Reprezentuje prevod penazi s casovou peciatkou
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// Unikatny identifikator transakcie
    pub id: Uuid,
    /// Identifikator uctu odosielatela
    pub from_account: Uuid,
    /// Identifikator uctu prijemcu
    pub to_account: Uuid,
    /// Suma prevodu (presne desatinne cislo)
    pub amount: Decimal,
    /// Cas vytvorenia transakcie
    pub created_at: Option<NaiveDateTime>,
}

/// Poziadavka na vytvorenie transakcie (prevod penazi)
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionRequest {
    /// Identifikator uctu, z ktoreho sa budu peniaze odcitat
    pub from_account: Uuid,
    /// Identifikator uctu, na ktory sa budu peniaze pridat
    pub to_account: Uuid,
    /// Suma prevodu (musi byt kladna)
    pub amount: Decimal,
}

/// Poziadavka na pridanie penazi na ucet
/// Pouziva sa pri vkladoch penazi
#[derive(Debug, Deserialize)]
pub struct AddMoneyRequest {
    /// Identifikator uctu, na ktory sa maju pridat peniaze
    pub account_id: Uuid,
    /// Suma, ktora sa ma pridat (musi byt kladna)
    pub amount: Decimal,
}
