# ID Siber Index NLP Tools

Natural Language Processing enrichment tools for the Indonesia Cybersecurity Incident Index.

## Features

- **Text Processing**: Cleaning, tokenization, stemming, and lemmatization
- **Entity Extraction**: Organizations, locations, attack types, and technical indicators
- **Language Detection**: Multi-language support with Indonesian focus
- **Incident Classification**: Machine learning-based attack type classification
- **Preprocessing**: Language-specific text preprocessing

## Installation

This package uses `uv` for dependency management. Install with:

```bash
cd nlp
uv sync
```

## Usage

### Text Processing

```python
from enrichment.text_processor import TextProcessor

processor = TextProcessor(language='english')
tokens = processor.preprocess(
    "Phishing attack detected at Indonesian bank",
    remove_stopwords=True,
    lemmatize=True
)
print(tokens)  # ['phishing', 'attack', 'detect', 'indonesian', 'bank']
```

### Entity Extraction

```python
from enrichment.entity_extractor import EntityExtractor

extractor = EntityExtractor()
entities = extractor.extract_entities(
    "Phishing attack at Bank Central Asia in Jakarta targeted customers"
)
print(entities)
# {'ORGANIZATION': ['Bank Central Asia'], 'LOCATION': ['Jakarta'], ...}
```

### Language Detection

```python
from enrichment.language_detector import LanguageDetector

detector = LanguageDetector()
result = detector.analyze_multilingual_text("Serangan phishing terdeteksi di bank Indonesia")
print(result)
# {'primary_language': 'id', 'primary_language_name': 'Indonesian', ...}
```

### Incident Classification

```python
from enrichment.classifier import IncidentClassifier

classifier = IncidentClassifier()
# Train with labeled data (you need to provide training data)
metrics = classifier.train_attack_type_classifier(texts, labels)

# Predict new incidents
attack_type, confidence = classifier.predict_attack_type(
    "Suspicious email requesting login credentials"
)
print(f"Attack: {attack_type} (confidence: {confidence:.2f})")
```

## Development

### Setup Development Environment

```bash
cd nlp
uv sync --dev
```

### Code Quality

```bash
# Format code
uv run black .

# Lint code
uv run ruff check .

# Type checking
uv run mypy src/

# Run tests
uv run pytest
```

### Pre-commit Hooks

Pre-commit hooks are configured to run automatically:

- Black formatting
- Ruff linting
- MyPy type checking
- pytest tests

## Model Requirements

For full functionality, you'll need to install spaCy models:

```bash
uv run python -m spacy download en_core_web_sm
```

## Architecture

```
nlp/
├── enrichment/
│   ├── __init__.py          # Package initialization
│   ├── text_processor.py    # Text preprocessing utilities
│   ├── entity_extractor.py  # Named entity recognition
│   ├── language_detector.py  # Multi-language support
│   └── classifier.py        # ML-based classification
├── src/id_siber_nlp/        # Package source
├── tests/                    # Test suite
├── pyproject.toml           # Project configuration
└── .python-version          # Python version pin
```

## Language Support

Primary focus on Indonesian and English, with additional support for:
- Malay (ms)
- Chinese (zh) 
- Japanese (ja)
- Korean (ko)
- Thai (th)
- Vietnamese (vi)
- Filipino (tl)

## Contributing

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Ensure all pre-commit hooks pass

## License

AGPL-3.0 - See LICENSE file for details.
