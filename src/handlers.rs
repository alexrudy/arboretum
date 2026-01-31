use crate::db::{Database, LogFilter, LogRecord, SpanFilter, SpanRecord};
use crate::level::Level;
use crate::otlp::{convert_otlp_logs, convert_otlp_traces};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

pub async fn export_logs(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    let request: ExportLogsServiceRequest = prost::Message::decode(body)
        .map_err(|e| AppError::BadRequest(format!("Failed to decode protobuf: {}", e)))?;

    let logs = convert_otlp_logs(request);

    for log in logs {
        state
            .db
            .insert_log(log)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    Ok(StatusCode::OK)
}

pub async fn export_traces(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<StatusCode, AppError> {
    let request: ExportTraceServiceRequest = prost::Message::decode(body)
        .map_err(|e| AppError::BadRequest(format!("Failed to decode protobuf: {}", e)))?;

    let spans = convert_otlp_traces(request);

    for span in spans {
        state
            .db
            .insert_span(span)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub service_name: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub span_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub timestamp: i64,
    pub service_name: Option<String>,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<LogRecord> for LogResponse {
    fn from(log: LogRecord) -> Self {
        Self {
            timestamp: log.timestamp,
            service_name: log.service_name,
            level: log.level,
            target: log.target,
            message: log.message,
            span_id: log.span_id,
            trace_id: log.trace_id,
            attributes: log.attributes.and_then(|a| serde_json::from_str(&a).ok()),
        }
    }
}

pub async fn query_logs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<Vec<LogResponse>>, AppError> {
    let level = params
        .level
        .as_ref()
        .map(|s| s.parse::<Level>())
        .transpose()
        .map_err(|e| AppError::BadRequest(format!("Invalid level: {}", e)))?;

    let filter = LogFilter {
        service_name: params.service_name,
        level,
        target: params.target,
        message: params.message,
        span_id: params.span_id,
        limit: params.limit,
    };

    let logs = state
        .db
        .query_logs(filter)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let response: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct SpanQueryParams {
    pub service_name: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SpanResponse {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: Option<String>,
    pub name: String,
    pub kind: Option<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub events: Option<serde_json::Value>,
    pub status: Option<String>,
}

impl From<SpanRecord> for SpanResponse {
    fn from(span: SpanRecord) -> Self {
        Self {
            trace_id: span.trace_id,
            span_id: span.span_id,
            parent_span_id: span.parent_span_id,
            service_name: span.service_name,
            name: span.name,
            kind: span.kind,
            start_time: span.start_time,
            end_time: span.end_time,
            level: span.level,
            target: span.target,
            attributes: span.attributes.and_then(|a| serde_json::from_str(&a).ok()),
            events: span.events.and_then(|e| serde_json::from_str(&e).ok()),
            status: span.status,
        }
    }
}

pub async fn query_spans(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SpanQueryParams>,
) -> Result<Json<Vec<SpanResponse>>, AppError> {
    let level = params
        .level
        .as_ref()
        .map(|s| s.parse::<Level>())
        .transpose()
        .map_err(|e| AppError::BadRequest(format!("Invalid level: {}", e)))?;

    let filter = SpanFilter {
        service_name: params.service_name,
        level,
        target: params.target,
        span_id: params.span_id,
        trace_id: params.trace_id,
        limit: params.limit,
    };

    let spans = state
        .db
        .query_spans(filter)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let response: Vec<SpanResponse> = spans.into_iter().map(SpanResponse::from).collect();

    Ok(Json(response))
}

pub async fn get_metadata(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::db::DatabaseStats>, AppError> {
    let stats = state
        .db
        .get_stats()
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    Ok(Json(stats))
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[tokio::test]
    async fn test_query_logs_endpoint() {
        let db = Database::in_memory().await.unwrap();
        let state = Arc::new(AppState::new(db.clone()));

        let log = crate::db::LogRecord {
            timestamp: 1234567890,
            service_name: Some("test-service".to_string()),
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            message: Some("Test message".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        db.insert_log(log).await.unwrap();

        let params = LogQueryParams {
            service_name: Some("test-service".to_string()),
            level: None,
            target: None,
            message: None,
            span_id: None,
            limit: None,
        };

        let result = query_logs(State(state), Query(params)).await.unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].service_name, Some("test-service".to_string()));
    }

    #[tokio::test]
    async fn test_query_spans_endpoint() {
        let db = Database::in_memory().await.unwrap();
        let state = Arc::new(AppState::new(db.clone()));

        let span = crate::db::SpanRecord {
            trace_id: "abc123".to_string(),
            span_id: "def456".to_string(),
            parent_span_id: None,
            service_name: Some("test-service".to_string()),
            name: "test_span".to_string(),
            kind: Some("internal".to_string()),
            start_time: 1234567890,
            end_time: Some(1234567900),
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: None,
            events: None,
            status: None,
        };

        db.insert_span(span).await.unwrap();

        let params = SpanQueryParams {
            service_name: Some("test-service".to_string()),
            level: None,
            target: None,
            span_id: None,
            trace_id: None,
            limit: None,
        };

        let result = query_spans(State(state), Query(params)).await.unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].name, "test_span");
    }
}
