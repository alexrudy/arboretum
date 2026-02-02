use crate::db::{
    CommonFilters, CursorFilter, Database, LogFilter, LogRecord, SpanEventFilter, SpanEventRecord,
    SpanFilter, SpanRecord,
};
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
use tracing::{debug, error, warn};

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse {
    /// Records in this page
    pub records: Vec<RecordResponse>,
    /// Cursor for next page (if available)
    pub since: Option<i64>,
}

impl PaginatedResponse {
    fn from_records(mut records: Vec<RecordResponse>, max_records: usize) -> Self {
        records.truncate(max_records);
        let since = records.iter().map(|r| r.timestamp()).max();

        if records.is_empty() {
            warn!("No records to include");
        } else {
            debug!(cursor=%since.unwrap(), "fetched {} records", records.len());
        }

        Self { records, since }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub max_records_returned: usize,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            max_records_returned: 1000,
        }
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
    let request: ExportTraceServiceRequest = prost::Message::decode(body).map_err(|e| {
        debug!("Failed to decode protobuf: {}", e);
        AppError::BadRequest(format!("Failed to decode protobuf: {}", e))
    })?;

    let (spans, events) = convert_otlp_traces(request);

    debug!("recording {} spans {} events", spans.len(), events.len());

    for span in spans {
        state
            .db
            .insert_span(span)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    for event in events {
        state
            .db
            .insert_span_event(event)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    }

    Ok(StatusCode::OK)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanResponse {
    pub timestamp: i64,
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
            timestamp: span.start_time,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEventResponse {
    pub timestamp: i64,
    pub span_id: String,
    pub trace_id: String,
    pub service_name: Option<String>,
    pub name: String,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<SpanEventRecord> for SpanEventResponse {
    fn from(event: SpanEventRecord) -> Self {
        Self {
            timestamp: event.timestamp,
            span_id: event.span_id,
            trace_id: event.trace_id,
            service_name: event.service_name,
            name: event.name,
            level: event.level,
            target: event.target,
            attributes: event.attributes.and_then(|a| serde_json::from_str(&a).ok()),
        }
    }
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

#[derive(Debug, Default, Deserialize)]
pub struct RecordQueryParams {
    pub service_name: Option<String>,
    pub level: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub lookback: Option<i64>,
    pub since: Option<i64>,
    pub until: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RecordResponse {
    Log(LogResponse),
    Span(SpanResponse),
    Event(SpanEventResponse),
}

impl From<LogResponse> for RecordResponse {
    fn from(log: LogResponse) -> Self {
        RecordResponse::Log(log)
    }
}

impl From<SpanResponse> for RecordResponse {
    fn from(span: SpanResponse) -> Self {
        RecordResponse::Span(span)
    }
}

impl From<SpanEventResponse> for RecordResponse {
    fn from(event: SpanEventResponse) -> Self {
        RecordResponse::Event(event)
    }
}

/// A key for uniquely identifying and comparing records without allocations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum RecordKey<'a> {
    Log {
        timestamp: i64,
        trace_id: Option<&'a str>,
    },
    Span {
        span_id: &'a str,
    },
    Event {
        span_id: &'a str,
        timestamp: i64,
    },
}

impl RecordResponse {
    pub fn timestamp(&self) -> i64 {
        match self {
            RecordResponse::Log(log) => log.timestamp,
            RecordResponse::Span(span) => span.timestamp,
            RecordResponse::Event(event) => event.timestamp,
        }
    }

    /// Get a unique key for this record for deduplication and comparison
    fn record_key(&self) -> RecordKey<'_> {
        match self {
            RecordResponse::Log(log) => RecordKey::Log {
                timestamp: log.timestamp,
                trace_id: log.trace_id.as_deref(),
            },
            RecordResponse::Span(span) => RecordKey::Span {
                span_id: &span.span_id,
            },
            RecordResponse::Event(event) => RecordKey::Event {
                span_id: &event.span_id,
                timestamp: event.timestamp,
            },
        }
    }

    /// Compare two records for sorting (timestamp, then type, then unique key)
    pub fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.timestamp(), self.record_key()).cmp(&(other.timestamp(), other.record_key()))
    }
}

/// Deduplicate a sorted vector of records in-place
/// This is efficient because duplicates are adjacent after sorting
fn deduplicate_records(records: &mut Vec<RecordResponse>) {
    if records.is_empty() {
        return;
    }

    let mut write_idx = 0;

    for read_idx in 1..records.len() {
        // Compare keys without allocating strings
        if records[write_idx].record_key() != records[read_idx].record_key() {
            write_idx += 1;
            if write_idx != read_idx {
                records[write_idx] = records[read_idx].clone();
            }
        }
    }

    records.truncate(write_idx + 1);
}

pub async fn query_records(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecordQueryParams>,
) -> Result<Json<PaginatedResponse>, AppError> {
    let common = CommonFilters {
        service_name: params.service_name,
        level: params
            .level
            .map(|l| l.parse())
            .transpose()
            .map_err(|msg| AppError::BadRequest(format!("Invalid level: {msg}")))?,
        target: params.target,
        message: params.message,
        span_id: params.span_id,
        trace_id: params.trace_id,
    };
    let cursor = CursorFilter {
        lookback_seconds: params.lookback,
        since: params.since,
        until: params.until,
    };

    // Query logs
    let log_filter = LogFilter {
        common: common.clone(),
        cursor: cursor.clone(),
        span_id: None,
    };

    let logs = state
        .db
        .query_logs(log_filter)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;
    // Query spans
    let span_filter = SpanFilter {
        common: common.clone(),
        cursor: cursor.clone(),
        span_id: None,
        trace_id: None,
    };

    let spans = state
        .db
        .query_spans(span_filter)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    // Query span events
    let event_filter = SpanEventFilter {
        common: common.clone(),
        cursor: cursor.clone(),
        span_id: None,
        trace_id: None,
    };

    let events = state
        .db
        .query_span_events(event_filter)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    // Convert to unified response format
    let mut records: Vec<RecordResponse> = Vec::new();

    records.extend(logs.into_iter().map(|log| LogResponse::from(log).into()));
    records.extend(
        spans
            .into_iter()
            .map(|span| SpanResponse::from(span).into()),
    );
    records.extend(
        events
            .into_iter()
            .map(|event| SpanEventResponse::from(event).into()),
    );

    // Sort by timestamp, then type, then unique key for stable ordering
    records.sort_by(|a, b| a.cmp(b));

    // Deduplicate records (efficient on sorted data)
    deduplicate_records(&mut records);

    Ok(Json(PaginatedResponse::from_records(
        records,
        state.max_records_returned,
    )))
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
            AppError::Internal(msg) => {
                error!("Internal server error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let Query(params) = Query::<RecordQueryParams>::try_from_uri(
            &("http://localhost:1234/api/v1/records?level=foo&since=5"
                .parse()
                .unwrap()),
        )
        .unwrap();

        assert_eq!(dbg!(params).since, Some(5));
    }

    #[test]
    fn test_deduplicate_records() {
        use crate::level::Level;

        // Create some duplicate records
        let log1 = LogResponse {
            timestamp: 100,
            service_name: Some("test".to_string()),
            level: Some(Level::Info),
            target: Some("test".to_string()),
            message: Some("msg1".to_string()),
            span_id: None,
            trace_id: Some("trace1".to_string()),
            attributes: None,
        };

        let log2 = log1.clone(); // Duplicate

        let span1 = SpanResponse {
            timestamp: 100,
            trace_id: "trace1".to_string(),
            span_id: "span1".to_string(),
            parent_span_id: None,
            service_name: Some("test".to_string()),
            name: "test_span".to_string(),
            kind: None,
            start_time: 100,
            end_time: Some(200),
            level: Some(Level::Info),
            target: None,
            attributes: None,
            events: None,
            status: None,
        };

        let span2 = span1.clone(); // Duplicate

        let mut records = vec![
            RecordResponse::Log(log1),
            RecordResponse::Log(log2),
            RecordResponse::Span(span1),
            RecordResponse::Span(span2),
        ];

        // Sort first (deduplication requires sorted input)
        records.sort_by(|a, b| a.cmp(b));

        assert_eq!(records.len(), 4);

        // Deduplicate
        deduplicate_records(&mut records);

        // Should have removed duplicates
        assert_eq!(records.len(), 2);

        // Verify we have one log and one span
        let log_count = records
            .iter()
            .filter(|r| matches!(r, RecordResponse::Log(_)))
            .count();
        let span_count = records
            .iter()
            .filter(|r| matches!(r, RecordResponse::Span(_)))
            .count();

        assert_eq!(log_count, 1);
        assert_eq!(span_count, 1);
    }

    #[test]
    fn test_record_sorting_with_same_timestamp() {
        use crate::level::Level;

        // Create records with the same timestamp but different types
        let log = LogResponse {
            timestamp: 100,
            service_name: Some("test".to_string()),
            level: Some(Level::Info),
            target: None,
            message: Some("log".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let span = SpanResponse {
            timestamp: 100,
            trace_id: "trace1".to_string(),
            span_id: "span1".to_string(),
            parent_span_id: None,
            service_name: Some("test".to_string()),
            name: "span".to_string(),
            kind: None,
            start_time: 100,
            end_time: Some(200),
            level: Some(Level::Info),
            target: None,
            attributes: None,
            events: None,
            status: None,
        };

        let event = SpanEventResponse {
            timestamp: 100,
            span_id: "span1".to_string(),
            trace_id: "trace1".to_string(),
            service_name: Some("test".to_string()),
            name: "event".to_string(),
            level: Some(Level::Info),
            target: None,
            attributes: None,
        };

        let mut records = vec![
            RecordResponse::Event(event),
            RecordResponse::Span(span),
            RecordResponse::Log(log),
        ];

        // Sort by cmp which includes timestamp, type discriminator, and unique key
        records.sort_by(|a, b| a.cmp(b));

        // Verify order: Log (0) < Span (1) < Event (2)
        assert!(matches!(records[0], RecordResponse::Log(_)));
        assert!(matches!(records[1], RecordResponse::Span(_)));
        assert!(matches!(records[2], RecordResponse::Event(_)));
    }
}
