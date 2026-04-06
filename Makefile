.PHONY: help dev stop test lint audit clean build docker-build docker-stop docker-clean logs

# Default target
help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:##? .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development Commands
dev: ## Start development stack (PostgreSQL + Meilisearch + API + NLP)
	@if [ ! -f .env ]; then \
		echo "Creating .env file from template..."; \
		cp .env.example .env; \
		echo "✅ .env file created. Please edit it with your configuration."; \
	fi
	@if command -v docker &> /dev/null; then \
		echo "Starting development stack with Docker..."; \
		docker compose up -d postgres meilisearch redis; \
		echo "Waiting for services to be healthy..."; \
		timeout 60 bash -c 'until docker compose ps | grep -q "healthy.*\(postgres\|meilisearch\|redis\)"; do sleep 2; done'; \
		echo "Starting API and NLP services..."; \
		docker compose up -d api nlp; \
		echo "Development stack is running!"; \
		echo "PostgreSQL: localhost:15432"; \
		echo "Meilisearch: localhost:7700"; \
		echo "Redis: localhost:6379"; \
		echo "API: http://localhost:8080"; \
		echo "NLP Service: http://localhost:8081"; \
		echo "Run 'make logs' to see service logs"; \
	else \
		echo "Docker not found. Showing local development setup..."; \
		./scripts/dev-setup.sh; \
	fi

stop: ## Stop development stack
	@echo "Stopping development stack..."
	docker compose down
	@echo "All services stopped"

restart: ## Restart development stack
	@echo "Restarting development stack..."
	docker compose restart
	@echo "Services restarted"

# Testing Commands
test: ## Run all tests (Rust + Python)
	@echo "Running Rust tests..."
	cargo test
	@echo "Running Python tests..."
	cd nlp && uv run pytest
	@echo "All tests completed!"

test-rust: ## Run Rust tests only
	@echo "Running Rust tests..."
	cargo test

test-python: ## Run Python tests only
	@echo "Running Python tests..."
	cd nlp && uv run pytest

test-integration: ## Run integration tests
	@echo "Running integration tests..."
	docker compose -f docker-compose.test.yml up --build --abort-on-container-exit --exit-code-from test-runner

# Code Quality Commands
lint: ## Run all linting (Rust + Python)
	@echo "Running Rust linting..."
	cargo clippy
	cargo fmt --check
	@echo "Running Python linting..."
	cd nlp && uv run ruff check
	cd nlp && uv run black --check
	@echo "Linting completed!"

lint-rust: ## Run Rust linting only
	@echo "Running Rust linting..."
	cargo clippy
	cargo fmt --check

lint-python: ## Run Python linting only
	@echo "Running Python linting..."
	cd nlp && uv run ruff check
	cd nlp && uv run black --check

format: ## Format all code (Rust + Python)
	@echo "Formatting Rust code..."
	cargo fmt
	@echo "Formatting Python code..."
	cd nlp && uv run black .
	cd nlp && uv run ruff format .
	@echo "Code formatted!"

# Security Commands
audit: ## Run security audits (Rust + Python)
	@echo "Running Rust security audit..."
	cargo audit
	cargo deny check
	@echo "Running Python security audit..."
	cd nlp && uv run safety check
	@echo "Security audit completed!"

# Build Commands
build: ## Build all services
	@echo "Building Rust workspace..."
	cargo build --release
	@echo "Building Python package..."
	cd nlp && uv build
	@echo "Build completed!"

build-rust: ## Build Rust services only
	@echo "Building Rust workspace..."
	cargo build --release

build-python: ## Build Python package only
	@echo "Building Python package..."
	cd nlp && uv build

# Docker Commands
docker-build: ## Build Docker images
	@echo "Building Docker images..."
	docker compose build
	@echo "Docker images built!"

docker-stop: ## Stop and remove Docker containers
	@echo "Stopping and removing Docker containers..."
	docker compose down -v --remove-orphans
	@echo "Docker containers stopped and removed!"

docker-clean: ## Clean Docker resources
	@echo "Cleaning Docker resources..."
	docker compose down -v --remove-orphans
	docker system prune -f
	docker volume prune -f
	@echo "Docker resources cleaned!"

# Database Commands
db-migrate: ## Run database migrations
	@echo "Running database migrations..."
	docker compose exec api cargo run --bin migrate up
	@echo "Migrations completed!"

db-reset: ## Reset database (WARNING: This will delete all data)
	@echo "Resetting database..."
	docker compose exec postgres psql -U id_siber -d id_siber_index -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	@echo "Database reset!"

db-backup: ## Backup database
	@echo "Creating database backup..."
	docker compose exec postgres pg_dump -U id_siber id_siber_index > backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "Database backup created!"

db-restore: ## Restore database (usage: make db-restore FILE=backup.sql)
	@if [ -z "$(FILE)" ]; then echo "Usage: make db-restore FILE=backup.sql"; exit 1; fi
	@echo "Restoring database from $(FILE)..."
	docker compose exec -T postgres psql -U id_siber id_siber_index < $(FILE)
	@echo "Database restored!"

# Utility Commands
logs: ## Show service logs
	docker compose logs -f

logs-api: ## Show API service logs
	docker compose logs -f api

logs-db: ## Show database logs
	docker compose logs -f postgres

logs-search: ## Show Meilisearch logs
	docker compose logs -f meilisearch

shell-api: ## Open shell in API container
	docker compose exec api sh

shell-db: ## Open PostgreSQL shell
	docker compose exec postgres psql -U id_siber -d id_siber_index

shell-redis: ## Open Redis shell
	docker compose exec redis redis-cli

status: ## Show service status
	docker compose ps

health: ## Check service health
	@echo "Checking service health..."
	@docker compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"

clean: ## Clean all build artifacts
	@echo "Cleaning Rust artifacts..."
	cargo clean
	@echo "Cleaning Python artifacts..."
	cd nlp && rm -rf .venv/ target/ *.egg-info/ nltk_data/
	@echo "Cleaning Docker resources..."
	docker compose down -v --remove-orphans
	@echo "All artifacts cleaned!"

# Development Utilities
install-tools: ## Install development tools
	@echo "Installing development tools..."
	# Install pre-commit hooks
	pre-commit install
	# Install Rust tools
	rustup component add rustfmt clippy
	# Install Python tools
	cd nlp && uv sync --dev
	@echo "Development tools installed!"

setup: ## Complete development setup
	@echo "Setting up development environment..."
	make install-tools
	make docker-build
	cp .env.example .env
	@echo "Development environment setup complete!"
	@echo "Please edit .env file with your configuration"

# Production Commands
deploy-staging: ## Deploy to staging environment
	@echo "Deploying to staging..."
	docker compose -f docker-compose.yml -f docker-compose.staging.yml up -d --build
	@echo "Staging deployment complete!"

deploy-prod: ## Deploy to production environment
	@echo "Deploying to production..."
	docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
	@echo "Production deployment complete!"

# Monitoring Commands
metrics: ## Show service metrics
	@echo "API Metrics:"
	@curl -s http://localhost:8080/metrics || echo "API not responding"
	@echo ""
	@echo "Meilisearch Stats:"
	@curl -s http://localhost:7700/stats || echo "Meilisearch not responding"

monitor: ## Open monitoring dashboards
	@echo "Opening monitoring dashboards..."
	@echo "API: http://localhost:8080/health"
	@echo "Meilisearch: http://localhost:7700"
	@if command -v xdg-open >/dev/null 2>&1; then \
		xdg-open http://localhost:8080/health; \
	elif command -v open >/dev/null 2>&1; then \
		open http://localhost:8080/health; \
	fi
