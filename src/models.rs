use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

/// Модель користувача для відповіді API (без пароля)
#[derive(Debug, Clone, Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PubAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: Decimal,
}
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub from_account: Uuid,
    pub to_account: Uuid,
    pub amount: Decimal,
    pub created_at: Option<NaiveDateTime>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionRequest {
    pub from_account: Uuid,
    pub to_account: Uuid,
    pub amount: Decimal,
}
#[derive(Debug, Deserialize)]
pub struct AddMoneyRequest {
    pub account_id: Uuid,
    pub amount: Decimal,
}
