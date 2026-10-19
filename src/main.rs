use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
//use sea_orm::*;


async fn hello() -> Json<Value> {
    println!("Hello, world!");
    Json(json!({ "message": "hello world" }))
}


#[tokio::main]
async fn main() {
    const PORT: i32 = 3000;
    let mut app = Router::new().route("/hello", get(hello));
    app = app.route("/", get(hello));

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", PORT)).await.unwrap();
    println!("Listening on port {}", PORT);
    axum::serve(listener, app).await.unwrap();
}