"""
Text classification utilities for cybersecurity incident analysis.
"""

import pickle
from pathlib import Path

import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import classification_report, confusion_matrix
from sklearn.model_selection import train_test_split
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import LabelEncoder


class IncidentClassifier:
    """Machine learning classifier for cybersecurity incidents."""

    def __init__(self, model_path: Path | None = None):
        """
        Initialize the classifier.

        Args:
            model_path: Path to saved model file
        """
        self.model_path = model_path
        self.pipeline = None
        self.label_encoder = LabelEncoder()
        self.is_trained = False

        # Attack type categories
        self.attack_categories = [
            "phishing",
            "malware",
            "ransomware",
            "ddos",
            "sql_injection",
            "xss",
            "social_engineering",
            "brute_force",
            "data_breach",
            "insider_threat",
            "unknown",
        ]

        # Severity levels
        self.severity_levels = ["low", "medium", "high", "critical"]

        if model_path and model_path.exists():
            self.load_model()

    def _create_pipeline(self) -> Pipeline:
        """Create the ML pipeline."""
        pipeline = Pipeline(
            [
                (
                    "tfidf",
                    TfidfVectorizer(
                        max_features=5000,
                        ngram_range=(1, 2),
                        stop_words="english",
                        lowercase=True,
                        min_df=2,
                        max_df=0.95,
                    ),
                ),
                (
                    "classifier",
                    LogisticRegression(
                        random_state=42, max_iter=1000, class_weight="balanced"
                    ),
                ),
            ]
        )
        return pipeline

    def train_attack_type_classifier(
        self, texts: list[str], labels: list[str], test_size: float = 0.2
    ) -> dict[str, any]:
        """
        Train attack type classifier.

        Args:
            texts: Training texts
            labels: Attack type labels
            test_size: Test set proportion

        Returns:
            Training metrics
        """
        # Filter valid labels
        valid_indices = [
            i for i, label in enumerate(labels) if label in self.attack_categories
        ]

        if len(valid_indices) == 0:
            raise ValueError("No valid training labels found")

        filtered_texts = [texts[i] for i in valid_indices]
        filtered_labels = [labels[i] for i in valid_indices]

        # Split data
        X_train, X_test, y_train, y_test = train_test_split(
            filtered_texts,
            filtered_labels,
            test_size=test_size,
            random_state=42,
            stratify=filtered_labels,
        )

        # Create and train pipeline
        self.pipeline = self._create_pipeline()
        self.pipeline.fit(X_train, y_train)

        # Evaluate
        train_score = self.pipeline.score(X_train, y_train)
        test_score = self.pipeline.score(X_test, y_test)
        y_pred = self.pipeline.predict(X_test)

        # Generate classification report
        report = classification_report(y_test, y_pred, output_dict=True)

        self.is_trained = True

        return {
            "train_accuracy": train_score,
            "test_accuracy": test_score,
            "classification_report": report,
            "confusion_matrix": confusion_matrix(y_test, y_pred).tolist(),
            "feature_count": len(self.pipeline.named_steps["tfidf"].vocabulary_),
            "training_samples": len(X_train),
            "test_samples": len(X_test),
        }

    def predict_attack_type(self, text: str) -> tuple[str, float]:
        """
        Predict attack type for given text.

        Args:
            text: Input text to classify

        Returns:
            Tuple of (predicted_attack_type, confidence_score)
        """
        if not self.is_trained:
            return "unknown", 0.0

        # Get prediction and probabilities
        prediction = self.pipeline.predict([text])[0]
        probabilities = self.pipeline.predict_proba([text])[0]

        # Find confidence score
        class_names = self.pipeline.named_steps["classifier"].classes_
        pred_index = np.where(class_names == prediction)[0][0]
        confidence = probabilities[pred_index]

        return prediction, float(confidence)

    def predict_attack_type_batch(self, texts: list[str]) -> list[tuple[str, float]]:
        """
        Predict attack types for multiple texts.

        Args:
            texts: List of input texts to classify

        Returns:
            List of (predicted_attack_type, confidence_score) tuples
        """
        if not self.is_trained:
            return [("unknown", 0.0) for _ in texts]

        predictions = self.pipeline.predict(texts)
        probabilities = self.pipeline.predict_proba(texts)
        class_names = self.pipeline.named_steps["classifier"].classes_

        results = []
        for i, pred in enumerate(predictions):
            pred_index = np.where(class_names == pred)[0][0]
            confidence = probabilities[i][pred_index]
            results.append((pred, float(confidence)))

        return results

    def get_feature_importance(
        self, top_n: int = 20
    ) -> dict[str, list[tuple[str, float]]]:
        """
        Get most important features for each class.

        Args:
            top_n: Number of top features to return per class

        Returns:
            Dictionary mapping class names to list of (feature, importance) tuples
        """
        if not self.is_trained:
            return {}

        feature_names = self.pipeline.named_steps["tfidf"].get_feature_names_out()
        classifier = self.pipeline.named_steps["classifier"]

        importance_dict = {}

        for i, class_name in enumerate(classifier.classes_):
            # Get coefficients for this class
            if hasattr(classifier, "coef_"):
                if len(classifier.coef_) == 1:  # Binary classification
                    coef = classifier.coef_[0]
                else:  # Multi-class
                    coef = classifier.coef_[i]

                # Get top features
                top_indices = np.argsort(coef)[-top_n:][::-1]
                top_features = [
                    (feature_names[idx], float(coef[idx])) for idx in top_indices
                ]

                importance_dict[class_name] = top_features

        return importance_dict

    def save_model(self, path: Path):
        """
        Save the trained model.

        Args:
            path: Path to save the model
        """
        if not self.is_trained:
            raise ValueError("Model must be trained before saving")

        model_data = {
            "pipeline": self.pipeline,
            "attack_categories": self.attack_categories,
            "severity_levels": self.severity_levels,
            "is_trained": self.is_trained,
        }

        with open(path, "wb") as f:
            pickle.dump(model_data, f)

        self.model_path = path

    def load_model(self, path: Path | None = None):
        """
        Load a saved model.

        Args:
            path: Path to the saved model (uses self.model_path if None)
        """
        load_path = path or self.model_path
        if not load_path or not load_path.exists():
            raise FileNotFoundError(f"Model file not found: {load_path}")

        with open(load_path, "rb") as f:
            model_data = pickle.load(f)

        self.pipeline = model_data["pipeline"]
        self.attack_categories = model_data["attack_categories"]
        self.severity_levels = model_data["severity_levels"]
        self.is_trained = model_data["is_trained"]

        if path:
            self.model_path = path

    def get_training_stats(self) -> dict[str, any]:
        """Get training statistics."""
        if not self.is_trained:
            return {"is_trained": False}

        return {
            "is_trained": True,
            "model_path": str(self.model_path) if self.model_path else None,
            "attack_categories": self.attack_categories,
            "severity_levels": self.severity_levels,
            "feature_count": len(self.pipeline.named_steps["tfidf"].vocabulary_),
            "model_type": type(self.pipeline.named_steps["classifier"]).__name__,
        }
