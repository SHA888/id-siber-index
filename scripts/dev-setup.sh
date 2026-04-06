#!/bin/bash

# Development setup script for environments without Docker
# This script sets up local development services

set -e

echo "🚀 Setting up Indonesia Cybersecurity Incident Index development environment..."

# Check if required tools are available
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install $1 first."
        return 1
    else
        echo "✅ $1 is available"
        return 0
    fi
}

echo "🔍 Checking required tools..."

# Check for PostgreSQL
if ! check_tool psql; then
    echo "💡 Install PostgreSQL: sudo apt-get install postgresql postgresql-contrib"
    echo "💡 Or use Docker: docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres:16"
fi

# Check for Redis
if ! check_tool redis-server; then
    echo "💡 Install Redis: sudo apt-get install redis-server"
    echo "💡 Or use Docker: docker run --name redis -p 6379:6379 -d redis:7"
fi

# Check for curl
check_tool curl || echo "💡 Install curl: sudo apt-get install curl"

echo ""
echo "📝 Environment Setup:"
echo "1. Copy .env.example to .env and configure your settings"
echo "2. Start PostgreSQL: sudo systemctl start postgresql"
echo "3. Start Redis: sudo systemctl start redis-server"
echo "4. Create database: createdb id_siber_index"
echo "5. Run migrations: cargo run --bin migrate up"
echo "6. Start API: cargo run --bin api"
echo "7. Start NLP service (in another terminal): cd nlp && uv run python -m enrichment"

echo ""
echo "🔗 Service URLs (when running):"
echo "- API: http://localhost:8080"
echo "- PostgreSQL: localhost:5432"
echo "- Redis: localhost:6379"
echo "- Meilisearch: http://localhost:7700 (if using Docker)"

echo ""
echo "🛠️ Alternative: Use Docker Desktop with WSL 2 integration"
echo "Visit: https://docs.docker.com/go/wsl2/"

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo ""
    echo "📄 Creating .env file from template..."
    cp .env.example .env
    echo "✅ .env file created. Please edit it with your configuration."
fi

echo ""
echo "✅ Development setup guide completed!"
