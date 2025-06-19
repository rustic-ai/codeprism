# CodePrism Development Makefile
# 
# This Makefile provides convenient commands for development, testing, and deployment.
# Run 'make help' to see all available commands.

.PHONY: help build test check fmt lint doc clean dev-up dev-down dev-logs coverage bench install

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

# Project configuration
PROJECT_NAME := codeprism
RUST_VERSION := 1.82
DOCKER_COMPOSE := docker-compose

help: ## Show this help message
	@echo "$(CYAN)CodePrism Development Commands$(RESET)"
	@echo ""
	@echo "$(GREEN)Development:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-15s$(RESET) %s\n", $$1, $$2}' | \
		grep -E "(build|test|check|fmt|lint|doc|clean)"
	@echo ""
	@echo "$(GREEN)Services:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-15s$(RESET) %s\n", $$1, $$2}' | \
		grep -E "(dev-|coverage|bench)"
	@echo ""
	@echo "$(GREEN)Utilities:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-15s$(RESET) %s\n", $$1, $$2}' | \
		grep -E "(install|setup|release)"

# Development Commands

build: ## Build all crates
	@echo "$(GREEN)Building all crates...$(RESET)"
	cargo build --all

build-release: ## Build all crates in release mode
	@echo "$(GREEN)Building all crates (release)...$(RESET)"
	cargo build --all --release

test: ## Run all tests
	@echo "$(GREEN)Running tests...$(RESET)"
	cargo test --all

test-js: ## Run JavaScript parser tests only
	@echo "$(GREEN)Running JavaScript parser tests...$(RESET)"
	cargo test -p codeprism-lang-js

test-core: ## Run core library tests only
	@echo "$(GREEN)Running core library tests...$(RESET)"
	cargo test --workspace

test-integration: ## Run integration tests
	@echo "$(GREEN)Running integration tests...$(RESET)"
	cargo test --test integration_test

check: fmt lint test ## Run all quality checks (format, lint, test)
	@echo "$(GREEN)All checks passed!$(RESET)"

fmt: ## Format code with rustfmt
	@echo "$(GREEN)Formatting code...$(RESET)"
	cargo fmt --all

fmt-check: ## Check code formatting without modifying files
	@echo "$(GREEN)Checking code formatting...$(RESET)"
	cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo "$(GREEN)Running clippy...$(RESET)"
	cargo clippy --all-targets --all-features -- -D warnings

lint-fix: ## Run clippy with automatic fixes
	@echo "$(GREEN)Running clippy with fixes...$(RESET)"
	cargo clippy --all-targets --all-features --fix -- -D warnings

doc: ## Generate documentation
	@echo "$(GREEN)Generating documentation...$(RESET)"
	cargo doc --no-deps --all-features

doc-open: ## Generate and open documentation
	@echo "$(GREEN)Generating and opening documentation...$(RESET)"
	cargo doc --no-deps --all-features --open

clean: ## Clean build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(RESET)"
	cargo clean

# Development Services

dev-up: ## Start development services (Neo4j, Kafka, Redis)
	@echo "$(GREEN)Starting development services...$(RESET)"
	$(DOCKER_COMPOSE) up -d neo4j kafka redis zookeeper
	@echo "$(GREEN)Services started. Waiting for readiness...$(RESET)"
	@sleep 10
	@echo "$(GREEN)Services ready:$(RESET)"
	@echo "  Neo4j:  http://localhost:7474 (neo4j/password)"
	@echo "  Kafka:  localhost:9092"
	@echo "  Redis:  localhost:6379"

dev-down: ## Stop development services
	@echo "$(GREEN)Stopping development services...$(RESET)"
	$(DOCKER_COMPOSE) down

dev-restart: dev-down dev-up ## Restart development services

dev-logs: ## Show logs from development services
	@echo "$(GREEN)Showing service logs...$(RESET)"
	$(DOCKER_COMPOSE) logs -f

dev-status: ## Show status of development services
	@echo "$(GREEN)Development services status:$(RESET)"
	$(DOCKER_COMPOSE) ps

# Testing and Quality

coverage: ## Generate test coverage report
	@echo "$(GREEN)Generating coverage report...$(RESET)"
	cargo tarpaulin --out Html --all-features --workspace
	@echo "$(GREEN)Coverage report generated: tarpaulin-report.html$(RESET)"

