use argon2::{
    password_hash::phc::PasswordHash,
    Argon2,
    PasswordHasher,
    PasswordVerifier,
};
use axum::{
    extract::FromRequestParts,
    http::{
        header::AUTHORIZATION,
        request::Parts,
        StatusCode,
    },
    Json,
};
use rand::Rng;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::models::{session, user};

type ApiError = (StatusCode, Json<Value>);

/// A user resolved from a valid session token.
pub struct AuthUser {
    pub id: i64,
    pub username: String,
}

pub fn json_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

pub fn hash_password(password: &str) -> String {
    // argon2 generates a random salt internally
    Argon2::default()
        .hash_password(password.as_bytes())
        .expect("Failed to hash password")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time is before the Unix epoch")
        .as_secs() as i64
}

impl FromRequestParts<DatabaseConnection> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &DatabaseConnection,
    ) -> Result<Self, Self::Rejection> {
        let Some(header) = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Err(json_error(StatusCode::UNAUTHORIZED, "Missing Authorization header."));
        };
        let Some(token) = header.strip_prefix("Bearer ").map(str::trim) else {
            return Err(json_error(
                StatusCode::UNAUTHORIZED,
                "Authorization header must be \"Bearer <token>\".",
            ));
        };

        let session = session::Entity::find()
            .filter(session::Column::Id.eq(token.to_string()))
            .filter(session::Column::ExpiresAt.gt(now_unix()))
            .one(state)
            .await
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load session."))?;

        let Some(session) = session else {
            return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid or expired session."));
        };

        let user = user::Entity::find_by_id(session.user_id)
            .one(state)
            .await
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load user."))?;

        match user {
            Some(user) => Ok(AuthUser {
                id: user.id,
                username: user.username,
            }),
            None => Err(json_error(StatusCode::UNAUTHORIZED, "Invalid or expired session.")),
        }
    }
}
