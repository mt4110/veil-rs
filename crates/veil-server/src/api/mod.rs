use crate::db::{Error, FromRow, PgPool, PgRow, Row};
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RuleRow {
    id: String,
    pattern: String,
    description: Option<String>,
    severity: String,
}

impl<'r> FromRow<'r, PgRow> for RuleRow {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        Ok(Self {
            id: row.try_get("id")?,
            pattern: row.try_get("pattern")?,
            description: row.try_get("description")?,
            severity: row.try_get("severity")?,
        })
    }
}

pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "version": "0.1.0" })),
    )
}

pub async fn get_rules(State(pool): State<PgPool>) -> (StatusCode, Json<Vec<RuleRow>>) {
    let rules = crate::db::query_as::<_, RuleRow>(
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
