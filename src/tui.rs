use crate::handlers::{PaginatedResponse, RecordResponse};
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
use std::collections::{HashMap, HashSet};
use std::io;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

/// Extension trait to add filtering and formatting methods to RecordResponse
trait RecordResponseExt {
    fn level(&self) -> Option<Level>;
    fn target(&self) -> Option<&str>;
    fn matches_filter(&self, filter: &RecordFilter) -> bool;
    fn record_key(&self) -> RecordKey;
}

/// Unique key for deduplicating records
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum RecordKey {
    Log {
        timestamp: i64,
        trace_id: Option<String>,
    },
    Span {
        span_id: String,
    },
    Event {
        span_id: String,
        timestamp: i64,
    },
}

impl RecordResponseExt for RecordResponse {
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
        let record_level = self.level();
        let target = self.target();

        // Apply directives in order - first match wins
        for directive in &filter.directives {
            if let Some(result) = directive.matches(target, record_level) {
                return result;
            }
        }

        // No directives matched, show everything
        true
    }

    fn record_key(&self) -> RecordKey {
        match self {
            RecordResponse::Log(log) => RecordKey::Log {
                timestamp: log.timestamp,
                trace_id: log.trace_id.clone(),
            },
            RecordResponse::Span(span) => RecordKey::Span {
                span_id: span.span_id.clone(),
            },
            RecordResponse::Event(event) => RecordKey::Event {
                span_id: event.span_id.clone(),
                timestamp: event.timestamp,
            },
        }
    }
}

const PREFIXES: [&'static str; 2] = ["code.", ""];
const ATTRS: [&'static str; 4] = ["idle_ns", "busy_ns", "target", "level"];

fn match_attr(key: &str) -> bool {
    ATTRS.contains(&key) || PREFIXES.iter().any(|prefix| key.starts_with(prefix))
}

/// A single filter directive
#[derive(Debug, Clone)]
enum FilterDirective {
    /// Match a specific target prefix
    Target { prefix: String, level: Level },
    /// Match all records
    Global { level: Level },
}

impl FilterDirective {
    fn matches(&self, target: Option<&str>, record_level: Option<Level>) -> Option<bool> {
        match self {
            FilterDirective::Target { prefix, level } => {
                // Only matches if target starts with prefix
                if let Some(target_str) = target {
                    if target_str.starts_with(prefix) {
                        return Some(record_level.map_or(false, |lvl| lvl >= *level));
                    }
                }
                None // Doesn't match this target
            }
            FilterDirective::Global { level } => {
                // Matches all records
                Some(record_level.map_or(false, |lvl| lvl >= *level))
            }
        }
    }
}

/// Parses a filter string like "warn,my_module=debug"
#[derive(Debug, Clone, Default)]
struct RecordFilter {
    directives: Vec<FilterDirective>,
}

impl RecordFilter {
    fn parse(input: &str) -> Self {
        let mut directives = Vec::new();

        for part in input.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((target, level_str)) = part.split_once('=') {
                let target = target.trim();
                let level_str = level_str.trim();

                if let Ok(level) = Level::from_str(level_str) {
                    if target.is_empty() {
                        // =level is a global directive
                        directives.push(FilterDirective::Global { level });
                    } else {
                        // target=level
                        directives.push(FilterDirective::Target {
                            prefix: target.to_string(),
                            level,
                        });
                    }
                }
            } else {
                // No '=' means it's a bare level (global directive)
                if let Ok(level) = Level::from_str(part) {
                    directives.push(FilterDirective::Global { level });
                }
            }
        }

        Self { directives }
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
                        .filter(|(k, _)| !match_attr(k))
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
                        .filter(|(k, _)| !match_attr(k))
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
                        .filter(|(k, _)| !match_attr(k))
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

/// Shared state between UI and background poller
struct SharedState {
    records: Vec<RecordResponse>,
    formatted_lines: HashMap<RecordKey, Line<'static>>,
    seen_keys: HashSet<RecordKey>,
    max_records: usize,
}

impl SharedState {
    fn new(max_records: usize) -> Self {
        Self {
            records: Vec::new(),
            formatted_lines: HashMap::new(),
            seen_keys: HashSet::new(),
            max_records,
        }
    }

    fn add_records(&mut self, new_records: Vec<RecordResponse>) {
        for record in new_records {
            let key = record.record_key();

            // Deduplicate
            if self.seen_keys.contains(&key) {
                continue;
            }

            self.seen_keys.insert(key.clone());

            // Format and cache
            let line = format_record(&record);
            self.formatted_lines.insert(key, line);

            self.records.push(record);
        }

        // Trim old records
        while self.records.len() > self.max_records {
            if let Some(old_record) = self.records.first() {
                let old_key = old_record.record_key();
                self.seen_keys.remove(&old_key);
                self.formatted_lines.remove(&old_key);
            }
            self.records.remove(0);
        }

        // Keep sorted
        self.records.sort_by_key(|r| r.timestamp());
    }

    fn get_formatted_line(&self, key: &RecordKey) -> Option<&Line<'static>> {
        self.formatted_lines.get(key)
    }
}

/// Configuration for TUI behavior
#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// Number of records to fetch on initial load
    pub lookback: u64,
    /// Interval between background polls (milliseconds)
    pub poll_interval_ms: u64,
    /// Maximum number of records to keep in memory
    pub max_records: usize,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 500,
            max_records: 1000,
            lookback: 10,
        }
    }
}

