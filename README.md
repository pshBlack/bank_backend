# Bank Backend API

Minimalistický bankový REST API server napísaný v jazyku Rust s použitím frameworku Axum a PostgreSQL databázy.

## 📋 Obsah

- [Funkcie](#funkcie)
- [Technológie](#technológie)
- [Požiadavky](#požiadavky)
- [Inštalácia](#inštalácia)
- [Konfigurácia databázy](#konfigurácia-databázy)
- [Spustenie](#spustenie)
- [API Endpoints](#api-endpoints)
- [Príklady použitia](#príklady-použitia)
- [Štruktúra projektu](#štruktúra-projektu)

## ✨ Funkcie

- ✅ Registrácia a prihlásenie používateľov
- ✅ Správa bankových účtov
- ✅ Prevody medzi účtami
- ✅ História transakcií
- ✅ Asynchronné spracovanie
- ✅ Bezpečné hashovanie hesiel (Argon2/Bcrypt)
- ✅ Transakcie v databáze (ACID)
- ✅ REST API s HTTP status kódmi

## 🛠 Technológie

- **Rust** - programovací jazyk
- **Axum** - webový framework
- **SQLx** - asynchronná práca s databázou
- **PostgreSQL** - relačná databáza
- **Tokio** - asynchronný runtime
- **Serde** - serializácia/deserializácia JSON
- **Argon2/Bcrypt** - hashovanie hesiel
- **rust_decimal** - presné operácie s desatinnými číslami

## 📦 Požiadavky

- Rust 1.70+ ([inštalácia](https://www.rust-lang.org/tools/install))
- PostgreSQL 14+ ([inštalácia](https://www.postgresql.org/download/))
- Cargo (nainštalovaný s Rust)

## 🚀 Inštalácia

### 1. Klonovanie repozitára
```bash
git clone 
cd bank_backend
```

### 2. Inštalácia závislostí
```bash
cargo build
```

## 🗄️ Konfigurácia databázy

### 1. Vytvorenie databázy
```bash
# Prihlásenie do PostgreSQL
psql -U postgres

# Vytvorenie databázy
CREATE DATABASE bank_db;

# Pripojenie k databáze
\c bank_db
```

### 2. Vytvorenie tabuliek
```sql
-- Povolenie UUID rozšírenia
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tabuľka používateľov
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Tabuľka účtov
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    balance NUMERIC(15, 2) DEFAULT 0.00 NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Tabuľka transakcií
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_account UUID NOT NULL REFERENCES accounts(id),
    to_account UUID NOT NULL REFERENCES accounts(id),
    amount NUMERIC(15, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexy pre rýchlejšie vyhľadávanie
CREATE INDEX idx_accounts_user_id ON accounts(user_id);
CREATE INDEX idx_transactions_from ON transactions(from_account);
CREATE INDEX idx_transactions_to ON transactions(to_account);
```

### 3. Konfigurácia pripojenia

Upravte súbor `src/db.rs` a nastavte svoje údaje pre pripojenie:
```rust
pub async fn create_pool() -> PgPool {
    let database_url = "postgresql://postgres:heslo@localhost/bank_db";
    PgPool::connect(database_url).await.expect("Failed to connect to database")
}
```

Alebo vytvorte súbor `.env`:
```bash
DATABASE_URL=postgresql://postgres:heslo@localhost/bank_db
```

## ▶️ Spustenie
```bash
# Vývoj (s debug informáciami)
cargo run

# Produkcia (optimalizovaný build)
cargo build --release
./target/release/bank_backend
```

Server bude bežať na `http://127.0.0.1:3000`

## 📡 API Endpoints

### Používatelia

| Metóda | Endpoint | Popis |
|--------|----------|-------|
| `POST` | `/register` | Registrácia nového používateľa |
| `POST` | `/login` | Prihlásenie používateľa |
| `GET` | `/users/:id` | Získanie informácií o používateľovi |
| `DELETE` | `/users/:id` | Zmazanie používateľa |

### Účty

| Metóda | Endpoint | Popis |
|--------|----------|-------|
| `POST` | `/accounts` | Vytvorenie nového účtu |
| `GET` | `/accounts/:id` | Informácie o účte |
| `GET` | `/users/:id/accounts` | Všetky účty používateľa |

### Transakcie

| Metóda | Endpoint | Popis |
|--------|----------|-------|
| `POST` | `/transactions` | Prevod medzi účtami |
| `GET` | `/accounts/:id/transactions` | História transakcií účtu |
| `POST` | `/addmoney` | Pridanie peňazí na účet |

## 💡 Príklady použitia

### Registrácia používateľa
```bash
curl -X POST http://127.0.0.1:3000/register \
  -H "Content-Type: application/json" \
  -d '{"username": "jan_novak", "password": "bezpecne_heslo123"}'
```

**Odpoveď:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "jan_novak"
}
```

### Prihlásenie
```bash
curl -X POST http://127.0.0.1:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username": "jan_novak", "password": "bezpecne_heslo123"}'
```

**Odpoveď:**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "jan_novak"
  },
  "accounts": []
}
```

### Vytvorenie účtu
```bash
curl -X POST http://127.0.0.1:3000/accounts \
  -H "Content-Type: application/json" \
  -d '{"user_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

**Odpoveď:**
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "balance": "0.00"
}
```

### Pridanie peňazí
```bash
curl -X POST http://127.0.0.1:3000/addmoney \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": "660e8400-e29b-41d4-a716-446655440001",
    "amount": "1000.00"
  }'
```

**Odpoveď:**
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "balance": "1000.00"
}
```

### Prevod medzi účtami
```bash
curl -X POST http://127.0.0.1:3000/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "from_account": "660e8400-e29b-41d4-a716-446655440001",
    "to_account": "770e8400-e29b-41d4-a716-446655440002",
    "amount": "250.50"
  }'
```

**Odpoveď:**
```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "from_account": "660e8400-e29b-41d4-a716-446655440001",
  "to_account": "770e8400-e29b-41d4-a716-446655440002",
  "amount": "250.50",
  "created_at": "2026-01-17T14:30:00"
}
```

### História transakcií
```bash
curl http://127.0.0.1:3000/accounts/660e8400-e29b-41d4-a716-446655440001/transactions
```

**Odpoveď:**
```json
[
  {
    "id": "880e8400-e29b-41d4-a716-446655440003",
    "from_account": "660e8400-e29b-41d4-a716-446655440001",
    "to_account": "770e8400-e29b-41d4-a716-446655440002",
    "amount": "250.50",
    "created_at": "2026-01-17T14:30:00"
  }
]
```

## 📁 Štruktúra projektu
```
bank_backend/
├── src/
│   ├── lib.rs              # Knižnica (exportuje moduly)
│   ├── main.rs             # Spustiteľný súbor (REST API handlers)
│   ├── crud.rs             # CRUD operácie (databázová logika)
│   ├── db.rs               # Pripojenie k databáze
│   └── models.rs           # Dátové modely a štruktúry
├── Cargo.toml              # Závislosti a konfigurácia projektu
├── Cargo.lock              # Zamknuté verzie závislostí
└── README.md               # Dokumentácia
```

### Popis súborov

- **lib.rs** - Hlavná knižnica exportujúca všetky moduly
- **main.rs** - REST API server a HTTP handlery
- **crud.rs** - Funkcie pre prácu s databázou (create, read, update, delete)
- **db.rs** - Konfigurácia a vytvorenie connection pool
- **models.rs** - Dátové štruktúry (User, Account, Transaction, atď.)
