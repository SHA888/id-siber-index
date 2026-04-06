"""
Language detection utilities for multilingual incident analysis.
"""


from iso639 import Lang
from langdetect import LangDetectException, detect


class LanguageDetector:
    """Detect and handle multiple languages in incident text."""

    def __init__(self):
        self.supported_languages = {
            "en": "English",
            "id": "Indonesian",
            "ms": "Malay",
            "zh": "Chinese",
            "ja": "Japanese",
            "ko": "Korean",
            "th": "Thai",
            "vi": "Vietnamese",
            "tl": "Filipino",
            "hi": "Hindi",
            "ar": "Arabic",
        }

    def detect_language(self, text: str) -> str | None:
        """
        Detect the primary language of the given text.

        Args:
            text: Input text to analyze

        Returns:
            ISO 639-1 language code or None if detection fails
        """
        try:
            return detect(text)
        except LangDetectException:
            return None

    def get_language_name(self, code: str) -> str:
        """
        Get full language name from ISO code.

        Args:
            code: ISO 639-1 language code

        Returns:
            Full language name
        """
        try:
            lang = Lang(code)
            return lang.name if lang.name else code
        except Exception:
            return code

    def is_supported(self, language_code: str) -> bool:
        """Check if a language is supported for processing."""
        return language_code in self.supported_languages

    def detect_with_confidence(self, text: str) -> tuple[str | None, float]:
        """
        Detect language with confidence score.

        Args:
            text: Input text to analyze

        Returns:
            Tuple of (language_code, confidence_score)
        """
        try:
            from langdetect import detect_langs

            langs = detect_langs(text)
            if langs:
                return langs[0].lang, langs[0].prob
            return None, 0.0
        except (LangDetectException, ImportError):
            return None, 0.0

    def analyze_multilingual_text(self, text: str) -> dict[str, any]:
        """
        Analyze text for multiple languages.

        Args:
            text: Input text to analyze

        Returns:
            Dictionary with language analysis results
        """
        primary_lang = self.detect_language(text)
        primary_lang_name = (
            self.get_language_name(primary_lang) if primary_lang else "Unknown"
        )

        lang_code, confidence = self.detect_with_confidence(text)
        lang_name = self.get_language_name(lang_code) if lang_code else "Unknown"

        return {
            "primary_language": primary_lang,
            "primary_language_name": primary_lang_name,
            "detected_language": lang_code,
            "detected_language_name": lang_name,
            "confidence": confidence,
            "is_supported": self.is_supported(lang_code) if lang_code else False,
            "supported_languages": list(self.supported_languages.keys()),
        }

    def preprocess_by_language(
        self, text: str, language_code: str | None = None
    ) -> str:
        """
        Apply language-specific preprocessing.

        Args:
            text: Input text to preprocess
            language_code: Target language code (auto-detect if None)

        Returns:
            Preprocessed text
        """
        if language_code is None:
            language_code = self.detect_language(text)

        if not language_code:
            return text

        # Language-specific preprocessing
        processed_text = text

        if language_code == "id":  # Indonesian
            # Handle Indonesian-specific preprocessing
            processed_text = self._preprocess_indonesian(processed_text)
        elif language_code == "zh":  # Chinese
            processed_text = self._preprocess_chinese(processed_text)
        elif language_code == "ja":  # Japanese
            processed_text = self._preprocess_japanese(processed_text)

        return processed_text

    def _preprocess_indonesian(self, text: str) -> str:
        """Indonesian-specific text preprocessing."""
        # Common Indonesian contractions and variations
        replacements = {
            "yg": "yang",
            "dgn": "dengan",
            "untk": "untuk",
            "sdh": "sudah",
            "blm": "belum",
            "krn": "karena",
            "jg": "juga",
            "utk": "untuk",
            "pd": "pada",
            "dr": "dari",
            "dlm": "dalam",
            "kpd": "kepada",
            "trh": "terhadap",
            "thd": "terhadap",
        }

        words = text.split()
        processed_words = []

        for word in words:
            lower_word = word.lower()
            if lower_word in replacements:
                processed_words.append(replacements[lower_word])
            else:
                processed_words.append(word)

        return " ".join(processed_words)

    def _preprocess_chinese(self, text: str) -> str:
        """Chinese-specific text preprocessing."""
        # Remove Chinese-specific punctuation and normalize
        import re

        # Chinese punctuation normalization
        text = re.sub(
            r'[，。！？；：""' "（）【】]",
            lambda m: {
                "，": ",",
                "。": ".",
                "！": "!",
                "？": "?",
                "；": ";",
                "：": ":",
                '"': '"',
                "'": "'",
                "(": "(",
                ")": ")",
            }.get(m.group(), m.group()),
            text,
        )

        return text

    def _preprocess_japanese(self, text: str) -> str:
        """Japanese-specific text preprocessing."""
        # Basic Japanese text normalization
        import re

        # Normalize Japanese punctuation
        text = re.sub(
            r"[、。！？]",
            lambda m: {"、": ",", "。": ".", "！": "!", "？": "?"}.get(
                m.group(), m.group()
            ),
            text,
        )

        return text
