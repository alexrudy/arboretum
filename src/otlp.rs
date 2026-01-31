use crate::db::{LogRecord, SpanRecord};
use crate::level::Level;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::KeyValue;
use opentelemetry_proto::tonic::common::v1::any_value::Value as OtelValue;

pub fn convert_otlp_logs(request: ExportLogsServiceRequest) -> Vec<LogRecord> {
    let mut logs = Vec::new();

    for resource_log in request.resource_logs {
        let service_name = resource_log.resource.as_ref().and_then(|r| {
            r.attributes.iter().find_map(|kv| {
                if kv.key == "service.name" {
                    extract_string_value(kv)
                } else {
                    None
                }
            })
        });

        for scope_log in resource_log.scope_logs {
            for log_record in scope_log.log_records {
                let level = severity_to_level(log_record.severity_number);
                let message = extract_body_as_string(&log_record.body);

                let mut target = None;
                let mut span_id_hex = None;
                let mut trace_id_hex = None;
                let mut attributes_map = serde_json::Map::new();

                for attr in &log_record.attributes {
                    if attr.key == "code.target" || attr.key == "target" {
                        target = extract_string_value(attr);
                        // Keep target in attributes too
                    }

                    // Add all attributes to the attributes map
                    if let Some(value) = attribute_to_json_value(attr) {
                        attributes_map.insert(attr.key.clone(), value);
                    }
                }

                if !log_record.span_id.is_empty() {
                    span_id_hex = Some(hex::encode(&log_record.span_id));
                }

                if !log_record.trace_id.is_empty() {
                    trace_id_hex = Some(hex::encode(&log_record.trace_id));
                }

                let attributes = if !attributes_map.is_empty() {
                    Some(serde_json::to_string(&attributes_map).unwrap_or_default())
                } else {
                    None
                };

                logs.push(LogRecord {
                    timestamp: log_record.time_unix_nano as i64,
                    service_name: service_name.clone(),
                    level,
                    target,
                    message,
                    span_id: span_id_hex,
                    trace_id: trace_id_hex,
                    attributes,
                });
            }
        }
    }

    logs
}

pub fn convert_otlp_traces(request: ExportTraceServiceRequest) -> Vec<SpanRecord> {
    let mut spans = Vec::new();

    for resource_span in request.resource_spans {
        let service_name = resource_span.resource.as_ref().and_then(|r| {
            r.attributes.iter().find_map(|kv| {
                if kv.key == "service.name" {
                    extract_string_value(kv)
                } else {
                    None
                }
            })
        });

        for scope_span in resource_span.scope_spans {
            // Check if the scope has level information in its name
            let scope_level = scope_span.scope.as_ref().and_then(|scope| {
                // Try to extract level from scope name (e.g., "my_module::info")
                if let Some(idx) = scope.name.rfind("::") {
                    let potential_level = &scope.name[idx + 2..];
                    potential_level.parse::<Level>().ok()
                } else {
                    None
                }
            });

            for span in scope_span.spans {
                let span_id_hex = hex::encode(&span.span_id);
                let trace_id_hex = hex::encode(&span.trace_id);
                let parent_span_id_hex = if !span.parent_span_id.is_empty() {
                    Some(hex::encode(&span.parent_span_id))
                } else {
                    None
                };

                let mut level = None;
                let mut target = None;
                let mut attributes_map = serde_json::Map::new();

                for attr in &span.attributes {
                    // Check for level in various attribute keys used by tracing-opentelemetry
                    if attr.key == "level"
                        || attr.key == "otel.level"
                        || attr.key == "log.level"
                        || attr.key == "log.severity"
                        || attr.key == "severity"
                    {
                        level = extract_string_value(attr).and_then(|s| s.parse().ok());
                        // Keep level in attributes too
                    }

                    if attr.key == "code.target" || attr.key == "target" {
                        target = extract_string_value(attr);
                        // Keep target in attributes too
                    }

                    // Add all attributes to the attributes map
                    if let Some(value) = attribute_to_json_value(attr) {
                        attributes_map.insert(attr.key.clone(), value);
                    }
                }

                // If level wasn't found in attributes, use scope level as fallback
                if level.is_none() {
                    level = scope_level;
                }

                let attributes = if !attributes_map.is_empty() {
                    Some(serde_json::to_string(&attributes_map).unwrap_or_default())
                } else {
                    None
                };

                let events = if !span.events.is_empty() {
                    let events_json: Vec<serde_json::Value> = span
                        .events
                        .iter()
                        .map(|e| {
                            let mut event_map = serde_json::Map::new();
                            event_map.insert(
                                "name".to_string(),
                                serde_json::Value::String(e.name.clone()),
                            );
                            event_map.insert(
                                "time".to_string(),
                                serde_json::Value::Number(e.time_unix_nano.into()),
                            );

                            if !e.attributes.is_empty() {
                                let mut attrs = serde_json::Map::new();
                                for attr in &e.attributes {
                                    if let Some(value) = attribute_to_json_value(attr) {
                                        attrs.insert(attr.key.clone(), value);
                                    }
                                }
                                event_map.insert(
                                    "attributes".to_string(),
                                    serde_json::Value::Object(attrs),
                                );
                            }

                            serde_json::Value::Object(event_map)
                        })
                        .collect();
                    Some(serde_json::to_string(&events_json).unwrap_or_default())
                } else {
                    None
                };

                let status = span.status.as_ref().map(|s| format!("{:?}", s.code()));

                let kind = Some(format!("{:?}", span.kind()));

                spans.push(SpanRecord {
                    trace_id: trace_id_hex,
                    span_id: span_id_hex,
                    parent_span_id: parent_span_id_hex,
                    service_name: service_name.clone(),
                    name: span.name,
                    kind,
                    start_time: span.start_time_unix_nano as i64,
                    end_time: Some(span.end_time_unix_nano as i64),
                    level,
                    target,
                    attributes,
                    events,
                    status,
                });
            }
        }
    }

    spans
}

