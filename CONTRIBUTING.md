# Contributing to id-siber-index

Thank you for your interest in contributing to the Indonesia Cybersecurity Incident Index! This document outlines how you can help build this public infrastructure.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Contribution Types](#contribution-types)
- [Submission Guidelines](#submission-guidelines)
- [Review Process](#review-process)
- [License](#license)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please read and follow these guidelines in all interactions.

## How to Contribute

### 1. Fork and Clone

```bash
git clone https://github.com/your-username/id-siber-index.git
cd id-siber-index
```

### 2. Set Up Development Environment

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies
cd nlp && uv sync

# Start development services
docker-compose -f docker-compose.override.yml up -d
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 4. Make Changes and Test

```bash
# Run tests
make test

# Run linting
make lint

# Format code
make fmt
```

### 5. Submit a Pull Request

- Ensure all tests pass
- Update documentation as needed
- Follow the commit message format below
- Open a PR against the `main` branch

## Development Setup

### Prerequisites

- Rust 1.70+ (use `rust-toolchain.toml` for version pinning)
- Python 3.11+ (managed with `uv`)
- Docker & Docker Compose
- PostgreSQL 16
- Meilisearch

### Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit with your local configuration
# Set database URLs, search endpoints, etc.
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests
cd nlp && uv run pytest

# All tests together
make test
```

## Contribution Types

We welcome contributions in these categories:

### 📊 Data Contributions

**What:** New incident records, corrections to existing data, source verification

**How:**
1. Open an issue with subject: "Data: [Organization Name] - [Incident Type]"
2. Include:
   - Organization name and sector
   - Incident date and disclosure date
   - Attack type and data categories
   - Source URLs (must be publicly accessible)
   - Brief description
3. We'll verify and add to the index

**Requirements:**
- Sources must be publicly accessible
- No personal data or stolen information
- Must be verifiable Indonesian cybersecurity incidents

### 🤖 Crawler Improvements

**What:** New source integrations, better extraction logic, improved normalization

**How:**
1. Identify a new public source (IDX, BSSN, OJK, media sites)
2. Create a new crawler module in `crates/crawler/src/sources/`
3. Add tests for extraction logic
4. Update schema if needed
5. Submit PR

**Guidelines:**
- Follow existing crawler patterns
- Include comprehensive tests
- Handle edge cases gracefully
- Respect rate limits and robots.txt

### 📋 Schema Proposals

**What:** Field additions, enum extensions, STIX alignment

**How:**
1. Open an issue with "Schema: [Proposal Title]"
2. Describe the change and rationale
3. Provide example usage
4. Discuss with maintainers
5. Implement after approval

**Considerations:**
- Backward compatibility
- API impact
- Data migration needs
- STIX 2.1 alignment

### 🔬 Research Contributions

**What:** Analysis notebooks, trend reports, infrastructure correlation

**How:**
1. Create notebooks in `research/` directory
2. Use public data only
3. Include methodology and sources
4. Submit as PR or share in Discussions

**Requirements:**
- Reproducible analysis
- Clear documentation
- No private or sensitive data

## Submission Guidelines

### Commit Messages

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code refactoring
- `test`: Tests
- `chore`: Maintenance

Examples:
```
feat(crawler): add IDX disclosure source
fix(api): handle null incident dates
docs(readme): update installation instructions
```

### Pull Requests

PR titles should follow commit message format. Include:

- Clear description of changes
- Testing performed
- Documentation updates
- Breaking changes (if any)

### Code Style

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **Python**: Use `ruff format` and `ruff check`
- **Markdown**: Follow commonmark conventions
- **JSON**: Use 2-space indentation

## Review Process

### Automated Checks

All PRs must pass:
- ✅ Rust tests (`cargo test`)
- ✅ Python tests (`uv run pytest`)
- ✅ Linting (`cargo clippy`, `ruff check`)
- ✅ Formatting (`cargo fmt --check`, `ruff format --check`)
- ✅ Security audit (`cargo audit`)

### Manual Review

Maintainers will review for:
- Security implications
- Data quality and accuracy
- Performance impact
- Documentation completeness
- Breaking changes

### Merge Requirements

- At least one maintainer approval
- All automated checks pass
- Documentation updated
- No merge conflicts
- Ready for production (or clearly marked as experimental)

## Security

If you discover a security vulnerability:

1. Do not open a public issue
2. Email (coming soon)
3. Include details and reproduction steps
4. We'll respond within 48 hours
5. Coordinate disclosure timeline

See [SECURITY.md](SECURITY.md) for details.

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Community chat (link coming soon)

## License

By contributing, you agree that your contributions will be licensed under:

- **Code**: AGPL-3.0 (same as project)
- **Data**: CC BY 4.0 (attribution required)

See [LICENSE](LICENSE) for full terms.

## Recognition

Contributors are recognized in:
- GitHub contributors list
- Annual maintainer report
- Special thanks in releases

Thank you for helping build Indonesia's public cybersecurity infrastructure! 🇮🇩
