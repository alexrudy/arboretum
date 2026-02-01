# arbor-gen

A trace and log generator for testing Arboretum. This tool simulates a realistic application that generates OpenTelemetry traces and logs with nested spans, parallel operations, and various log levels.


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