coverage-open: coverage ## Generate and open coverage report
	@echo "$(GREEN)Opening coverage report...$(RESET)"
	@if command -v xdg-open > /dev/null; then \
		xdg-open tarpaulin-report.html; \
	elif command -v open > /dev/null; then \
		open tarpaulin-report.html; \
	else \
		echo "$(YELLOW)Please open tarpaulin-report.html manually$(RESET)"; \
	fi

bench: ## Run performance benchmarks
	@echo "$(GREEN)Running benchmarks...$(RESET)"
	cargo bench

bench-js: ## Run JavaScript parser benchmarks
	@echo "$(GREEN)Running JavaScript parser benchmarks...$(RESET)"
	cargo bench -p codeprism-lang-js

# Development Tools

watch: ## Watch for changes and run tests
	@echo "$(GREEN)Watching for changes...$(RESET)"
	cargo watch -x "test --all"

watch-check: ## Watch for changes and run checks
	@echo "$(GREEN)Watching for changes (with checks)...$(RESET)"
	cargo watch -x "check" -x "test --all" -x "clippy --all-targets --all-features"

expand: ## Expand macros for debugging
	@echo "$(GREEN)Expanding macros...$(RESET)"
	cargo expand --package codeprism-core

expand-js: ## Expand macros for JavaScript parser
	@echo "$(GREEN)Expanding macros (JavaScript parser)...$(RESET)"
	cargo expand --package codeprism-lang-js

# Installation and Setup

install: ## Install development dependencies
	@echo "$(GREEN)Installing development dependencies...$(RESET)"
	@echo "$(CYAN)Installing Rust toolchain components...$(RESET)"
	rustup component add rustfmt clippy
	@echo "$(CYAN)Installing cargo tools...$(RESET)"
	cargo install cargo-tarpaulin cargo-watch cargo-expand
	@echo "$(GREEN)Development dependencies installed!$(RESET)"

setup: install dev-up ## Complete development setup
	@echo "$(GREEN)Development environment setup complete!$(RESET)"
	@echo ""
	@echo "$(CYAN)Next steps:$(RESET)"
	@echo "  1. Run 'make check' to verify everything works"
	@echo "  2. Run 'make doc-open' to view documentation"
	@echo "  3. Run 'make watch' to start development"

# Database Management

db-reset: ## Reset Neo4j database
	@echo "$(GREEN)Resetting Neo4j database...$(RESET)"
	$(DOCKER_COMPOSE) stop neo4j
	$(DOCKER_COMPOSE) rm -f neo4j
	docker volume rm codeprism_neo4j_data 2>/dev/null || true
	$(DOCKER_COMPOSE) up -d neo4j
	@echo "$(GREEN)Neo4j database reset complete$(RESET)"

db-backup: ## Backup Neo4j database
	@echo "$(GREEN)Backing up Neo4j database...$(RESET)"
	mkdir -p backups
	$(DOCKER_COMPOSE) exec neo4j neo4j-admin database dump --to-path=/backups neo4j
	docker cp $$($(DOCKER_COMPOSE) ps -q neo4j):/backups/neo4j.dump backups/neo4j-$$(date +%Y%m%d-%H%M%S).dump
	@echo "$(GREEN)Database backup complete$(RESET)"

# Release Management

version-check: ## Check current version
	@echo "$(GREEN)Current version information:$(RESET)"
	@grep '^version' Cargo.toml | head -1
	@echo "Rust version: $$(rustc --version)"

release-check: ## Check if ready for release
	@echo "$(GREEN)Checking release readiness...$(RESET)"
	@echo "$(CYAN)Running full test suite...$(RESET)"
	cargo test --all --release
	@echo "$(CYAN)Checking documentation...$(RESET)"
	cargo doc --no-deps --all-features
	@echo "$(CYAN)Running clippy...$(RESET)"
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(CYAN)Checking formatting...$(RESET)"
	cargo fmt --all -- --check
	@echo "$(GREEN)Release checks passed!$(RESET)"

# Docker Management

