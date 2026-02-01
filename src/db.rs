use crate::level::Level;
use chrono::{Duration, Utc};
use std::path::Path;
use std::sync::Arc;
use tokio_rusqlite::Connection as AsyncConnection;
use tokio_rusqlite::rusqlite::{Connection, Result as SqliteResult, params};

#[derive(Debug, Clone)]
pub struct Database {
    conn: Arc<AsyncConnection>,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, tokio_rusqlite::Error> {
        let conn = AsyncConnection::open(path).await?;

        conn.call(|conn| {
            Self::create_tables(conn)?;
            Ok(())
        })
        .await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn in_memory() -> Result<Self, tokio_rusqlite::Error> {
        let conn = AsyncConnection::open_in_memory().await?;

        conn.call(|conn| {
            Self::create_tables(conn)?;
            Ok(())
        })
        .await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    fn create_tables(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                service_name TEXT,
                level INTEGER,
                target TEXT,
                message TEXT,
                span_id TEXT,
                trace_id TEXT,
                attributes TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_service_name ON logs(service_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_target ON logs(target)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_span_id ON logs(span_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS spans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trace_id TEXT NOT NULL,
                span_id TEXT NOT NULL UNIQUE,
                parent_span_id TEXT,
                service_name TEXT,
                name TEXT NOT NULL,
                kind TEXT,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                level INTEGER,
                target TEXT,
                attributes TEXT,
                events TEXT,
                status TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_spans_trace_id ON spans(trace_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_spans_span_id ON spans(span_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_spans_service_name ON spans(service_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_spans_target ON spans(target)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_spans_level ON spans(level)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS span_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                span_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                service_name TEXT,
                name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                level INTEGER,
                target TEXT,
                attributes TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (span_id) REFERENCES spans(span_id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_span_events_span_id ON span_events(span_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_span_events_trace_id ON span_events(trace_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_span_events_timestamp ON span_events(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_span_events_service_name ON span_events(service_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_span_events_level ON span_events(level)",
            [],
        )?;

        Ok(())
    }

    pub async fn insert_log(&self, log: LogRecord) -> Result<(), tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO logs (timestamp, service_name, level, target, message, span_id, trace_id, attributes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    log.timestamp,
                    log.service_name,
                    log.level.map(|l| l.to_int()),
                    log.target,
                    log.message,
                    log.span_id,
                    log.trace_id,
                    log.attributes,
                    Utc::now().timestamp_nanos_opt().unwrap_or(0),
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn insert_span(&self, span: SpanRecord) -> Result<(), tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO spans (trace_id, span_id, parent_span_id, service_name, name, kind, start_time, end_time, level, target, attributes, events, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    span.trace_id,
                    span.span_id,
                    span.parent_span_id,
                    span.service_name,
                    span.name,
                    span.kind,
                    span.start_time,
                    span.end_time,
                    span.level.map(|l| l.to_int()),
                    span.target,
                    span.attributes,
                    span.events,
                    span.status,
                    Utc::now().timestamp_nanos_opt().unwrap_or(0),
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn insert_span_event(
        &self,
        event: SpanEventRecord,
    ) -> Result<(), tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO span_events (span_id, trace_id, service_name, name, timestamp, level, target, attributes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    event.span_id,
                    event.trace_id,
                    event.service_name,
                    event.name,
                    event.timestamp,
                    event.level.map(|l| l.to_int()),
                    event.target,
                    event.attributes,
                    Utc::now().timestamp_nanos_opt().unwrap_or(0),
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub async fn query_logs(
        &self,
        filter: LogFilter,
    ) -> Result<Vec<LogRecord>, tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            let mut query = String::from("SELECT id, timestamp, service_name, level, target, message, span_id, trace_id, attributes FROM logs WHERE 1=1");
            let mut params: Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>> = Vec::new();

            filter.cursor.filter_query(&mut query, &mut params);
            filter.common.filter_query(&mut query, &mut params);


            if let Some(span_id) = &filter.span_id {
                query.push_str(" AND span_id = ?");
                params.push(Box::new(span_id.clone()));
            }

            query.push_str(" ORDER BY timestamp ASC");

            let param_refs: Vec<&dyn tokio_rusqlite::rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&query)?;
            let logs = stmt
                .query_map(param_refs.as_slice(), |row| {
                    let level_int: Option<i32> = row.get(3)?;
                    Ok(LogRecord {
                        timestamp: row.get(1)?,
                        service_name: row.get(2)?,
                        level: level_int.and_then(Level::from_int),
                        target: row.get(4)?,
                        message: row.get(5)?,
                        span_id: row.get(6)?,
                        trace_id: row.get(7)?,
                        attributes: row.get(8)?,
                    })
                })?
                .collect::<SqliteResult<Vec<_>>>()?;

            Ok(logs)
        })
        .await
    }

    pub async fn query_spans(
        &self,
        filter: SpanFilter,
    ) -> Result<Vec<SpanRecord>, tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        let initial_spans = conn.call(move |conn| {
            let mut query = String::from("SELECT trace_id, span_id, parent_span_id, service_name, name, kind, start_time, end_time, level, target, attributes, events, status FROM spans WHERE 1=1");
            let mut params: Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>> = Vec::new();

            filter.cursor.filter_spans(&mut query, &mut params);
            filter.common.filter_query(&mut query, &mut params);


            if let Some(span_id) = &filter.span_id {
                query.push_str(" AND span_id = ?");
                params.push(Box::new(span_id.clone()));
            }

            if let Some(trace_id) = &filter.trace_id {
                query.push_str(" AND trace_id = ?");
                params.push(Box::new(trace_id.clone()));
            }

            query.push_str(" ORDER BY start_time ASC");

            let param_refs: Vec<&dyn tokio_rusqlite::rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&query)?;
            let spans = stmt
                .query_map(param_refs.as_slice(), |row| {
                    let level_int: Option<i32> = row.get(8)?;
                    Ok(SpanRecord {
                        trace_id: row.get(0)?,
                        span_id: row.get(1)?,
                        parent_span_id: row.get(2)?,
                        service_name: row.get(3)?,
                        name: row.get(4)?,
                        kind: row.get(5)?,
                        start_time: row.get(6)?,
                        end_time: row.get(7)?,
                        level: level_int.and_then(Level::from_int),
                        target: row.get(9)?,
                        attributes: row.get(10)?,
                        events: row.get(11)?,
                        status: row.get(12)?,
                    })
                })?
                .collect::<SqliteResult<Vec<_>>>()?;

            Ok(spans)
        })
        .await?;

        self.get_spans_with_children(initial_spans).await
    }

    async fn get_spans_with_children(
        &self,
        spans: Vec<SpanRecord>,
    ) -> Result<Vec<SpanRecord>, tokio_rusqlite::Error> {
        let mut result = Vec::new();
        let mut to_process = spans;

        while !to_process.is_empty() {
            let current_span_ids: Vec<String> =
                to_process.iter().map(|s| s.span_id.clone()).collect();
            result.append(&mut to_process);

            if current_span_ids.is_empty() {
                break;
            }

            let conn = Arc::clone(&self.conn);
            to_process = conn.call(move |conn| {
                let placeholders = current_span_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let query = format!(
                    "SELECT trace_id, span_id, parent_span_id, service_name, name, kind, start_time, end_time, level, target, attributes, events, status FROM spans WHERE parent_span_id IN ({})",
                    placeholders
                );

                let params: Vec<&dyn tokio_rusqlite::rusqlite::ToSql> = current_span_ids.iter().map(|s| s as &dyn tokio_rusqlite::rusqlite::ToSql).collect();

                let mut stmt = conn.prepare(&query)?;
                let children = stmt
                    .query_map(params.as_slice(), |row| {
                        let level_int: Option<i32> = row.get(8)?;
                        Ok(SpanRecord {
                            trace_id: row.get(0)?,
                            span_id: row.get(1)?,
                            parent_span_id: row.get(2)?,
                            service_name: row.get(3)?,
                            name: row.get(4)?,
                            kind: row.get(5)?,
                            start_time: row.get(6)?,
                            end_time: row.get(7)?,
                            level: level_int.and_then(Level::from_int),
                            target: row.get(9)?,
                            attributes: row.get(10)?,
                            events: row.get(11)?,
                            status: row.get(12)?,
                        })
                    })?
                    .collect::<SqliteResult<Vec<_>>>()?;

                Ok(children)
            })
            .await?;
        }

        Ok(result)
    }

    pub async fn query_span_events(
        &self,
        filter: SpanEventFilter,
    ) -> Result<Vec<SpanEventRecord>, tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            let mut query = String::from("SELECT span_id, trace_id, service_name, name, timestamp, level, target, attributes FROM span_events WHERE 1=1");
            let mut params: Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>> = Vec::new();

            filter.cursor.filter_query(&mut query, &mut params);
            filter.common.filter_query(&mut query, &mut params);
            if let Some(span_id) = &filter.span_id {
                query.push_str(" AND span_id = ?");
                params.push(Box::new(span_id.clone()));
            }

            if let Some(trace_id) = &filter.trace_id {
                query.push_str(" AND trace_id = ?");
                params.push(Box::new(trace_id.clone()));
            }

            query.push_str(" ORDER BY timestamp ASC");

            let param_refs: Vec<&dyn tokio_rusqlite::rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&query)?;
            let events = stmt
                .query_map(param_refs.as_slice(), |row| {
                    let level_int: Option<i32> = row.get(5)?;
                    Ok(SpanEventRecord {
                        span_id: row.get(0)?,
                        trace_id: row.get(1)?,
                        service_name: row.get(2)?,
                        name: row.get(3)?,
                        timestamp: row.get(4)?,
                        level: level_int.and_then(Level::from_int),
                        target: row.get(6)?,
                        attributes: row.get(7)?,
                    })
                })?
                .collect::<SqliteResult<Vec<_>>>()?;

            Ok(events)
        })
        .await
    }

    pub async fn cleanup_old_records(
        &self,
        max_age_seconds: i64,
    ) -> Result<(), tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            let cutoff =
                Utc::now().timestamp_nanos_opt().unwrap_or(0) - (max_age_seconds * 1_000_000_000);

            conn.execute("DELETE FROM logs WHERE created_at < ?1", params![cutoff])?;
            conn.execute("DELETE FROM spans WHERE created_at < ?1", params![cutoff])?;
            conn.execute(
                "DELETE FROM span_events WHERE created_at < ?1",
                params![cutoff],
            )?;

            Ok(())
        })
        .await
    }

    pub async fn cleanup_by_count(
        &self,
        max_logs: i64,
        max_spans: i64,
    ) -> Result<(), tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            // Delete oldest logs beyond max_logs
            conn.execute(
                "DELETE FROM logs WHERE id IN (
                    SELECT id FROM logs ORDER BY created_at DESC LIMIT -1 OFFSET ?1
                )",
                params![max_logs],
            )?;