struct ViewState {
    spans: bool,
    events: bool,
    logs: bool,
    editing_filter: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            spans: true,
            events: true,
            logs: true,
            editing_filter: false,
        }
    }
}

/// Main TUI application state
struct App {
    /// Shared state with background poller
    state: Arc<RwLock<SharedState>>,
    /// Input widget for filter
    filter_input: Input,
    /// Parsed filter
    filter: RecordFilter,
    /// View interactive state
    view: ViewState,
    /// Server URL
    server_url: String,
}

impl App {
    fn new(server_url: String, state: Arc<RwLock<SharedState>>) -> Self {
        Self {
            state,
            filter_input: Input::default(),
            filter: RecordFilter::default(),
            view: ViewState::default(),
            server_url,
        }
    }

    fn update_filter(&mut self) {
        self.filter = RecordFilter::parse(self.filter_input.value());
    }
}

/// Data for the background poller
struct BackgroundPoller {
    base_url: String,
    state: Arc<RwLock<SharedState>>,
    cursor: Option<i64>,
    config: TuiConfig,
    client: Client,
}

impl BackgroundPoller {
    fn new(base_url: String, state: Arc<RwLock<SharedState>>, config: TuiConfig) -> Self {
        Self {
            base_url,
            state,
            cursor: None,
            config,
            client: Client::new(),
        }
    }

    async fn poll_once(&mut self) -> Result<(), reqwest::Error> {
        // Poll for new records using timestamp-based cursor
        // Use lookback of 10 seconds to ensure we don't miss records during polling
        let url = if let Some(cursor) = self.cursor.as_ref() {
            format!("{}/api/v1/records?cursor={}", self.base_url, cursor)
        } else {
            format!(
                "{}/api/v1/records?lookback={}",
                self.base_url, self.config.lookback
            )
        };

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let paginated = response.json::<PaginatedResponse>().await?;

            tracing::debug!("fetched {} records", paginated.records.len());

            // Filter to only records newer than what we've seen
            let filtered: Vec<RecordResponse> = paginated
                .records
                .into_iter()
                .filter(|r| self.cursor.is_none_or(|c| r.timestamp() > c))
                .collect();

            if paginated.cursor.is_some() {
                self.cursor = paginated.cursor;
            }

            if !filtered.is_empty() {
                let mut state = self.state.write().await;
                state.add_records(filtered);
            }
        }

        Ok(())
    }

    async fn poll_task(&mut self, mut shutdown: mpsc::Receiver<()>) {
        let mut interval =
            tokio::time::interval(Duration::from_millis(self.config.poll_interval_ms));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(error) = self.poll_once().await {
                        tracing::error!("Error polling records: {error}");
                    }
                }

                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }
}

