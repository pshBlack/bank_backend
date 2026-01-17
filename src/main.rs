use axum::{
    Router,
    extract::{Json, Path},
    http::StatusCode,
    routing::{delete, get, post},
};
use bank_backend::*;
use serde_json::json;
use uuid::Uuid;

/// Hlavna funkcia - spustenie HTTP servera
/// Server bezi na adrese 127.0.0.1:3000 a poskytuje REST API pre bankovy system
#[tokio::main]
async fn main() {
    // Konfigurovanie routing pre REST API endpointy
    let app = Router::new()
        // Registracia noveho pouzivatela
        .route("/register", post(create_user_handler))
        // Prihlasenie existujuceho pouzivatela
        .route("/login", post(login_user_handler))
        // Ziskanie informacii o pouzivatelovi podla ID
        .route("/users/:id", get(get_user_handler))
        // Zmazanie pouzivatela podla ID
        .route("/users/:id", delete(delete_user_handler))
        // Vytvorenie noveho bankoveho uctu
        .route("/accounts", post(create_account_handler))
        // Ziskanie informacii o ucte podla ID
        .route("/accounts/:id", get(get_account_handler))
        // Ziskanie historie transakci pre dany ucet
        .route(
            "/accounts/:id/transactions",
            get(get_transaction_history_handler),
        )
        // Vytvorenie novej transakcie (prevod penazi)
        .route("/transactions", post(make_transaction_handler))
        // Pridanie penazi na ucet
        .route("/addmoney", post(add_money_handler));

    // Spustenie HTTP servera na porte 3000
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handler pre registraciu noveho pouzivatela
///
/// # Endpoint
/// POST /register
///
/// # Vstupy
/// - username: pouzivatelske meno
/// - password: heslo (bude zahashovane)
///
/// # Vystupy
/// - 200 OK: uspesne vytvoreny pouzivatel (vracia PublicUser)
/// - 400 Bad Request: chyba pri vytvarani (napr. uz existuje)
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

/// Handler pre ziskanie informacii o pouzivatelovi
///
/// # Endpoint
/// GET /users/:id
///
/// # Parametre
/// - id: UUID pouzivatela
///
/// # Vystupy
/// - 200 OK: uspesne ziskane udaje (vracia PublicUser)
/// - 404 Not Found: pouzivatel neexistuje
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

/// Handler pre zmazanie pouzivatela
///
/// # Endpoint
/// DELETE /users/:id
///
/// # Parametre
/// - id: UUID pouzivatela
///
/// # Vystupy
/// - 200 OK: pouzivatel uspesne zmazany
/// - 404 Not Found: pouzivatel neexistuje
/// - 500 Internal Server Error: chyba pri mazani
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

/// Handler pre vytvorenie noveho bankoveho uctu
///
/// # Endpoint
/// POST /accounts
///
/// # Vstupy
/// - user_id: UUID pouzivatela, pre ktoreho sa ma ucet vytvorit
///
/// # Vystupy
/// - 200 OK: ucet uspesne vytvoreny (vracia PubAccount)
/// - 400 Bad Request: chyba pri vytvarani uctu
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

/// Handler pre ziskanie informacii o ucte
///
/// # Endpoint
/// GET /accounts/:id
///
/// # Parametre
/// - id: UUID uctu alebo pouzivatela
///
/// # Vystupy
/// - 200 OK: uspesne ziskane udaje o ucte(och)
/// - 400 Bad Request: chyba pri ziskavani udajov
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

/// Handler pre pridanie penazi na ucet
///
/// # Endpoint
/// POST /addmoney
///
/// # Vstupy
/// - account_id: UUID uctu
/// - amount: suma na pridanie (musi byt kladna)
///
/// # Vystupy
/// - 200 OK: peniaze uspesne pridane (vracia aktualizovany PubAccount)
/// - Chybova odpoved: nepodarilo sa pridat peniaze
async fn add_money_handler(
    Json(payload): Json<AddMoneyRequest>,
) -> impl axum::response::IntoResponse {
    match add_money(payload.account_id, payload.amount).await {
        Ok(account) => axum::response::Json(json!(account)),
        Err(_e) => axum::response::Json(json!({"error": "Failed to add money"})),
    }
}

/// Handler pre vytvorenie transakcie (prevod penazi medzi uctami)
///
/// # Endpoint
/// POST /transactions
///
/// # Vstupy
/// - from_account: UUID uctu odosielatela
/// - to_account: UUID uctu prijemcu
/// - amount: suma prevodu (musi byt kladna)
///
/// # Validacie
/// - Overuje ci ma odosielatel dostatocny zostatok
/// - Zabranuje prevodu na ten isty ucet
/// - Pouziva databazovu transakciu pre ACID vlastnosti
///
/// # Vystupy
/// - 200 OK: transakcia uspesne vytvorena (vracia Transaction)
/// - 400 Bad Request: nedostatocny zostatok, neplatne ucty, atd.
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

/// Handler pre prihlasenie pouzivatela
///
/// # Endpoint
/// POST /login
///
/// # Vstupy
/// - username: pouzivatelske meno
/// - password: heslo
///
/// # Vystupy
/// - 200 OK: uspesne prihlasenie (vracia pouzivatela a jeho ucty)
/// - 401 Unauthorized: nespravne prihlasovacie udaje
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

/// Handler pre ziskanie historie transakci uctu
///
/// # Endpoint
/// GET /accounts/:id/transactions
///
/// # Parametre
/// - id: UUID uctu
///
/// # Vystupy
/// - 200 OK: zoznam vsetkych transakci (odosielatel alebo prijemca)
/// - 400 Bad Request: chyba pri ziskavani transakci
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
