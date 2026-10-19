use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::env;
//use sea_orm::*;


async fn hello() -> Json<Value> {
    println!("Hello, world!");
    Json(json!({ "message": "hello world" }))
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let app = Router::new()
        .route("/", get(hello))
    ;


    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();
    println!("Listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}