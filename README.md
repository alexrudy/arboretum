# Arboretum

OpenTelemetry log and trace caching server for short-term analysis.

## Features

- **OTLP Ingestion**: Accepts OpenTelemetry logs and traces via OTLP protocol
- **SQLite Storage**: Efficient on-disk storage with automatic cleanup
- **Query API**: REST API for querying logs and spans with filtering
- **Recursive Span Retrieval**: Automatically returns child spans when querying
- **Configurable Retention**: Time-based and count-based retention policies
- **Rust Tracing Optimized**: Designed for rust `tracing` library output

## Quick Start

### Installation

```bash
cargo build --release
```

### Running with Configuration File

Create a `config.toml` file (see `config.example.toml`):

```toml
listen_addr = "0.0.0.0:3333"
db_path = "arboretum.db"
max_retention_seconds = 7200
cleanup_interval_seconds = 300
max_logs = 100000
max_spans = 50000
```

Then run:

```bash
./target/release/arboretum --config config.toml
```

### Running with Defaults

```bash
./target/release/arboretum
```

This uses default values and environment variable overrides.

## Configuration

### Configuration File (TOML)

Use the `--config` or `-c` flag to specify a TOML configuration file:

```bash
arboretum --config /path/to/config.toml
```

Available options:

- `listen_addr` (string): Address to bind the server (default: `"0.0.0.0:3333"`)
- `db_path` (string): Path to SQLite database file (default: `"arboretum.db"`)
- `max_retention_seconds` (integer): Maximum age of records in seconds (default: `7200` - 2 hours)
- `cleanup_interval_seconds` (integer): Interval between cleanup runs (default: `300` - 5 minutes)
- `max_logs` (integer, optional): Maximum number of log records to keep
- `max_spans` (integer, optional): Maximum number of span records to keep

### Environment Variable Overrides

Environment variables override config file values:

- `ARBORETUM_LISTEN_ADDR`
- `ARBORETUM_DB_PATH`
- `ARBORETUM_MAX_RETENTION_SECONDS`
- `ARBORETUM_CLEANUP_INTERVAL_SECONDS`
- `ARBORETUM_MAX_LOGS`
- `ARBORETUM_MAX_SPANS`

Example:

```bash
ARBORETUM_LISTEN_ADDR=127.0.0.1:8080 arboretum --config config.toml
```

### Logging

Set the `RUST_LOG` environment variable to control log verbosity:

```bash
RUST_LOG=arboretum=debug,tower_http=debug arboretum --config config.toml
```

## API Endpoints

### OTLP Ingestion

- **POST `/v1/logs`**: Accept OTLP log export requests (protobuf)
- **POST `/v1/traces`**: Accept OTLP trace export requests (protobuf)

### Query API

- **GET `/api/v1/logs`**: Query logs with filtering
  - Query parameters:
    - `service_name`: Filter by service name
    - `level`: Filter by level (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
    - `target`: Filter by rust module target
    - `message`: Filter by message content (partial match)
    - `span_id`: Filter by span ID
    - `limit`: Maximum number of results (default: 1000)

- **GET `/api/v1/spans`**: Query spans with recursive child retrieval
  - Query parameters:
    - `service_name`: Filter by service name
    - `level`: Filter by level
    - `target`: Filter by rust module target
    - `span_id`: Filter by span ID (also returns all child spans)
    - `trace_id`: Filter by trace ID
    - `limit`: Maximum number of initial results (default: 1000)

- **GET `/api/v1/metadata`**: Get database statistics
  - Returns: Total counts, oldest/newest timestamps, database size

### Example Queries

```bash
# Query all ERROR logs
curl "http://localhost:3333/api/v1/logs?level=ERROR"

# Query logs for a specific service
curl "http://localhost:3333/api/v1/logs?service_name=my-service"

# Query spans for a trace (includes all child spans recursively)
curl "http://localhost:3333/api/v1/spans?trace_id=abc123"

# Get database statistics
curl "http://localhost:3333/api/v1/metadata"
```

## OpenTelemetry Integration

### Rust Application Example

```rust
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:3333")
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Your application code here
}
```

## Development

### Running Tests

Run all tests (unit + integration):
```bash
cargo test
```

Run only unit tests:
```bash
cargo test --lib
```

Run only integration tests:
```bash
cargo test --test integration_test
```

The integration tests start a real HTTP server and test:
- Server startup and health check
- OTLP log ingestion and querying
- OTLP trace ingestion and querying
- Log filtering by level

### Building

```bash
cargo build
```

Release build:
```bash
cargo build --release
```

## License

See LICENSE file for details.
