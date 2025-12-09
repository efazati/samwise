# Makefile for Samwise
# A lightweight desktop utility for text transformation with LLMs

.PHONY: help dev build clean install install-deps install-rust check test format lint setup prod release run-frontend kill

# Default target - show help
help:
	@echo "Samwise - Desktop App Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start development server (hot reload)"
	@echo "  make kill         - Stop all running Samwise processes"
	@echo "  make run-frontend - Run frontend only (Vite dev server)"
	@echo "  make check        - Check Rust code without building"
	@echo "  make format       - Format Rust and TypeScript code"
	@echo "  make lint         - Run linters"
	@echo ""
	@echo "Building:"
	@echo "  make build        - Build production desktop binaries"
	@echo "  make prod         - Alias for build"
	@echo "  make release      - Build optimized release binary"
	@echo ""
	@echo "Setup:"
	@echo "  make setup        - Complete setup (install deps)"
	@echo "  make install      - Install npm dependencies"
	@echo "  make install-rust - Install Rust toolchain"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make clean-all    - Clean everything (including node_modules)"
	@echo "  make update       - Update dependencies"
	@echo ""
	@echo "Information:"
	@echo "  make info         - Show project information"
	@echo "  make help         - Show this help message"

# Ensure Rust/Cargo is in PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Development - start Tauri app with hot reload
dev:
	@echo "🚀 Starting Samwise in development mode..."
	@npm run tauri dev

# Kill all running Samwise processes
kill:
	@echo "🛑 Stopping all Samwise processes..."
	-@pkill -f "target/debug/samwise" 2>/dev/null || true
	-@pkill -f "vite" 2>/dev/null || true
	-@pkill -f "cargo.*run.*samwise" 2>/dev/null || true
	-@pkill -f "tauri dev" 2>/dev/null || true
	@sleep 2
	@echo "✅ All processes stopped"

# Run frontend only (Vite dev server)
run-frontend:
	@echo "🌐 Starting Vite dev server..."
	@npm run dev

# Build production binaries
build:
	@echo "📦 Building Samwise for production..."
	@npm run tauri build

# Alias for build
prod: build

# Build release binary only (no installer)
release:
	@echo "🔨 Building release binary..."
	@cd src-tauri && cargo build --release

# Install npm dependencies
install:
	@echo "📥 Installing npm dependencies..."
	@npm install

# Install Rust toolchain
install-rust:
	@echo "🦀 Installing Rust toolchain..."
	@if ! command -v rustc > /dev/null 2>&1; then \
		echo "Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "Rust installed successfully!"; \
	else \
		echo "Rust is already installed."; \
		rustc --version; \
	fi

# Complete setup
setup: install-rust install
	@echo "✅ Setup complete! Run 'make dev' to start development."

# Check Rust code without building
check:
	@echo "🔍 Checking Rust code..."
	@cd src-tauri && cargo check

# Format code
format:
	@echo "✨ Formatting code..."
	@cd src-tauri && cargo fmt
	@npm run build > /dev/null 2>&1 || true
	@echo "Code formatted!"

# Run linters
lint:
	@echo "🔎 Running linters..."
	@cd src-tauri && cargo clippy -- -D warnings
	@npm run build
	@echo "Linting complete!"

# Run tests
test:
	@echo "🧪 Running tests..."
	@cd src-tauri && cargo test
	@echo "Tests complete!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cd src-tauri && cargo clean
	@rm -rf dist
	@rm -rf src-tauri/target
	@echo "Clean complete!"

# Clean everything including dependencies
clean-all: clean
	@echo "🧹 Cleaning all dependencies..."
	@rm -rf node_modules
	@rm -rf package-lock.json
	@echo "Deep clean complete!"

# Update dependencies
update:
	@echo "⬆️  Updating dependencies..."
	@npm update
	@cd src-tauri && cargo update
	@echo "Dependencies updated!"

# Show project information
info:
	@echo "📊 Samwise Project Information"
	@echo ""
	@echo "Node.js version: $(shell node --version 2>/dev/null || echo 'Not installed')"
	@echo "npm version: $(shell npm --version 2>/dev/null || echo 'Not installed')"
	@echo "Rust version: $(shell rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "Cargo version: $(shell cargo --version 2>/dev/null || echo 'Not installed')"
	@echo ""
	@echo "Project details:"
	@echo "  Name: Samwise"
	@echo "  Version: 0.1.0"
	@echo "  Frontend: React + TypeScript"
	@echo "  Backend: Rust + Tauri v2"
	@echo "  Build Tool: Vite"
	@echo ""
	@if [ -d "node_modules" ]; then \
		echo "✅ npm dependencies installed"; \
	else \
		echo "❌ npm dependencies not installed (run: make install)"; \
	fi
	@if [ -d "src-tauri/target" ]; then \
		echo "✅ Rust project has been built"; \
	else \
		echo "ℹ️  Rust project not built yet (run: make dev or make build)"; \
	fi

# Quick alias targets
.PHONY: start run
start: dev
run: dev

