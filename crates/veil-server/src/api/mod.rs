use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RuleRow {
    id: String,
    pattern: String,
    description: Option<String>,
    severity: String,
}

pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "version": "0.1.0" })),
    )
}

pub async fn get_rules(State(pool): State<PgPool>) -> (StatusCode, Json<Vec<RuleRow>>) {
    let rules = sqlx::query_as::<_, RuleRow>(
        "SELECT id, pattern, description, severity FROM rules WHERE enabled = true",
    )
    .fetch_all(&pool)
    .await;

    match rules {
        Ok(data) => (StatusCode::OK, Json(data)),
        Err(e) => {
            tracing::error!("Database query error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}
