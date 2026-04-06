"""
Text processing utilities for cybersecurity incident analysis.
"""

import os
import re

import nltk
from nltk.corpus import stopwords
from nltk.stem import PorterStemmer, WordNetLemmatizer
from nltk.tokenize import sent_tokenize, word_tokenize

# Download required NLTK data to project directory
NLTK_DATA_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "nltk_data")
os.makedirs(NLTK_DATA_PATH, exist_ok=True)
nltk.data.path.append(NLTK_DATA_PATH)

try:
    nltk.data.find("tokenizers/punkt")
except LookupError:
    nltk.download("punkt", download_dir=NLTK_DATA_PATH)

try:
    nltk.data.find("corpora/stopwords")
except LookupError:
    nltk.download("stopwords", download_dir=NLTK_DATA_PATH)

try:
    nltk.data.find("corpora/wordnet")
except LookupError:
    nltk.download("wordnet", download_dir=NLTK_DATA_PATH)


class TextProcessor:
    """Text preprocessing and cleaning utilities."""

    def __init__(self, language: str = "english"):
        self.language = language
        self.stemmer = PorterStemmer()
        self.lemmatizer = WordNetLemmatizer()
        self.stop_words = set(stopwords.words(language))

    def clean_text(self, text: str) -> str:
        """Clean and normalize text."""
        # Remove URLs
        text = re.sub(
            r"http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+",
            "",
            text,
        )

        # Remove email addresses
        text = re.sub(r"\S+@\S+", "", text)

        # Remove extra whitespace
        text = re.sub(r"\s+", " ", text).strip()

        # Remove special characters but keep spaces and basic punctuation
        text = re.sub(r"[^\w\s\.\,\!\?\;\:]", " ", text)

        return text

    def tokenize(self, text: str) -> list[str]:
        """Tokenize text into words."""
        return word_tokenize(text.lower())

    def remove_stopwords(self, tokens: list[str]) -> list[str]:
        """Remove stopwords from tokens."""
        return [token for token in tokens if token not in self.stop_words]

    def stem_tokens(self, tokens: list[str]) -> list[str]:
        """Apply stemming to tokens."""
        return [self.stemmer.stem(token) for token in tokens]

    def lemmatize_tokens(self, tokens: list[str]) -> list[str]:
        """Apply lemmatization to tokens."""
        return [self.lemmatizer.lemmatize(token) for token in tokens]

    def preprocess(
        self,
        text: str,
        remove_stopwords: bool = True,
        stem: bool = False,
        lemmatize: bool = True,
    ) -> list[str]:
        """
        Complete text preprocessing pipeline.

        Args:
            text: Input text to preprocess
            remove_stopwords: Whether to remove stopwords
            stem: Whether to apply stemming
            lemmatize: Whether to apply lemmatization

        Returns:
            List of processed tokens
        """
        # Clean text
        cleaned_text = self.clean_text(text)

        # Tokenize
        tokens = self.tokenize(cleaned_text)

        # Remove stopwords
        if remove_stopwords:
            tokens = self.remove_stopwords(tokens)

        # Apply stemming/lemmatization
        if stem:
            tokens = self.stem_tokens(tokens)
        elif lemmatize:
            tokens = self.lemmatize_tokens(tokens)

        # Remove empty tokens and single characters
        tokens = [token for token in tokens if len(token) > 1]

        return tokens

    def extract_sentences(self, text: str) -> list[str]:
        """Extract sentences from text."""
        return sent_tokenize(text)

    def normalize_case(self, text: str) -> str:
        """Normalize text case."""
        return text.lower().strip()
