use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, patch, post, put},
    Router,
};
use chrono::Utc;
use serde_json::json;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::models::{
    RegisterUserRequest, SetPreferenceRequest, SetVerificationRequest, UpdateProfileRequest,
    UserAnalyticsRow, UserPreference, UserProfile, UserSearchQuery,
    UserAnalyticsRow, UserPreference, UserProfile,
};

/// POST /users — register a new user profile
pub async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<RegisterUserRequest>,
) -> Result<(StatusCode, Json<UserProfile>), AppError> {
    let now = Utc::now().timestamp();
    let profile = sqlx::query_as::<_, UserProfile>(
        r#"
        INSERT INTO user_profiles (address, username_hash, contact_hash, registered_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        ON CONFLICT (address) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(&req.address)
    .bind(&req.username_hash)
    .bind(&req.contact_hash)
    .bind(now)
    .fetch_optional(state.database.pool())
    .await?
    .ok_or_else(|| AppError::Conflict("User already registered".into()))?;

    Ok((StatusCode::CREATED, Json(profile)))
}

/// GET /users/:address — fetch a user profile
pub async fn get_user(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<UserProfile>, AppError> {
    let profile = sqlx::query_as::<_, UserProfile>(
        "SELECT * FROM user_profiles WHERE address = $1",
    )
    .bind(&address)
    .fetch_optional(state.database.pool())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(profile))
}

/// PATCH /users/:address — update profile hashes / avatar
pub async fn update_user(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AppError> {
    let now = Utc::now().timestamp();
    let profile = sqlx::query_as::<_, UserProfile>(
        r#"
        UPDATE user_profiles
        SET
            username_hash = COALESCE($2, username_hash),
            contact_hash  = COALESCE($3, contact_hash),
            avatar_hash   = COALESCE($4, avatar_hash),
            updated_at    = $5
        WHERE address = $1
        RETURNING *
        "#,
    )
    .bind(&address)
    .bind(&req.username_hash)
    .bind(&req.contact_hash)
    .bind(&req.avatar_hash)
    .bind(now)
    .fetch_optional(state.database.pool())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(profile))
}

/// PUT /users/:address/preferences — upsert a preference key/value
pub async fn set_preference(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<SetPreferenceRequest>,
) -> Result<Json<UserPreference>, AppError> {
    let pref = sqlx::query_as::<_, UserPreference>(
        r#"
        INSERT INTO user_preferences (address, key, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (address, key) DO UPDATE SET value = EXCLUDED.value
        RETURNING *
        "#,
    )
    .bind(&address)
    .bind(&req.key)
    .bind(&req.value)
    .fetch_one(state.database.pool())
    .await?;

    Ok(Json(pref))
}

/// GET /users/:address/preferences — list all preferences for a user
pub async fn get_preferences(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Vec<UserPreference>>, AppError> {
    let prefs = sqlx::query_as::<_, UserPreference>(
        "SELECT * FROM user_preferences WHERE address = $1 ORDER BY key",
    )
    .bind(&address)
    .fetch_all(state.database.pool())
    .await?;

    Ok(Json(prefs))
}

/// GET /users/:address/analytics — fetch trade analytics for a user
pub async fn get_user_analytics(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<UserAnalyticsRow>, AppError> {
    let row = sqlx::query_as::<_, UserAnalyticsRow>(
        "SELECT * FROM user_analytics WHERE address = $1",
    )
    .bind(&address)
    .fetch_optional(state.database.pool())
    .await?
    .unwrap_or_else(|| UserAnalyticsRow {
        address: address.clone(),
        total_trades: 0,
        trades_as_seller: 0,
        trades_as_buyer: 0,
        total_volume: 0,
        completed_trades: 0,
        disputed_trades: 0,
        cancelled_trades: 0,
        updated_at: Utc::now(),
    });

    Ok(Json(row))
}

/// PATCH /users/:address/verification — admin: set verification status
pub async fn set_verification(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Json(req): Json<SetVerificationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let valid = ["Unverified", "Pending", "Verified", "Rejected"];
    if !valid.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: {:?}",
            req.status, valid
        )));
    }

    let rows = sqlx::query(
        "UPDATE user_profiles SET verification = $2, updated_at = $3 WHERE address = $1",
    )
    .bind(&address)
    .bind(&req.status)
    .bind(Utc::now().timestamp())
    .execute(state.database.pool())
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }

    Ok(Json(json!({ "address": address, "verification": req.status })))
}

/// GET /users/search?q=…&limit=25&offset=0 — full-text search over user profiles
pub async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<UserSearchQuery>,
) -> Result<Json<Vec<UserProfile>>, AppError> {
    let q = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(25);
    let offset = params.offset.unwrap_or(0);

    if !q.is_empty() {
        state.database.record_search(&q, "users").await?;
    }

    let results = state.database.search_users(&q, limit, offset).await?;
    Ok(Json(results))
}
