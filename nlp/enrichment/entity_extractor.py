"""
Entity extraction utilities for cybersecurity incident analysis.
"""

import re

import spacy
from spacy.matcher import Matcher


class EntityExtractor:
    """Extract entities from cybersecurity incident text."""

    def __init__(self, model_name: str = "en_core_web_sm"):
        """Initialize with spaCy model."""
        try:
            self.nlp = spacy.load(model_name)
        except OSError:
            print(f"spaCy model '{model_name}' not found. Using blank model.")
            self.nlp = spacy.blank("en")

        # Define patterns for cybersecurity entities
        self.matcher = Matcher(self.nlp.vocab)
        self._setup_patterns()

    def _setup_patterns(self):
        """Setup entity matching patterns."""
        # Organization patterns
        org_patterns = [
            [
                {
                    "LOWER": {
                        "REGEX": r"^(bank|pt|cv|yayasan|universitas|kementerian|lembaga)"
                    }
                }
            ],
            [{"LOWER": {"REGEX": r"^(company|corporation|ltd|inc|corp)"}}],
        ]
        self.matcher.add("ORGANIZATION", org_patterns)

        # Attack type patterns
        attack_patterns = [
            [
                {
                    "LOWER": {
                        "REGEX": r"^(phishing|malware|ransomware|ddos|sql.*injection)"
                    }
                }
            ],
            [
                {
                    "LOWER": {
                        "REGEX": r"^(cross.*site.*scripting|xss|man.*in.*the.*middle)"
                    }
                }
            ],
            [
                {
                    "LOWER": {
                        "REGEX": r"^(social.*engineering|brute.*force|denial.*of.*service)"
                    }
                }
            ],
        ]
        self.matcher.add("ATTACK_TYPE", attack_patterns)

        # Location patterns (Indonesia-specific)
        location_patterns = [
            [
                {
                    "LOWER": {
                        "REGEX": r"^(jakarta|surabaya|bandung|medan|semarang|makassar)"
                    }
                }
            ],
            [
                {
                    "LOWER": {
                        "REGEX": r"^(bali|yogyakarta|palembang|tangerang|depok|bekasi)"
                    }
                }
            ],
            [
                {
                    "LOWER": {
                        "REGEX": r"^(indonesia|jawa|sumatera|kalimantan|sulawesi|papua)"
                    }
                }
            ],
        ]
        self.matcher.add("LOCATION", location_patterns)

    def extract_entities(self, text: str) -> dict[str, list[str]]:
        """
        Extract entities from text.

        Args:
            text: Input text to analyze

        Returns:
            Dictionary with entity types as keys and lists of entities as values
        """
        doc = self.nlp(text)

        # Extract spaCy entities
        entities = {
            "PERSON": [],
            "ORG": [],
            "GPE": [],  # Geopolitical Entity
            "LOC": [],  # Location
            "PRODUCT": [],
            "EVENT": [],
        }

        for ent in doc.ents:
            if ent.label_ in entities:
                entities[ent.label_].append(ent.text)

        # Extract custom pattern matches
        matches = self.matcher(doc)
        custom_entities = {
            "ORGANIZATION": [],
            "ATTACK_TYPE": [],
            "LOCATION": [],
        }

        for match_id, start, end in matches:
            span = doc[start:end]
            entity_type = self.nlp.vocab.strings[match_id]
            if entity_type in custom_entities:
                custom_entities[entity_type].append(span.text)

        # Merge entities
        entities.update(custom_entities)

        # Remove duplicates and sort
        for key in entities:
            entities[key] = sorted(set(entities[key]))

        return entities

    def extract_indicators(self, text: str) -> dict[str, list[str]]:
        """
        Extract technical indicators from text.

        Args:
            text: Input text to analyze

        Returns:
            Dictionary with indicator types as keys
        """
        indicators = {
            "ip_addresses": [],
            "domains": [],
            "urls": [],
            "email_addresses": [],
            "file_hashes": [],
            "phone_numbers": [],
        }

        # IP addresses
        ip_pattern = r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b"
        indicators["ip_addresses"] = re.findall(ip_pattern, text)

        # Domains
        domain_pattern = r"\b[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b"
        indicators["domains"] = re.findall(domain_pattern, text)

        # URLs
        url_pattern = r'https?://[^\s<>"{}|\\^`\[\]]+'
        indicators["urls"] = re.findall(url_pattern, text)

        # Email addresses
        email_pattern = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
        indicators["email_addresses"] = re.findall(email_pattern, text)

        # File hashes (MD5, SHA1, SHA256)
        hash_patterns = [
            r"\b[a-fA-F0-9]{32}\b",  # MD5
            r"\b[a-fA-F0-9]{40}\b",  # SHA1
            r"\b[a-fA-F0-9]{64}\b",  # SHA256
        ]
        for pattern in hash_patterns:
            indicators["file_hashes"].extend(re.findall(pattern, text))

        # Phone numbers (Indonesia format)
        phone_pattern = r"\b(?:\+62|62|0)[0-9]{9,13}\b"
        indicators["phone_numbers"] = re.findall(phone_pattern, text)

        # Remove duplicates
        for key in indicators:
            indicators[key] = sorted(set(indicators[key]))

        return indicators

    def extract_sectors(self, text: str) -> list[str]:
        """Extract industry sectors mentioned in text."""
        sector_keywords = [
            "banking",
            "finance",
            "financial",
            "healthcare",
            "medical",
            "hospital",
            "government",
            "public",
            "education",
            "university",
            "school",
            "telecom",
            "telecommunication",
            "energy",
            "power",
            "electricity",
            "retail",
            "ecommerce",
            "manufacturing",
            "transportation",
            "logistics",
            "technology",
            "software",
            "insurance",
            "real estate",
            "construction",
            "agriculture",
            "mining",
        ]

        text_lower = text.lower()
        found_sectors = []

        for sector in sector_keywords:
            if sector in text_lower:
                found_sectors.append(sector)

        return sorted(set(found_sectors))
