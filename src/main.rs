use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::env;
use sea_orm::*;
use std::time::Duration;

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
       .acquire_timeout(Duration::from_secs(8));


    let db = Database::connect(opt)
        .await
        .expect("Failed to load database");
    
    check(&db).await;

    let app = Router::new()
        .route("/", get(hello))
    ;


    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}