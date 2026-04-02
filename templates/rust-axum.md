---
name: rust-axum
description: Rust web server with Axum and SQLx
language: rust
docker_image: rust:1.82-slim
variables:
  - name: project_name
    description: Name of the project
    required: true
tags: [backend, rust, api, async]
---

# {{ project_name }}

High-performance Rust web server using Axum framework and SQLx for database access.

## Project Structure

```
{{ project_name }}/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   └── health.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── queries.rs
│   ├── error.rs
│   └── config.rs
├── migrations/
│   └── 001_init.sql
├── Dockerfile
├── docker-compose.yml
└── .sqlx
```

## Files

### Cargo.toml
```toml
[package]
name = "{{ project_name }}"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"

[dev-dependencies]
tokio-test = "0.4"
```

### src/main.rs
```rust
#![deny(unsafe_code)]
#![warn(missing_docs)]

//! {{ project_name }} - Axum web server

mod config;
mod db;
mod error;
mod handlers;
mod models;

use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/users", post(handlers::users::create_user))
        .route("/users/:id", get(handlers::users::get_user))
        .route("/users/:id", put(handlers::users::update_user))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(pool));

    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

### src/config.rs
```rust
//! Configuration management

use std::env;

/// Application configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Server address
    pub server_addr: String,
    /// Database URL
    pub database_url: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
        }
    }
}
```

### src/error.rs
```rust
//! Error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Not found error
    #[error("Resource not found")]
    NotFound,
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred",
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, &msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### src/models.rs
```rust
//! Data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// User ID
    pub id: Uuid,
    /// Email address
    pub email: String,
    /// Username
    pub username: String,
    /// Full name
    pub full_name: Option<String>,
    /// Is active
    pub is_active: bool,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Email address
    pub email: String,
    /// Username
    pub username: String,
    /// Full name
    pub full_name: Option<String>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// Full name
    pub full_name: Option<String>,
    /// Is active
    pub is_active: Option<bool>,
}
```

### src/handlers/mod.rs
```rust
//! Request handlers

pub mod health;
pub mod users;
```

### src/handlers/health.rs
```rust
//! Health check handler

use axum::Json;
use serde_json::json;

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "{{ project_name }}"
    }))
}
```

### src/handlers/users.rs
```rust
//! User handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::Result,
    models::{CreateUserRequest, UpdateUserRequest, User},
};

/// Create a new user
pub async fn create_user(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>)> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, username, full_name, is_active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, NOW(), NOW())
         RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(&req.email)
    .bind(&req.username)
    .bind(&req.full_name)
    .fetch_one(pool.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// Get user by ID
pub async fn get_user(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await?
        .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(user))
}

/// Update user
pub async fn update_user(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET full_name = COALESCE($1, full_name),
         is_active = COALESCE($2, is_active),
         updated_at = NOW()
         WHERE id = $3
         RETURNING *"
    )
    .bind(&req.full_name)
    .bind(req.is_active)
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(user))
}
```

### migrations/001_init.sql
```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### Dockerfile
```dockerfile
FROM rust:1.82-slim as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/{{ project_name }} /usr/local/bin/

EXPOSE 8000

CMD ["{{ project_name }}"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
      POSTGRES_DB: {{ project_name }}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  api:
    build: .
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://user:password@postgres/{{ project_name }}
      SERVER_ADDR: 0.0.0.0:8000
      RUST_LOG: info
    depends_on:
      - postgres

volumes:
  postgres_data:
```

## Getting Started

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Set up environment:
   ```bash
   cp .env.example .env
   ```

3. Run with Docker Compose:
   ```bash
   docker-compose up
   ```

4. Or run locally:
   ```bash
   cargo run
   ```

5. Test the API:
   ```bash
   curl http://localhost:8000/health
   ```

## Environment Variables

Create a `.env` file:
```
DATABASE_URL=postgresql://user:password@localhost/{{ project_name }}
SERVER_ADDR=127.0.0.1:8000
RUST_LOG=info
```
