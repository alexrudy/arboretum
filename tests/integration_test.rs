use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as OtelValue;
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use prost::Message;
use std::net::TcpListener;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to find an available port
fn get_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start the arboretum server in a background task
async fn start_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Create an in-memory database for testing
        let db = arboretum::db::Database::in_memory().await.unwrap();

        // Use the shared router construction function
        let app = arboretum::server::create_router(db);

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();

        axum::serve(listener, app).await.unwrap();
    })
}

#[tokio::test]
async fn test_server_startup_and_health() {
    let port = get_available_port();
    let _server = start_test_server(port).await;

    // Give the server time to start
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/api/v1/metadata", port);

    // Try to connect to the metadata endpoint
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), 200);

    let metadata: serde_json::Value = response.json().await.unwrap();
    assert!(metadata.get("total_logs").is_some());
    assert!(metadata.get("total_spans").is_some());
}

#[tokio::test]
async fn test_otlp_log_ingestion_and_query() {
    let port = get_available_port();
    let _server = start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Create an OTLP log request
    let request = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(OtelValue::StringValue("integration-test".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    name: "test".to_string(),
                    version: "1.0".to_string(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                log_records: vec![LogRecord {
                    time_unix_nano: 1234567890,
                    severity_number: 9, // INFO
                    severity_text: "INFO".to_string(),
                    body: Some(AnyValue {
                        value: Some(OtelValue::StringValue("Integration test log".to_string())),
                    }),
                    attributes: vec![KeyValue {
                        key: "target".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelValue::StringValue("integration::test".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    flags: 0,
                    trace_id: vec![],
                    span_id: vec![],
                    observed_time_unix_nano: 0,
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    // Encode to protobuf
    let mut buf = Vec::new();
    request.encode(&mut buf).unwrap();

    // Send logs to the server
    let response = client
        .post(format!("http://127.0.0.1:{}/v1/logs", port))
        .header("content-type", "application/x-protobuf")
        .body(buf)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Query the logs back
    sleep(Duration::from_millis(50)).await;

    let response = client
        .get(format!(
            "http://127.0.0.1:{}/api/v1/logs?service_name=integration-test",
            port
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let logs: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0]["message"], "Integration test log");
    assert_eq!(logs[0]["service_name"], "integration-test");
    assert_eq!(logs[0]["level"], "INFO");
    assert_eq!(logs[0]["target"], "integration::test");
}

#[tokio::test]
async fn test_otlp_trace_ingestion_and_query() {
    let port = get_available_port();
    let _server = start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Create an OTLP trace request
    let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let span_id = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(OtelValue::StringValue("trace-test".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope {
                    name: "test".to_string(),
                    version: "1.0".to_string(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                spans: vec![Span {
                    trace_id: trace_id.clone(),
                    span_id: span_id.clone(),
                    parent_span_id: vec![],
                    name: "integration_test_span".to_string(),
                    kind: 1,
                    start_time_unix_nano: 1234567890,
                    end_time_unix_nano: 1234567900,
                    attributes: vec![KeyValue {
                        key: "target".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelValue::StringValue("integration::test".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                    events: vec![],
                    dropped_events_count: 0,
                    links: vec![],
                    dropped_links_count: 0,
                    status: None,
                    trace_state: String::new(),
                    flags: 0,
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    // Encode to protobuf
    let mut buf = Vec::new();
    request.encode(&mut buf).unwrap();

    // Send traces to the server
    let response = client
        .post(format!("http://127.0.0.1:{}/v1/traces", port))
        .header("content-type", "application/x-protobuf")
        .body(buf)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Query the spans back
    sleep(Duration::from_millis(50)).await;

    let response = client
        .get(format!(
            "http://127.0.0.1:{}/api/v1/spans?service_name=trace-test",
            port
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let spans: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0]["name"], "integration_test_span");
    assert_eq!(spans[0]["service_name"], "trace-test");
    assert_eq!(spans[0]["target"], "integration::test");
}

#[tokio::test]
async fn test_log_filtering() {
    let port = get_available_port();
    let _server = start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Insert logs with different levels
    for (level, severity) in &[("ERROR", 17), ("WARN", 13), ("INFO", 9)] {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelValue::StringValue("filter-test".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "test".to_string(),
                        version: "1.0".to_string(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                    }),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1234567890,
                        severity_number: *severity,
                        severity_text: level.to_string(),
                        body: Some(AnyValue {
                            value: Some(OtelValue::StringValue(format!("{} message", level))),
                        }),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        flags: 0,
                        trace_id: vec![],
                        span_id: vec![],
                        observed_time_unix_nano: 0,
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();

        client
            .post(format!("http://127.0.0.1:{}/v1/logs", port))
            .header("content-type", "application/x-protobuf")
            .body(buf)
            .send()
            .await
            .unwrap();
    }

    sleep(Duration::from_millis(50)).await;

    // Query for ERROR level logs only
    let response = client
        .get(format!(
            "http://127.0.0.1:{}/api/v1/logs?service_name=filter-test&level=ERROR",
            port
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let logs: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0]["level"], "ERROR");
    assert_eq!(logs[0]["message"], "ERROR message");
}
