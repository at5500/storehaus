# Storehaus Development Makefile

.PHONY: help docker-up docker-down docker-restart run-example test clean build

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

docker-up: ## Start PostgreSQL database in Docker
	@echo "🐳 Starting PostgreSQL database..."
	docker compose up -d postgres
	@echo "⏳ Waiting for database to be ready..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	@echo "✅ Database is ready!"

docker-down: ## Stop and remove Docker containers
	@echo "🛑 Stopping Docker containers..."
	docker compose down

docker-restart: docker-down docker-up ## Restart Docker containers

docker-logs: ## Show PostgreSQL logs
	docker compose logs -f postgres

pgadmin-up: ## Start PgAdmin (database management tool)
	@echo "🔧 Starting PgAdmin..."
	docker compose --profile tools up -d pgadmin
	@echo "🌐 PgAdmin available at http://localhost:5050"
	@echo "   Email: admin@storehaus.local"
	@echo "   Password: admin"

demo: ## Run demo example (requires database)
	@echo "🚀 Running Storehaus demo..."
	@echo "📋 Make sure database is running with: make docker-up"
	cd dispatcher && cargo run --example demo

demo-with-cache: ## Run demo with Redis cache support
	@echo "🚀 Running Storehaus demo with cache..."
	@echo "📋 Starting database and Redis..."
	docker compose up -d postgres redis
	@echo "⏳ Waiting for services to be ready..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	cd dispatcher && cargo run --example demo

test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test

test-dispatcher: ## Run dispatcher tests only
	@echo "🧪 Running dispatcher tests..."
	cd dispatcher && cargo test

build: ## Build all crates
	@echo "🔨 Building project..."
	cargo build

build-release: ## Build in release mode
	@echo "🔨 Building project in release mode..."
	cargo build --release

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

format: ## Format code
	@echo "🎨 Formatting code..."
	cargo fmt

lint: ## Run linting
	@echo "🔍 Running linting..."
	cargo clippy -- -D warnings

check: ## Run all checks (format, lint, test)
	@echo "🔍 Running all checks..."
	@make format
	@make lint
	@make test

db-connect: ## Connect to PostgreSQL database via psql
	@echo "🔌 Connecting to database..."
	docker compose exec postgres psql -U postgres -d storehaus

db-reset: ## Reset database (WARNING: destroys all data)
	@echo "⚠️  Resetting database - this will destroy all data!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker compose down -v; \
		docker compose up -d postgres; \
		echo "✅ Database reset complete!"; \
	else \
		echo "❌ Database reset cancelled."; \
	fi

demo-full: demo-with-cache ## Start database + Redis and run full demo

# Development workflow
dev: docker-up ## Start development environment
	@echo "🚀 Development environment ready!"
	@echo "   Database: postgres://postgres:password@localhost:5432/storehaus"
	@echo "   Run demo: make demo"
	@echo "   With cache: make demo-with-cache"
	@echo "   PgAdmin: make pgadmin-up"

setup: ## Initial setup for development
	@echo "⚙️  Setting up development environment..."
	@echo "🔨 Building project..."
	@make build
	@echo "🐳 Starting database..."
	@make docker-up
	@echo "✅ Setup complete! Run 'make demo' to test."