docker-build: ## Build Docker image
	@echo "$(GREEN)Building Docker image...$(RESET)"
	docker build -t $(PROJECT_NAME):latest .

docker-run: ## Run Docker container
	@echo "$(GREEN)Running Docker container...$(RESET)"
	docker run --rm -it \
		--network codeprism_default \
		-e NEO4J_URI=bolt://neo4j:7687 \
		-e KAFKA_BROKERS=kafka:9092 \
		-e REDIS_URL=redis://redis:6379 \
		$(PROJECT_NAME):latest

# Utility Commands

deps: ## Show dependency tree
	@echo "$(GREEN)Dependency tree:$(RESET)"
	cargo tree

deps-outdated: ## Check for outdated dependencies
	@echo "$(GREEN)Checking for outdated dependencies...$(RESET)"
	cargo outdated

audit: ## Security audit
	@echo "$(GREEN)Running security audit...$(RESET)"
	cargo audit

bloat: ## Analyze binary size
	@echo "$(GREEN)Analyzing binary size...$(RESET)"
	cargo bloat --release

# Language-specific commands

js-test: test-js ## Alias for test-js
js-bench: bench-js ## Alias for bench-js

# CI/CD simulation

ci: ## Simulate CI pipeline
	@echo "$(GREEN)Simulating CI pipeline...$(RESET)"
	@echo "$(CYAN)Step 1: Format check...$(RESET)"
	make fmt-check
	@echo "$(CYAN)Step 2: Lint check...$(RESET)"
	make lint
	@echo "$(CYAN)Step 3: Build...$(RESET)"
	make build
	@echo "$(CYAN)Step 4: Test...$(RESET)"
	make test
	@echo "$(CYAN)Step 5: Documentation...$(RESET)"
	make doc
	@echo "$(GREEN)CI pipeline completed successfully!$(RESET)"

# Performance testing

perf: ## Run performance tests
	@echo "$(GREEN)Running performance tests...$(RESET)"
	cargo test --release --test performance_test

flamegraph: ## Generate flamegraph for profiling
	@echo "$(GREEN)Generating flamegraph...$(RESET)"
	cargo flamegraph --bin codeprism-mcp

# Maintenance

update: ## Update dependencies
	@echo "$(GREEN)Updating dependencies...$(RESET)"
	cargo update

fix: ## Apply automatic fixes
	@echo "$(GREEN)Applying automatic fixes...$(RESET)"
	cargo fix --all-targets --all-features
	cargo clippy --all-targets --all-features --fix

# Environment info

env: ## Show environment information
	@echo "$(GREEN)Environment Information:$(RESET)"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Docker version: $$(docker --version 2>/dev/null || echo 'Not installed')"
	@echo "Docker Compose version: $$(docker-compose --version 2>/dev/null || echo 'Not installed')"
	@echo "OS: $$(uname -s)"
	@echo "Architecture: $$(uname -m)"

# Quick development workflow

quick: fmt lint test ## Quick development check (format, lint, test)

full: clean build test doc coverage ## Full development check

# Help for specific areas

help-dev: ## Show development-specific help
	@echo "$(CYAN)Development Workflow:$(RESET)"
	@echo "  1. make setup          # Initial setup"
	@echo "  2. make watch          # Start development"
	@echo "  3. make quick          # Quick checks"
	@echo "  4. make full           # Full validation"

help-services: ## Show services help
	@echo "$(CYAN)Development Services:$(RESET)"
	@echo "  make dev-up            # Start all services"
	@echo "  make dev-status        # Check service status"
	@echo "  make dev-logs          # View service logs"
	@echo "  make dev-down          # Stop all services"

# Error handling
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c

# Ensure required tools are available
check-tools:
	@command -v cargo >/dev/null 2>&1 || { echo "$(RED)Error: cargo is required but not installed$(RESET)" >&2; exit 1; }
	@command -v docker >/dev/null 2>&1 || { echo "$(RED)Error: docker is required but not installed$(RESET)" >&2; exit 1; }
	@command -v docker-compose >/dev/null 2>&1 || { echo "$(RED)Error: docker-compose is required but not installed$(RESET)" >&2; exit 1; }

# Make all development commands depend on tool check
build test check fmt lint doc: check-tools
dev-up dev-down dev-logs: check-tools 