            // Delete oldest spans beyond max_spans
            conn.execute(
                "DELETE FROM spans WHERE id IN (
                    SELECT id FROM spans ORDER BY created_at DESC LIMIT -1 OFFSET ?1
                )",
                params![max_spans],
            )?;

            Ok(())
        })
        .await
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats, tokio_rusqlite::Error> {
        let conn = Arc::clone(&self.conn);
        conn.call(move |conn| {
            let total_logs: i64 =
                conn.query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))?;
            let total_spans: i64 =
                conn.query_row("SELECT COUNT(*) FROM spans", [], |row| row.get(0))?;

            let oldest_log_timestamp: Option<i64> = conn
                .query_row("SELECT MIN(timestamp) FROM logs", [], |row| row.get(0))
                .ok();

            let newest_log_timestamp: Option<i64> = conn
                .query_row("SELECT MAX(timestamp) FROM logs", [], |row| row.get(0))
                .ok();

            let oldest_span_timestamp: Option<i64> = conn
                .query_row("SELECT MIN(start_time) FROM spans", [], |row| row.get(0))
                .ok();

            let newest_span_timestamp: Option<i64> = conn
                .query_row("SELECT MAX(start_time) FROM spans", [], |row| row.get(0))
                .ok();

            let database_size_bytes: Option<i64> = conn
                .query_row(
                    "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                    [],
                    |row| row.get(0),
                )
                .ok();

            Ok(DatabaseStats {
                total_logs,
                total_spans,
                oldest_log_timestamp,
                newest_log_timestamp,
                oldest_span_timestamp,
                newest_span_timestamp,
                database_size_bytes,
            })
        })
        .await
    }
}

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: i64,
    pub service_name: Option<String>,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub attributes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpanRecord {
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
    pub attributes: Option<String>,
    pub events: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpanEventRecord {
    pub span_id: String,
    pub trace_id: String,
    pub service_name: Option<String>,
    pub name: String,
    pub timestamp: i64,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub attributes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CursorFilter {
    pub lookback_seconds: Option<i64>,
    pub starting_at: Option<i64>,
}

impl CursorFilter {
    pub fn filter_query(
        &self,
        query: &mut String,
        params: &mut Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>>,
    ) {
        // Timestamp-based filtering
        if let Some(lookback) = self.lookback_seconds {
            tracing::debug!(%lookback, "Applying lookback filter");
            let cutoff = Utc::now() - Duration::seconds(lookback);
            query.push_str(" AND timestamp > ?");
            params.push(Box::new(
                cutoff.timestamp_nanos_opt().expect("valid timestamp"),
            ));
        }

        if let Some(after) = self.starting_at {
            tracing::debug!(%after, "Applying starting_at filter");
            query.push_str(" AND timestamp > ?");
            params.push(Box::new(after));
        }
    }

    pub fn filter_spans(
        &self,
        query: &mut String,
        params: &mut Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>>,
    ) {
        // Timestamp-based filtering
        if let Some(lookback) = self.lookback_seconds {
            tracing::debug!(%lookback, "Applying lookback filter");
            let cutoff = Utc::now() - Duration::seconds(lookback);
            query.push_str(" AND start_time > ?");
            params.push(Box::new(
                cutoff.timestamp_nanos_opt().expect("valid timestamp"),
            ));
        }

        if let Some(after) = self.starting_at {
            tracing::debug!(%after, "Applying starting_at filter");
            query.push_str(" AND start_time > ?");
            params.push(Box::new(after));
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommonFilters {
    pub service_name: Option<String>,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub message: Option<String>,
}

impl CommonFilters {
    pub fn filter_query(
        &self,
        query: &mut String,
        params: &mut Vec<Box<dyn tokio_rusqlite::rusqlite::ToSql>>,
    ) {
        if let Some(service_name) = &self.service_name {
            query.push_str(" AND service_name = ?");
            params.push(Box::new(service_name.clone()));
        }

        if let Some(level) = &self.level {
            // Filter for minimum level (e.g., if level is WARN, show WARN, ERROR, and FATAL)
            query.push_str(" AND level >= ?");
            params.push(Box::new(level.to_int()));
        }

        if let Some(target) = &self.target {
            query.push_str(" AND target = ?");
            params.push(Box::new(target.clone()));
        }

        if let Some(message) = &self.message {
            query.push_str(" AND message LIKE ?");
            params.push(Box::new(format!("%{message}%")));
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub common: CommonFilters,
    pub cursor: CursorFilter,
    pub span_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SpanFilter {
    pub common: CommonFilters,
    pub cursor: CursorFilter,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SpanEventFilter {
    pub common: CommonFilters,
    pub cursor: CursorFilter,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub total_logs: i64,
    pub total_spans: i64,
    pub oldest_log_timestamp: Option<i64>,
    pub newest_log_timestamp: Option<i64>,
    pub oldest_span_timestamp: Option<i64>,
    pub newest_span_timestamp: Option<i64>,
    pub database_size_bytes: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_query_logs() {
        let db = Database::in_memory().await.unwrap();

        let log = LogRecord {
            timestamp: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            service_name: Some("test-service".to_string()),
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            message: Some("Test message".to_string()),
            span_id: Some("span123".to_string()),
            trace_id: Some("trace456".to_string()),
            attributes: Some("{}".to_string()),
        };

        db.insert_log(log.clone()).await.unwrap();

        let filter = LogFilter {
            common: CommonFilters {
                service_name: Some("test-service".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let logs = db.query_logs(filter).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, Some("Test message".to_string()));
    }

    #[tokio::test]
    async fn test_insert_and_query_spans() {
        let db = Database::in_memory().await.unwrap();

        let span = SpanRecord {
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            parent_span_id: None,
            service_name: Some("test-service".to_string()),
            name: "test_span".to_string(),
            kind: Some("internal".to_string()),
            start_time: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: Some("{}".to_string()),
            events: Some("[]".to_string()),
            status: Some("Ok".to_string()),
        };

        db.insert_span(span.clone()).await.unwrap();

        let filter = SpanFilter {
            trace_id: Some("trace123".to_string()),
            ..Default::default()
        };

        let spans = db.query_spans(filter).await.unwrap();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "test_span");
    }

    #[tokio::test]
    async fn test_log_filtering() {
        let db = Database::in_memory().await.unwrap();

        let log1 = LogRecord {
            timestamp: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            service_name: Some("service1".to_string()),
            level: Some(Level::Error),
            target: Some("module1".to_string()),
            message: Some("Error occurred".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let log2 = LogRecord {
            timestamp: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            service_name: Some("service2".to_string()),
            level: Some(Level::Info),
            target: Some("module2".to_string()),
            message: Some("Info message".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        db.insert_log(log1).await.unwrap();
        db.insert_log(log2).await.unwrap();

        let filter = LogFilter {
            common: CommonFilters {
                level: Some(Level::Error),
                ..Default::default()
            },
            ..Default::default()
        };

        let logs = db.query_logs(filter).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, Some(Level::Error));
    }

    #[tokio::test]
    async fn test_recursive_child_span_retrieval() {
        let db = Database::in_memory().await.unwrap();

        let parent_span = SpanRecord {
            trace_id: "trace789".to_string(),
            span_id: "parent".to_string(),
            parent_span_id: None,
            service_name: Some("test-service".to_string()),
            name: "parent_span".to_string(),
            kind: Some("internal".to_string()),
            start_time: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: None,
            events: None,
            status: None,
        };

        let child_span1 = SpanRecord {
            trace_id: "trace789".to_string(),
            span_id: "child1".to_string(),
            parent_span_id: Some("parent".to_string()),
            service_name: Some("test-service".to_string()),
            name: "child_span_1".to_string(),
            kind: Some("internal".to_string()),
            start_time: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: None,
            events: None,
            status: None,
        };

        let child_span2 = SpanRecord {
            trace_id: "trace789".to_string(),
            span_id: "child2".to_string(),
            parent_span_id: Some("parent".to_string()),
            service_name: Some("test-service".to_string()),
            name: "child_span_2".to_string(),
            kind: Some("internal".to_string()),
            start_time: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: None,
            events: None,
            status: None,
        };

        let grandchild_span = SpanRecord {
            trace_id: "trace789".to_string(),
            span_id: "grandchild".to_string(),
            parent_span_id: Some("child1".to_string()),
            service_name: Some("test-service".to_string()),
            name: "grandchild_span".to_string(),
            kind: Some("internal".to_string()),
            start_time: Utc::now().timestamp_nanos_opt().unwrap_or(0),
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: None,
            events: None,
            status: None,
        };

        db.insert_span(parent_span).await.unwrap();
        db.insert_span(child_span1).await.unwrap();
        db.insert_span(child_span2).await.unwrap();
        db.insert_span(grandchild_span).await.unwrap();

        let filter = SpanFilter {
            span_id: Some("parent".to_string()),
            ..Default::default()
        };

        let spans = db.query_spans(filter).await.unwrap();

        assert_eq!(spans.len(), 4);

        let span_names: Vec<&str> = spans.iter().map(|s| s.name.as_str()).collect();
        assert!(span_names.contains(&"parent_span"));
        assert!(span_names.contains(&"child_span_1"));
        assert!(span_names.contains(&"child_span_2"));
        assert!(span_names.contains(&"grandchild_span"));
    }

    #[tokio::test]
    async fn test_duplicate_span_handling() {
        let db = Database::in_memory().await.unwrap();

        let span1 = SpanRecord {
            trace_id: "trace-duplicate".to_string(),
            span_id: "duplicate-span".to_string(),
            parent_span_id: None,
            service_name: Some("test-service".to_string()),
            name: "first_insert".to_string(),
            kind: Some("internal".to_string()),
            start_time: 1000,
            end_time: None,
            level: Some(Level::Info),
            target: Some("test::module".to_string()),
            attributes: Some("{}".to_string()),
            events: Some("[]".to_string()),
            status: Some("Ok".to_string()),
        };

        let span2 = SpanRecord {
            trace_id: "trace-duplicate".to_string(),
            span_id: "duplicate-span".to_string(), // Same span_id
            parent_span_id: None,
            service_name: Some("test-service".to_string()),
            name: "second_insert".to_string(), // Different name
            kind: Some("internal".to_string()),
            start_time: 2000, // Different timestamp
            end_time: None,
            level: Some(Level::Warn), // Different level
            target: Some("test::module".to_string()),
            attributes: Some("{}".to_string()),
            events: Some("[]".to_string()),
            status: Some("Ok".to_string()),
        };

        // Insert the same span_id twice
        db.insert_span(span1).await.unwrap();
        db.insert_span(span2).await.unwrap();

        let filter = SpanFilter {
            span_id: Some("duplicate-span".to_string()),
            ..Default::default()
        };

        let spans = db.query_spans(filter).await.unwrap();

        // Should only have one span (the second one replaces the first)
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "second_insert");
        assert_eq!(spans[0].start_time, 2000);
        assert_eq!(spans[0].level, Some(Level::Warn));
    }
}
