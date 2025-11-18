# Storehaus Development Makefile

.PHONY: help docker-up docker-down docker-restart run-example test clean build \
        example-complete example-models example-system-fields example-querying \
        example-caching example-signals example-tags examples-all

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
	cargo run --example demo

demo-with-cache: ## Run demo with Redis cache support
	@echo "🚀 Running Storehaus demo with cache..."
	@echo "📋 Starting database and Redis..."
	docker compose up -d postgres redis
	@echo "⏳ Waiting for services to be ready..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	cargo run --example demo

test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test

test-storehaus: ## Run StoreHaus tests only
	@echo "🧪 Running StoreHaus tests..."
	cargo test

test-integration: ## Run integration tests with fresh PostgreSQL in Docker
	@echo "🧪 Running integration tests with fresh database..."
	@echo "🛑 Stopping existing test containers..."
	-docker compose -f docker-compose.test.yml down -v 2>/dev/null
	@echo "🐳 Starting fresh PostgreSQL on port 5433..."
	docker compose -f docker-compose.test.yml up -d postgres-test
	@echo "⏳ Waiting for database to be ready..."
	@until docker compose -f docker-compose.test.yml exec -T postgres-test pg_isready -U postgres -d storehaus_test 2>/dev/null; do \
		sleep 1; \
	done
	@echo "✅ Database is ready!"
	@echo "🧪 Running integration tests..."
	DATABASE_URL=postgres://postgres:password@localhost:5433/storehaus_test cargo test --test json_types_test -- --test-threads=1
	@echo "🛑 Cleaning up..."
	docker compose -f docker-compose.test.yml down -v
	@echo "✅ Integration tests complete!"

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

# StoreHaus Examples
example-complete: ## Run complete integration example (e-commerce demo with all features)
	@echo "🏪 Running Complete Integration Example..."
	@echo "📋 This showcases all StoreHaus features in a full e-commerce platform"
	@echo "🚀 Starting database and Redis..."
	docker compose up -d postgres redis
	@echo "⏳ Waiting for services to be ready..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	@echo "🎯 Running example..."
	cargo run --example ecommerce_demo

example-models: ## Run model definitions showcase
	@echo "📝 Running Model Definitions Example..."
	@echo "📋 Demonstrates various model types, attributes, and system fields"
	@make docker-up
	cargo run --example 02_model_definition

example-system-fields: ## Run system fields demonstration
	@echo "🔧 Running System Fields Example..."
	@echo "📋 Shows automatic timestamps, tags, soft delete, and system field queries"
	@make docker-up
	cargo run --example 01_basic_usage

example-querying: ## Run advanced querying showcase
	@echo "🔍 Running Advanced Querying Example..."
	@echo "📋 Complex filters, sorting, pagination, tag-based searches, and performance"
	@make docker-up
	cargo run --example tags_demo

example-caching: ## Run caching system demonstration
	@echo "🚀 Running Caching System Example..."
	@echo "📋 Redis cache setup, TTL strategies, performance comparisons, error resilience"
	@echo "🚀 Starting database and Redis..."
	docker compose up -d postgres redis
	@echo "⏳ Waiting for services..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	cargo run --example caching_basic

example-signals: ## Run signal system showcase
	@echo "📡 Running Signal System Example..."
	@echo "📋 Event monitoring, WAL implementation, metrics, audit trail, notifications"
	@make docker-up
	cargo run --example signals_basic

example-tags: ## Run tags system demonstration
	@echo "🏷️  Running Tags System Example..."
	@echo "📋 Operation tagging, grouping, tag-based queries, business context tracking"
	@make docker-up
	cargo run --example tags_demo

examples-all: ## Run all examples in sequence
	@echo "🎊 Running All StoreHaus Examples..."
	@echo "📋 This will demonstrate every aspect of the system"
	@echo "🚀 Starting all required services..."
	docker compose up -d postgres redis
	@echo "⏳ Waiting for services to be ready..."
	docker compose exec postgres bash -c 'while ! pg_isready -U postgres -d storehaus; do sleep 1; done'
	@echo ""
	@echo "1️⃣  Model Definitions..."
	cargo run --example 02_model_definition
	@echo ""
	@echo "2️⃣  System Fields..."
	cargo run --example 01_basic_usage
	@echo ""
	@echo "3️⃣  Advanced Querying..."
	cargo run --example tags_demo
	@echo ""
	@echo "4️⃣  Caching System..."
	cargo run --example caching_basic
	@echo ""
	@echo "5️⃣  Signal System..."
	cargo run --example signals_basic
	@echo ""
	@echo "6️⃣  Tags System..."
	cargo run --example tags_demo
	@echo ""
	@echo "7️⃣  Complete Integration..."
	cargo run --example ecommerce_demo
	@echo ""
	@echo "🎉 All examples completed!"

examples-help: ## Show detailed information about all examples
	@echo "📚 StoreHaus Examples Guide"
	@echo "=========================="
	@echo ""
	@echo "🏪 Complete Integration (example-complete):"
	@echo "   Full e-commerce platform with all features working together"
	@echo "   • Multi-model domain (Users, Products, Orders, Inventory)"
	@echo "   • Business logic automation and real-time notifications"
	@echo "   • WAL implementation and comprehensive analytics"
	@echo ""
	@echo "📝 Model Definitions (example-models):"
	@echo "   Different types of models and field attributes"
	@echo "   • Basic, complex, and soft-delete models"
	@echo "   • Field attributes and system fields behavior"
	@echo "   • Model migration and usage patterns"
	@echo ""
	@echo "🔧 System Fields (example-system-fields):"
	@echo "   Automatic system field management"
	@echo "   • Timestamps (__created_at__, __updated_at__)"
	@echo "   • Tags (__tags__) and soft delete (__is_active__)"
	@echo "   • System field querying and indexing"
	@echo ""
	@echo "🔍 Advanced Querying (example-querying):"
	@echo "   Complex database queries and filtering"
	@echo "   • Filters, sorting, pagination, and aggregation"
	@echo "   • Tag-based searching and performance optimization"
	@echo "   • Real-world query patterns"
	@echo ""
	@echo "🚀 Caching System (example-caching):"
	@echo "   Redis caching integration and strategies"
	@echo "   • Cache configuration and TTL strategies"
	@echo "   • Performance comparisons and error resilience"
	@echo "   • Automatic cache invalidation"
	@echo ""
	@echo "📡 Signal System (example-signals):"
	@echo "   Database event monitoring and processing"
	@echo "   • Event handlers and WAL implementation"
	@echo "   • Real-time metrics and audit trails"
	@echo "   • Business notifications and integrations"
	@echo ""
	@echo "🏷️  Tags System (example-tags):"
	@echo "   Operation tagging and categorization"
	@echo "   • Tag-based queries and business context"
	@echo "   • Operation grouping and tracking"
	@echo ""
	@echo "Usage:"
	@echo "  make example-complete    # Best starting point"
	@echo "  make examples-all        # Run all examples"
	@echo "  make example-<name>      # Run specific example"

# Development workflow
dev: docker-up ## Start development environment
	@echo "🚀 Development environment ready!"
	@echo "   Database: postgres://postgres:password@localhost:5432/storehaus"
	@echo "   Run examples: make examples-help"
	@echo "   Quick start: make example-complete"
	@echo "   PgAdmin: make pgadmin-up"

setup: ## Initial setup for development
	@echo "⚙️  Setting up development environment..."
	@echo "🔨 Building project..."
	@make build
	@echo "🐳 Starting database..."
	@make docker-up
	@echo "✅ Setup complete! Run 'make demo' to test."