use crate::config::Config;
use crate::db::Database;
use crate::handlers::{
    AppState, export_logs, export_traces, get_metadata, query_logs, query_records,
    query_span_events, query_spans,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{Span, error, info};

use crate::web;

/// Construct the Axum router with all routes and middleware
pub fn create_router(db: Database) -> Router {
    let state = Arc::new(AppState::new(db));

    // Create a permissive CORS layer for development
    let cors = if cfg!(debug_assertions) {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
    };

    Router::new()
        .route("/v1/logs", post(export_logs))
        .route("/v1/traces", post(export_traces))
        .route("/api/v1/logs", get(query_logs))
        .route("/api/v1/spans", get(query_spans))
        .route("/api/v1/events", get(query_span_events))
        .route("/api/v1/records", get(query_records))
        .route("/api/v1/metadata", get(get_metadata))
        .with_state(state)
        .fallback_service(web::EmbedServer::<web::Assets>::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http()
            .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
                tracing::debug_span!("http-request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version())
            })
            .on_request(|_: &axum::http::Request<axum::body::Body> , _: &Span| {
                tracing::debug!("started processing request");
            })
            .on_response(|response: &axum::http::Response<axum::body::Body>, latency: Duration, _span: &Span| {
                if response.status().is_success() {
                    tracing::debug!(status=%response.status(), latency=%format!("{} ms", latency.as_millis()), "finished processing request");
                } else {
                    tracing::warn!(status=%response.status(), latency=%format!("{} ms", latency.as_millis()), "failed to process request");
                }
            } ))
}

/// Spawn a background task that periodically cleans up old records
pub fn spawn_cleanup_task(db: Database, config: &Config) -> tokio::task::JoinHandle<()> {
    let cleanup_db = db.clone();
    let max_retention = config.max_retention_seconds;
    let cleanup_interval = config.cleanup_interval_seconds;
    let max_logs = config.max_logs;
    let max_spans = config.max_spans;

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(cleanup_interval));
        loop {
            interval.tick().await;

            if let Err(e) = cleanup_db.cleanup_old_records(max_retention).await {
                error!("Cleanup by age error: {}", e);
            } else {
                info!("Cleaned up old records by age");
            }

            if let (Some(max_logs), Some(max_spans)) = (max_logs, max_spans) {
                if let Err(e) = cleanup_db.cleanup_by_count(max_logs, max_spans).await {
                    error!("Cleanup by count error: {}", e);
                } else {
                    info!(
                        "Cleaned up records by count (max_logs={}, max_spans={})",
                        max_logs, max_spans
                    );
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_construction() {
        let db = Database::in_memory().await.unwrap();
        let router = create_router(db);

        // Just verify it compiles and constructs without panicking
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_cleanup_task_starts() {
        let db = Database::in_memory().await.unwrap();
        let config = Config::default();

        let handle = spawn_cleanup_task(db, &config);

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Abort the task
        handle.abort();
    }
}
