use axum::{
    routing::{get, post},
    Json,
    Router,
};
use sea_orm::*;
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

mod auth;
mod db;
mod handlers;
mod models;

async fn hello() -> Json<Value> {
    println!("Hello, world!");
    Json(json!({ "message": "hello world" }))
}

async fn check(db: &DatabaseConnection) {
    db.ping().await.expect("Database ping failed");
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let db_path = env::var("SAPHIRE_DB_PATH").expect("SAPHIRE_DB_PATH env variable is missing.");

    let mut opt = ConnectOptions::new(format!("sqlite://{db_path}?mode=rwc"));
    opt.connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        // SQLite only enforces foreign keys when the pragma is set per connection
        .map_sqlx_sqlite_opts(|opts| opts.foreign_keys(true));

    let db = Database::connect(opt)
        .await
        .expect("Failed to load database");

    check(&db).await;
    db::migrate(&db).await.expect("Failed to create database tables");

    let app = Router::new()
        .route("/", get(hello))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/me", get(handlers::auth::me))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}
