use aws_sdk_dynamodb::types::AttributeValue;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone)]
struct AppState {
    dynamo: aws_sdk_dynamodb::Client,
    table_name: String,
}

#[derive(Serialize, Deserialize)]
struct Word {
    id: u64,
    word: String,
}

#[derive(Deserialize)]
struct UpdateWordRequest {
    word: String,
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    tracing_subscriber::fmt().json().with_target(false).init();

    let table_name = env::var("DYNAMODB_TABLE_NAME").expect("DYNAMODB_TABLE_NAME must be set");
    let endpoint = env::var("DYNAMODB_ENDPOINT").ok();

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;

    let mut dynamo_config = aws_sdk_dynamodb::config::Builder::from(&config);
    if let Some(ref ep) = endpoint {
        dynamo_config = dynamo_config.endpoint_url(ep);
    }

    let state = AppState {
        dynamo: aws_sdk_dynamodb::Client::from_conf(dynamo_config.build()),
        table_name,
    };

    let app = Router::new()
        .route("/api", get(health))
        .route("/api/word", get(get_word).put(update_word))
        .with_state(state);

    lambda_http::run(app).await
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn get_word(State(state): State<AppState>) -> Result<Json<Word>, AppError> {
    let pk = "WORD#1".to_string();

    let result = state
        .dynamo
        .get_item()
        .table_name(&state.table_name)
        .key("PK", AttributeValue::S(pk.clone()))
        .key("SK", AttributeValue::S(pk))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("DynamoDB get_item error: {}", e);
            AppError::internal(e)
        })?;

    let item = result.item().ok_or(AppError::not_found("Word not found"))?;
    let word = item
        .get("word")
        .and_then(|v| v.as_s().ok())
        .ok_or(AppError::internal("Missing 'word' attribute"))?;

    Ok(Json(Word { id: 1, word: word.clone() }))
}

async fn update_word(
    State(state): State<AppState>,
    Json(body): Json<UpdateWordRequest>,
) -> Result<Json<Word>, AppError> {
    if body.word.is_empty() {
        return Err(AppError(StatusCode::BAD_REQUEST, "'word' must not be empty".into()));
    }

    let pk = "WORD#1".to_string();

    state
        .dynamo
        .put_item()
        .table_name(&state.table_name)
        .item("PK", AttributeValue::S(pk.clone()))
        .item("SK", AttributeValue::S(pk))
        .item("word", AttributeValue::S(body.word.clone()))
        .item("GSI1PK", AttributeValue::S("WORDS".into()))
        .item("GSI1SK", AttributeValue::S("WORD#1".into()))
        .send()
        .await
        .map_err(AppError::internal)?;

    Ok(Json(Word { id: 1, word: body.word }))
}

struct AppError(StatusCode, String);

impl AppError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into())
    }
    fn internal(msg: impl std::fmt::Display) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}
