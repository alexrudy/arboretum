# Justfile for Arboretum development

# Default recipe - show available commands
default:
    @just --list

# Run all components in parallel for development
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' SIGINT
    cargo run --bin arboretum -- --config config.example.toml &
    cd web && npm run dev &
    cd arbor-gen && cargo run -- --quiet &
    wait

# Run the Arboretum backend server
dev-backend:
    cargo run --bin arboretum -- --config config.example.toml

# Run the frontend Vite development server
dev-frontend:
    cd web && npm run dev

# Run the trace generator
dev-gen:
    cd arbor-gen && cargo run

# Build all components
build: build-backend build-frontend build-gen

# Build the backend
build-backend:
    cargo build --bin arboretum

# Build the frontend
build-frontend:
    cd web && npm run build

# Build the generator
build-gen:
    cd arbor-gen && cargo build

# Run all tests
test: test-backend test-frontend

# Run backend tests
test-backend:
    cargo test

# Run frontend tests (if any)
test-frontend:
    cd web && npm test

# Clean build artifacts
clean:
    cargo clean
    cd web && rm -rf dist node_modules
    cd arbor-gen && cargo clean

# Install frontend dependencies
install:
    cd web && npm install

# Format all code
fmt:
    cargo fmt
    cd web && npm run format || true

# Run linters
lint:
    cargo clippy
    cd web && npm run lint || true

# Create a new config file from example
init-config:
    cp config.example.toml config.toml
    @echo "Created config.toml from example"

# Start backend and frontend (without generator)
serve:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' SIGINT
    cargo run --bin arboretum -- --config config.example.toml &
    cd web && npm run dev &
    wait

# Show status of all services
status:
    @echo "Checking Arboretum services..."
    @echo ""
    @echo "Backend (port 3333):"
    @lsof -i :3333 || echo "  Not running"
    @echo ""
    @echo "Frontend (port 5173):"
    @lsof -i :5173 || echo "  Not running"
