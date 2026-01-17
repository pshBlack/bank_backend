use axum::{
    Router,
    extract::{Json, Path},
    http::StatusCode,
    routing::{delete, get, post},
};
use bank_backend::*;
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/register", post(create_user_handler))
        .route("/login", post(login_user_handler))
        .route("/users/:id", get(get_user_handler))
        // .route("/users/:id", put(update_user_handler))
        .route("/users/:id", delete(delete_user_handler))
        .route("/accounts", post(create_account_handler))
        .route("/accounts/:id", get(get_account_handler))
        .route(
            "/accounts/:id/transactions",
            get(get_transaction_history_handler),
        )
        .route("/transactions", post(make_transaction_handler))
        .route("/addmoney", post(add_money_handler));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_user_handler(
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match create_user(&payload.username, &payload.password).await {
        Ok(user) => Ok(Json(json!(user))),
        Err(_e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Failed to create user"})),
        )),
    }
}

async fn get_user_handler(
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match get_user(user_id).await {
        Ok(user) => Ok(Json(json!(user))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )),
    }
}
async fn delete_user_handler(
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match delete_user(user_id).await {
        Ok(rows) if rows > 0 => Ok(Json(json!({"message": "User deleted"}))),
        Ok(_) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete user"})),
        )),
    }
}

async fn create_account_handler(
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match create_account(payload.user_id).await {
        Ok(account) => Ok(Json(json!(account))),
        Err(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Failed to create account"})),
        )),
    }
}

async fn get_account_handler(
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match get_account(user_id).await {
        Ok(account) => Ok(Json(json!(account))),
        Err(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Failed to get account"})),
        )),
    }
}

async fn add_money_handler(
    Json(payload): Json<AddMoneyRequest>,
) -> impl axum::response::IntoResponse {
    match add_money(payload.account_id, payload.amount).await {
        Ok(account) => axum::response::Json(json!(account)),
        Err(_e) => axum::response::Json(json!({"error": "Failed to add money"})),
    }
}

async fn make_transaction_handler(
    Json(payload): Json<TransactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match make_transaction(payload.from_account, payload.to_account, payload.amount).await {
        Ok(transaction) => Ok(Json(json!(transaction))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

async fn login_user_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match login_user(&payload.username, &payload.password).await {
        Ok(user) => {
            let accounts = get_account(user.id).await.unwrap_or_default();
            Ok(Json(json!({
                "user": user,
                "accounts": accounts
            })))
        }
        Err(e) => Err((StatusCode::UNAUTHORIZED, Json(json!({"error": e})))),
    }
}

async fn get_transaction_history_handler(
    Path(account_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match get_transaction_history(account_id).await {
        Ok(transactions) => Ok(Json(json!(transactions))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )),
    }
}
