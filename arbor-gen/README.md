# arbor-gen

A trace and log generator for testing Arboretum. This tool simulates a realistic application that generates OpenTelemetry traces and logs with nested spans, parallel operations, and various log levels.

## Features

- Generates realistic nested trace spans simulating:
  - User request handling
  - Database queries
  - Parallel operations (fetching related items)
  - Background jobs
  - Metric calculations
- Exports traces via OpenTelemetry Protocol (OTLP) over HTTP
- Runs indefinitely until interrupted with Ctrl+C
- Configurable endpoint, service name, and delay between iterations
- Includes both successful operations and simulated failures
- Uses structured logging with tracing
- Graceful shutdown with trace flushing

## Usage

Run with default settings (sends to `http://localhost:3333`, runs indefinitely):

```bash
cargo run
```

The generator will run continuously until you press Ctrl+C. Customize the behavior with CLI arguments:

```bash
cargo run -- \
  --endpoint http://localhost:3333 \
  --service my-app \
  --delay 500
```

## CLI Arguments

- `-e, --endpoint <URL>`: OTLP endpoint URL (default: `http://localhost:3333`)
- `-s, --service <NAME>`: Service name for traces (default: `arbor-gen`)
- `-d, --delay <MS>`: Delay between iterations in milliseconds (default: `1000`)
- `-q, --quiet`: Disable console logging (only export to OpenTelemetry)

## Example

Run continuously with 200ms delay between iterations, sending to a custom endpoint:

```bash
cargo run -- -e http://192.168.1.100:3000 -s webapp -d 200
```

Press Ctrl+C to stop the generator and gracefully shut down.

## Simulated Operations

Each iteration simulates:

1. **Multiple User Requests** (3 concurrent requests):
   - Process user data (fetch from database)
   - Fetch related items in parallel (books, movies, music)
   - Calculate user metrics

2. **Background Job**:
   - Cleanup expired sessions

The generator includes:
- Random delays to simulate realistic processing times
- 10% chance of database query failures
- Various log levels (debug, info, warn, error)
- Nested spans showing parent-child relationships
- Parallel span execution

## Integration with Arboretum

To use with Arboretum:

1. Start Arboretum:
   ```bash
   cargo run --bin arboretum -- --config config.example.toml
   ```

2. In another terminal, run arbor-gen:
   ```bash
   cd arbor-gen
   cargo run
   ```

3. View the generated traces in the Arboretum web UI at `http://localhost:5173`

## Environment Variables

You can control the tracing output level with the `RUST_LOG` environment variable:

```bash
RUST_LOG=arbor_gen=trace,debug cargo run
```
