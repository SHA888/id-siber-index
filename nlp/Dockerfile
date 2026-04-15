# Multi-stage build for Python NLP Service
FROM python:3.11-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:$PATH"

# Set working directory
WORKDIR /app

# Copy Python files
COPY pyproject.toml ./
COPY src/ ./src/
COPY enrichment/ ./enrichment/

# Install dependencies
RUN uv sync --frozen --no-dev

# Download NLTK data
RUN uv run python -c "import os; import nltk; nltk_data_path='/app/nltk_data'; os.makedirs(nltk_data_path, exist_ok=True); nltk.data.path.append(nltk_data_path); nltk.download('punkt', download_dir=nltk_data_path); nltk.download('stopwords', download_dir=nltk_data_path); nltk.download('wordnet', download_dir=nltk_data_path)"

# Runtime stage
FROM python:3.11-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:$PATH"

# Create non-root user
RUN groupadd -r nlp && useradd -r -g nlp nlp

# Set working directory
WORKDIR /app

# Copy Python files and dependencies
COPY --from=builder /app/pyproject.toml ./
COPY --from=builder /app/src/ ./src/
COPY --from=builder /app/enrichment/ ./enrichment/
COPY --from=builder /app/.venv/ ./.venv/

# Copy NLTK data
COPY --from=builder /root/nltk_data ./nltk_data

# Create directories
RUN mkdir -p /app/logs /app/uploads /app/transformers_cache && \
    chown -R nlp:nlp /app

# Switch to non-root user
USER nlp

# Set environment variables
ENV PYTHONPATH=/app
ENV NLTK_DATA=/app/nltk_data
ENV TRANSFORMERS_CACHE=/app/transformers_cache

# Expose port
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Default command (will be overridden in docker-compose)
CMD ["uv", "run", "python", "-m", "enrichment"]