/// Runs the TUI application with default configuration
pub async fn run(server_url: String) -> Result<(), Box<dyn std::error::Error>> {
    run_with_config(server_url, TuiConfig::default()).await
}

/// Runs the TUI application with custom configuration
pub async fn run_with_config(
    server_url: String,
    config: TuiConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let state = Arc::new(RwLock::new(SharedState::new(config.max_records)));
    let mut app = App::new(server_url.clone(), Arc::clone(&state));

    // Start background polling task with the max timestamp from initial load
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let mut poller = BackgroundPoller::new(server_url, Arc::clone(&state), config);
    let poll_handle = tokio::spawn(async move {
        if let Err(error) = poller.poll_once().await {
            tracing::error!("Error polling: {error}");
        }
        poller.poll_task(shutdown_rx).await;
    });

    let res = run_app(&mut terminal, &mut app).await;

    // Stop background task
    let _ = shutdown_tx.send(()).await;
    let _ = poll_handle.await;

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
        // Read state once per frame
        let state = app.state.read().await;
        let record_count = state.records.len();

        // Collect filtered records with cached formatting
        let filtered_items: Vec<ListItem> = state
            .records
            .iter()
            .filter(|r| r.matches_filter(&app.filter))
            .filter(|r| match r {
                RecordResponse::Log(_) => app.view.logs,
                RecordResponse::Span(_) => app.view.spans,
                RecordResponse::Event(_) => app.view.events,
            })
            .filter_map(|r| {
                let key = r.record_key();
                state
                    .get_formatted_line(&key)
                    .map(|line| ListItem::new(line.clone()))
            })
            .collect();

        drop(state); // Release lock before drawing

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
                app.server_url, record_count
            ))
            .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(metadata, chunks[0]);

            // Records list
            let records_list = List::new(filtered_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Logs, Spans, and Events"),
            );
            f.render_widget(records_list, chunks[1]);

            // Filter input
            let filter_style = if app.view.editing_filter {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let filter_display = if app.view.editing_filter {
                app.filter_input.value().to_string()
            } else {
                if app.filter_input.value().is_empty() {
                    "none (press 'f' to edit)".to_string()
                } else {
                    format!("{} (press 'f' to edit)", app.filter_input.value())
                }
            };

            let filter_paragraph = Paragraph::new(filter_display)
                .style(filter_style)
                .block(Block::default().borders(Borders::ALL).title("Filter"));

            // Show cursor when editing
            if app.view.editing_filter {
                let cursor_pos = app.filter_input.visual_cursor() as u16;
                f.set_cursor_position((chunks[2].x + cursor_pos + 1, chunks[2].y + 1));
            }

            f.render_widget(filter_paragraph, chunks[2]);
        })?;

        // Handle events with a short timeout for responsiveness
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.view.editing_filter {
                        match key.code {
                            KeyCode::Enter => {
                                app.view.editing_filter = false;
                                app.update_filter();
                            }
                            KeyCode::Esc => {
                                app.view.editing_filter = false;
                            }
                            _ => {
                                // Let tui-input handle all other keys
                                app.filter_input.handle_event(&Event::Key(key));
                            }
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('f') => {
                                app.view.editing_filter = true;
                            }
                            KeyCode::Char('s') => {
                                app.view.spans = !app.view.spans;
                            }
                            KeyCode::Char('e') => {
                                app.view.events = !app.view.events;
                            }
                            KeyCode::Char('l') => app.view.logs = !app.view.logs,

                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::LogResponse;

    #[test]
    fn test_filter_parsing() {
        let filter = RecordFilter::parse("my_module=info,other=debug");
        assert_eq!(filter.directives.len(), 2);

        match &filter.directives[0] {
            FilterDirective::Target { prefix, level } => {
                assert_eq!(prefix, "my_module");
                assert_eq!(*level, Level::Info);
            }
            _ => panic!("Expected Target directive"),
        }

        match &filter.directives[1] {
            FilterDirective::Target { prefix, level } => {
                assert_eq!(prefix, "other");
                assert_eq!(*level, Level::Debug);
            }
            _ => panic!("Expected Target directive"),
        }
    }

    #[test]
    fn test_filter_parsing_with_global() {
        let filter = RecordFilter::parse("=warn,my_module=info");
        assert_eq!(filter.directives.len(), 2);

        match &filter.directives[0] {
            FilterDirective::Global { level } => {
                assert_eq!(*level, Level::Warn);
            }
            _ => panic!("Expected Global directive"),
        }

        match &filter.directives[1] {
            FilterDirective::Target { prefix, level } => {
                assert_eq!(prefix, "my_module");
                assert_eq!(*level, Level::Info);
            }
            _ => panic!("Expected Target directive"),
        }
    }

    #[test]
    fn test_filter_parsing_bare_level() {
        // Bare level (no '=') creates a global directive
        let filter = RecordFilter::parse("warn");
        assert_eq!(filter.directives.len(), 1);

        match &filter.directives[0] {
            FilterDirective::Global { level } => {
                assert_eq!(*level, Level::Warn);
            }
            _ => panic!("Expected Global directive"),
        }
    }

    #[test]
    fn test_filter_parsing_bare_with_targets() {
        // Bare level + target filters
        let filter = RecordFilter::parse("info,my_module=debug");
        assert_eq!(filter.directives.len(), 2);

        match &filter.directives[0] {
            FilterDirective::Global { level } => {
                assert_eq!(*level, Level::Info);
            }
            _ => panic!("Expected Global directive"),
        }

        match &filter.directives[1] {
            FilterDirective::Target { prefix, level } => {
                assert_eq!(prefix, "my_module");
                assert_eq!(*level, Level::Debug);
            }
            _ => panic!("Expected Target directive"),
        }
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

    #[test]
    fn test_record_filter_with_no_target() {
        // Records without targets should filter by default level
        let filter = RecordFilter::parse("warn");

        let log_error_no_target = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Error),
            target: None,
            message: Some("error message".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let log_info_no_target = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Info),
            target: None,
            message: Some("info message".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let record_error = RecordResponse::Log(log_error_no_target);
        let record_info = RecordResponse::Log(log_info_no_target);

        // ERROR >= WARN, so should pass
        assert!(record_error.matches_filter(&filter));
        // INFO < WARN, so should be filtered out
        assert!(!record_info.matches_filter(&filter));
    }

    #[test]
    fn test_record_filter_specific_before_global() {
        // When you want specific targets to override global, put them first
        let filter = RecordFilter::parse("my_module=debug,warn");

        let log_debug_my_module = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Debug),
            target: Some("my_module::test".to_string()),
            message: Some("test".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let log_debug_other = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Debug),
            target: Some("other_module".to_string()),
            message: Some("test".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let record_my_module = RecordResponse::Log(log_debug_my_module);
        let record_other = RecordResponse::Log(log_debug_other);

        // my_module=debug matches target directive first, allows DEBUG
        assert!(record_my_module.matches_filter(&filter));
        // other_module matches global warn directive, DEBUG < WARN
        assert!(!record_other.matches_filter(&filter));
    }

    #[test]
    fn test_directive_order_matters() {
        // Order matters: first matching directive wins

        // Global first, then specific
        let filter1 = RecordFilter::parse("debug,my_module=error");

        let log = LogResponse {
            timestamp: 0,
            service_name: None,
            level: Some(Level::Info),
            target: Some("my_module::test".to_string()),
            message: Some("test".to_string()),
            span_id: None,
            trace_id: None,
            attributes: None,
        };

        let record = RecordResponse::Log(log.clone());

        // First directive (global debug) matches, INFO >= DEBUG
        assert!(record.matches_filter(&filter1));

        // Specific first, then global
        let filter2 = RecordFilter::parse("my_module=error,debug");
        let record2 = RecordResponse::Log(log);

        // First directive (my_module=error) matches, INFO < ERROR
        assert!(!record2.matches_filter(&filter2));
    }
}
