use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::{
    AuthUser, generate_token, hash_password, json_error, now_unix, verify_password,
};
use crate::models::{session, user};

const SESSION_LIFETIME_SECONDS: i64 = 7 * 24 * 60 * 60;

type ApiError = (StatusCode, Json<Value>);

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

fn validate_credentials(username: &str, password: &str) -> Result<(), ApiError> {
    if username.len() < 3 || username.len() > 32 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Username must be between 3 and 32 characters.",
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Username may only contain letters, digits, underscores, dots and hyphens.",
        ));
    }
    if password.len() < 8 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters.",
        ));
    }
    if password.len() > 72 {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "Password must be at most 72 characters.",
        ));
    }
    Ok(())
}

pub async fn register(
    State(db): State<DatabaseConnection>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let username = req.username.trim();
    validate_credentials(username, &req.password)?;

    let active = user::ActiveModel {
        username: Set(username.to_string()),
        password_hash: Set(hash_password(&req.password)),
        ..Default::default()
    };

    match user::Entity::insert(active).exec(&db).await {
        Ok(result) => {
            let created = user::Entity::find_by_id(result.last_insert_id)
                .one(&db)
                .await
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load created user."))?;
            let created = match created {
                Some(created) => created,
                None => {
                    return Err(json_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to load created user.",
                    ));
                }
            };
            Ok((
                StatusCode::CREATED,
                Json(json!({
                    "id": created.id,
                    "username": created.username,
                    "createdAt": created.created_at,
                })),
            ))
        }
        Err(err) => {
            if err.to_string().contains("UNIQUE") {
                Err(json_error(
                    StatusCode::CONFLICT,
                    "That username is already taken.",
                ))
            } else {
                Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Registration failed."))
            }
        }
    }
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let username = req.username.trim();

    let found = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(&db)
        .await
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Login failed."))?;

    let Some(found) = found else {
        return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid username or password."));
    };
    if !verify_password(&req.password, &found.password_hash) {
        return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid username or password."));
    }

    let token = generate_token();
    let created_at = now_unix();
    let expires_at = created_at + SESSION_LIFETIME_SECONDS;

    session::Entity::insert(session::ActiveModel {
        id: Set(token.clone()),
        user_id: Set(found.id),
        created_at: Set(created_at),
        expires_at: Set(expires_at),
    })
    .exec(&db)
    .await
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session."))?;

    Ok(Json(json!({ "token": token, "expiresAt": expires_at })))
}

pub async fn me(user: AuthUser) -> Json<Value> {
    Json(json!({ "id": user.id, "username": user.username }))
}