fn severity_to_level(severity: i32) -> Option<Level> {
    Level::from_severity(severity)
}

fn extract_body_as_string(
    body: &Option<opentelemetry_proto::tonic::common::v1::AnyValue>,
) -> Option<String> {
    body.as_ref().and_then(|b| {
        b.value.as_ref().and_then(|v| match v {
            OtelValue::StringValue(s) => Some(s.clone()),
            OtelValue::IntValue(i) => Some(i.to_string()),
            OtelValue::DoubleValue(d) => Some(d.to_string()),
            OtelValue::BoolValue(b) => Some(b.to_string()),
            _ => None,
        })
    })
}

fn extract_string_value(kv: &KeyValue) -> Option<String> {
    kv.value.as_ref().and_then(|v| {
        v.value.as_ref().and_then(|val| match val {
            OtelValue::StringValue(s) => Some(s.clone()),
            _ => None,
        })
    })
}

fn attribute_to_json_value(kv: &KeyValue) -> Option<serde_json::Value> {
    kv.value.as_ref().and_then(|v| {
        v.value.as_ref().map(|val| match val {
            OtelValue::StringValue(s) => serde_json::Value::String(s.clone()),
            OtelValue::IntValue(i) => serde_json::Value::Number((*i).into()),
            OtelValue::DoubleValue(d) => serde_json::Number::from_f64(*d)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            OtelValue::BoolValue(b) => serde_json::Value::Bool(*b),
            OtelValue::ArrayValue(arr) => {
                let values: Vec<serde_json::Value> = arr
                    .values
                    .iter()
                    .filter_map(|v| {
                        v.value.as_ref().map(|val| match val {
                            OtelValue::StringValue(s) => serde_json::Value::String(s.clone()),
                            OtelValue::IntValue(i) => serde_json::Value::Number((*i).into()),
                            OtelValue::BoolValue(b) => serde_json::Value::Bool(*b),
                            _ => serde_json::Value::Null,
                        })
                    })
                    .collect();
                serde_json::Value::Array(values)
            }
            _ => serde_json::Value::Null,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::common::v1::any_value::Value as OtelValue;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use opentelemetry_proto::tonic::logs::v1::{
        LogRecord as OtelLogRecord, ResourceLogs, ScopeLogs,
    };
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

    #[test]
    fn test_convert_otlp_logs() {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelValue::StringValue("test-service".to_string())),
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
                    log_records: vec![OtelLogRecord {
                        time_unix_nano: 1234567890,
                        severity_number: 9,
                        severity_text: "INFO".to_string(),
                        body: Some(AnyValue {
                            value: Some(OtelValue::StringValue("Test log message".to_string())),
                        }),
                        attributes: vec![KeyValue {
                            key: "target".to_string(),
                            value: Some(AnyValue {
                                value: Some(OtelValue::StringValue("test::module".to_string())),
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

        let logs = convert_otlp_logs(request);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].service_name, Some("test-service".to_string()));
        assert_eq!(logs[0].level, Some(Level::Info));
        assert_eq!(logs[0].message, Some("Test log message".to_string()));
        assert_eq!(logs[0].target, Some("test::module".to_string()));
    }

    #[test]
    fn test_convert_otlp_traces() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(OtelValue::StringValue("test-service".to_string())),
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
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        parent_span_id: vec![],
                        name: "test_span".to_string(),
                        kind: 1,
                        start_time_unix_nano: 1234567890,
                        end_time_unix_nano: 1234567900,
                        attributes: vec![KeyValue {
                            key: "target".to_string(),
                            value: Some(AnyValue {
                                value: Some(OtelValue::StringValue("test::module".to_string())),
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

        let spans = convert_otlp_traces(request);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].service_name, Some("test-service".to_string()));
        assert_eq!(spans[0].name, "test_span");
        assert_eq!(spans[0].target, Some("test::module".to_string()));
    }

    #[test]
    fn test_severity_to_level() {
        assert_eq!(severity_to_level(2), Some(Level::Trace));
        assert_eq!(severity_to_level(5), Some(Level::Debug));
        assert_eq!(severity_to_level(9), Some(Level::Info));
        assert_eq!(severity_to_level(13), Some(Level::Warn));
        assert_eq!(severity_to_level(17), Some(Level::Error));
        assert_eq!(severity_to_level(21), Some(Level::Fatal));
    }
}
