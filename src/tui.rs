use crate::level::Level;
use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use reqwest::Client;
use serde::Deserialize;
use std::io;
use std::str::FromStr;
use std::time::Duration;

/// API response types
#[derive(Debug, Clone, Deserialize)]
struct LogResponse {
    pub timestamp: i64,
    pub service_name: Option<String>,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct SpanResponse {
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

#[derive(Debug, Clone, Deserialize)]
struct SpanEventResponse {
    pub timestamp: i64,
    pub span_id: String,
    pub trace_id: String,
    pub service_name: Option<String>,
    pub name: String,
    pub level: Option<Level>,
    pub target: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RecordResponse {
    Log(LogResponse),
    Span(SpanResponse),
    Event(SpanEventResponse),
}

impl RecordResponse {
    fn timestamp(&self) -> i64 {
        match self {
            RecordResponse::Log(log) => log.timestamp,
            RecordResponse::Span(span) => span.timestamp,
            RecordResponse::Event(event) => event.timestamp,
        }
    }

    fn level(&self) -> Option<Level> {
        match self {
            RecordResponse::Log(log) => log.level,
            RecordResponse::Span(span) => span.level,
            RecordResponse::Event(event) => event.level,
        }
    }

    fn target(&self) -> Option<&str> {
        match self {
            RecordResponse::Log(log) => log.target.as_deref(),
            RecordResponse::Span(span) => span.target.as_deref(),
            RecordResponse::Event(event) => event.target.as_deref(),
        }
    }

    fn matches_filter(&self, filter: &RecordFilter) -> bool {
        let target = self.target();
        if target.is_none() {
            return true;
        }

        let target = target.unwrap();
        let level = self.level();

        // Check if any filter matches this target
        for (filter_target, min_level) in &filter.filters {
            if target.starts_with(filter_target) {
                if let Some(record_level) = level {
                    return record_level >= *min_level;
                }
                return true;
            }
        }

        // If no specific filter matched, use the default level if set
        if let Some(default_level) = filter.default_level {
            if let Some(record_level) = level {
                return record_level >= default_level;
            }
        }

        true
    }
}

/// Parses a filter string like "target=level,other_target=info"
#[derive(Debug, Clone, Default)]
struct RecordFilter {
    filters: Vec<(String, Level)>,
    default_level: Option<Level>,
}

impl RecordFilter {
    fn parse(input: &str) -> Self {
        let mut filters = Vec::new();
        let mut default_level = None;

        for part in input.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((target, level_str)) = part.split_once('=') {
                let target = target.trim().to_string();
                let level_str = level_str.trim();

                if let Ok(level) = Level::from_str(level_str) {
                    if target.is_empty() {
                        default_level = Some(level);
                    } else {
                        filters.push((target, level));
                    }
                }
            }
        }

        Self {
            filters,
            default_level,
        }
    }
}

/// Formats a record as a single line for display
fn format_record(record: &RecordResponse) -> Line<'static> {
    let mut spans = Vec::new();

    // Timestamp (dimmed)
    let timestamp_nanos = record.timestamp();
    let timestamp_secs = timestamp_nanos / 1_000_000_000;
    let timestamp = DateTime::<Utc>::from_timestamp(timestamp_secs, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    spans.push(Span::styled(
        format!("{} ", timestamp),
        Style::default().fg(Color::DarkGray),
    ));

    // Level (colored)
    if let Some(level) = record.level() {
        let (level_text, level_color) = match level {
            Level::Trace => ("TRACE", Color::Blue),
            Level::Debug => ("DEBUG", Color::Cyan),
            Level::Info => ("INFO ", Color::Green),
            Level::Warn => ("WARN ", Color::Yellow),
            Level::Error => ("ERROR", Color::Red),
            Level::Fatal => ("FATAL", Color::Magenta),
        };
        spans.push(Span::styled(
            format!("{} ", level_text),
            Style::default()
                .fg(level_color)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Target (dimmed)
    if let Some(target) = record.target() {
        spans.push(Span::styled(
            format!("{}: ", target),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Message/name and attributes
    match record {
        RecordResponse::Log(log) => {
            if let Some(message) = &log.message {
                spans.push(Span::raw(message.clone()));
            }

            if let Some(attrs) = &log.attributes {
                if let Some(obj) = attrs.as_object() {
                    let filtered: Vec<String> = obj
                        .iter()
                        .filter(|(k, _)| !k.starts_with("code."))
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    if !filtered.is_empty() {
                        spans.push(Span::styled(
                            format!(" {}", filtered.join(" ")),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }
            }
        }
        RecordResponse::Span(span) => {
            spans.push(Span::styled(
                format!("span: {}", span.name),
                Style::default().add_modifier(Modifier::ITALIC),
            ));

            if let Some(kind) = &span.kind {
                spans.push(Span::styled(
                    format!(" [{}]", kind),
                    Style::default().fg(Color::Gray),
                ));
            }

            if let Some(attrs) = &span.attributes {
                if let Some(obj) = attrs.as_object() {
                    let filtered: Vec<String> = obj
                        .iter()
                        .filter(|(k, _)| !k.starts_with("code."))
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    if !filtered.is_empty() {
                        spans.push(Span::styled(
                            format!(" {}", filtered.join(" ")),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }
            }
        }
        RecordResponse::Event(event) => {
            spans.push(Span::styled(
                format!("event: {}", event.name),
                Style::default().add_modifier(Modifier::ITALIC),
            ));

            if let Some(attrs) = &event.attributes {
                if let Some(obj) = attrs.as_object() {
                    let filtered: Vec<String> = obj
                        .iter()
                        .filter(|(k, _)| !k.starts_with("code."))
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    if !filtered.is_empty() {
                        spans.push(Span::styled(
                            format!(" {}", filtered.join(" ")),
                            Style::default().fg(Color::Gray),
                        ));
                    }
                }
            }
        }
    }

    Line::from(spans)
}

/// Main TUI application state
struct App {
    /// Records to display
    records: Vec<RecordResponse>,
    /// Maximum number of records to keep
    max_records: usize,
    /// Current filter string
    filter_input: String,
    /// Parsed filter
    filter: RecordFilter,
    /// Whether we're editing the filter
    editing_filter: bool,
    /// Cursor position in filter input
    cursor_position: usize,
    /// Last timestamp we've seen
    last_timestamp: i64,
    /// Server URL
    server_url: String,
    /// HTTP client
    client: Client,
}

impl App {
    fn new(server_url: String) -> Self {
        Self {
            records: Vec::new(),
            max_records: 1000,
            filter_input: String::new(),
            filter: RecordFilter::default(),
            editing_filter: false,
            cursor_position: 0,
            last_timestamp: 0,
            server_url,
            client: Client::new(),
        }
    }

    fn update_filter(&mut self) {
        self.filter = RecordFilter::parse(&self.filter_input);
    }

    fn add_record(&mut self, record: RecordResponse) {
        self.records.push(record);
        if self.records.len() > self.max_records {
            self.records.remove(0);
        }
    }

    async fn poll_new_records(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch records from the unified endpoint
        let url = format!("{}/api/v1/records?limit=100", self.server_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        let new_records: Vec<RecordResponse> = response.json().await?;

        for record in new_records {
            if record.timestamp() > self.last_timestamp {
                self.last_timestamp = record.timestamp();
                self.add_record(record);
            }
        }

        // Sort records by timestamp
        self.records.sort_by_key(|r| r.timestamp());

        Ok(())
    }
}

/// Runs the TUI application
pub async fn run(server_url: String) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(server_url);

    // Initial load
    app.poll_new_records().await?;

    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Metadata bar
                    Constraint::Min(0),    // Records list
                    Constraint::Length(3), // Filter input
                ])
                .split(f.area());

            // Metadata bar
            let metadata = Paragraph::new(format!(
                "Arboretum TUI | Server: {} | Records: {} | Press 'f' to filter, 'q' to quit",
                app.server_url,
                app.records.len()
            ))
            .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(metadata, chunks[0]);

            // Records list - filter and display
            let filtered_records: Vec<ListItem> = app
                .records
                .iter()
                .filter(|r| r.matches_filter(&app.filter))
                .map(|r| ListItem::new(format_record(r)))
                .collect();

            let records_list = List::new(filtered_records).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Logs, Spans, and Events"),
            );
            f.render_widget(records_list, chunks[1]);

            // Filter input
            let filter_style = if app.editing_filter {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let filter_display = if app.editing_filter {
                format!("Filter: {}█", app.filter_input)
            } else {
                format!(
                    "Filter: {} (press 'f' to edit)",
                    if app.filter_input.is_empty() {
                        "none"
                    } else {
                        &app.filter_input
                    }
                )
            };

            let filter_input = Paragraph::new(filter_display)
                .style(filter_style)
                .block(Block::default().borders(Borders::ALL).title("Filter"));
            f.render_widget(filter_input, chunks[2]);
        })?;

        // Handle events with timeout for polling
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.editing_filter {
                        match key.code {
                            KeyCode::Enter => {
                                app.editing_filter = false;
                                app.update_filter();
                            }
                            KeyCode::Esc => {
                                app.editing_filter = false;
                            }
                            KeyCode::Char(c) => {
                                app.filter_input.insert(app.cursor_position, c);
                                app.cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if app.cursor_position > 0 {
                                    app.cursor_position -= 1;
                                    app.filter_input.remove(app.cursor_position);
                                }
                            }
                            KeyCode::Delete => {
                                if app.cursor_position < app.filter_input.len() {
                                    app.filter_input.remove(app.cursor_position);
                                }
                            }
                            KeyCode::Left => {
                                if app.cursor_position > 0 {
                                    app.cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if app.cursor_position < app.filter_input.len() {
                                    app.cursor_position += 1;
                                }
                            }
                            KeyCode::Home => {
                                app.cursor_position = 0;
                            }
                            KeyCode::End => {
                                app.cursor_position = app.filter_input.len();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('f') => {
                                app.editing_filter = true;
                                app.cursor_position = app.filter_input.len();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Poll for new records
        app.poll_new_records().await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_parsing() {
        let filter = RecordFilter::parse("my_module=info,other=debug");
        assert_eq!(filter.filters.len(), 2);
        assert_eq!(filter.filters[0].0, "my_module");
        assert_eq!(filter.filters[0].1, Level::Info);
        assert_eq!(filter.filters[1].0, "other");
        assert_eq!(filter.filters[1].1, Level::Debug);
    }

    #[test]
    fn test_filter_parsing_with_default() {
        let filter = RecordFilter::parse("=warn,my_module=info");
        assert_eq!(filter.default_level, Some(Level::Warn));
        assert_eq!(filter.filters.len(), 1);
        assert_eq!(filter.filters[0].0, "my_module");
        assert_eq!(filter.filters[0].1, Level::Info);
    }

    #[test]
    fn test_record_filter_matching() {
        let filter = RecordFilter::parse("my_module=warn");

        let log_warn = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Warn),
            target: Some("my_module::submodule".to_string()),
            message: Some("test".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let log_info = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Info),
            target: Some("my_module::submodule".to_string()),
            message: Some("test".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let record_warn = RecordResponse::Log(log_warn);
        let record_info = RecordResponse::Log(log_info);

        assert!(record_warn.matches_filter(&filter));
        assert!(!record_info.matches_filter(&filter));
    }
}
