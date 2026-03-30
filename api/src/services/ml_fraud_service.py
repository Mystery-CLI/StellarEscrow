"""
ML Fraud Detection Service - Scikit-learn based
Serves fraud prediction models via REST API
"""

from flask import Flask, request, jsonify
from sklearn.ensemble import RandomForestClassifier, GradientBoostingClassifier
from sklearn.preprocessing import StandardScaler
from sklearn.model_selection import train_test_split
from sklearn.metrics import confusion_matrix, classification_report
import numpy as np
import joblib
import json
import os
from datetime import datetime
from typing import Dict, List, Tuple

# ============================================================================
# CONFIGURATION
# ============================================================================

app = Flask(__name__)
MODEL_DIR = os.getenv('MODEL_DIR', '/models')
MODEL_VERSION = os.getenv('MODEL_VERSION', 'v1.0.0')
MIN_CONFIDENCE = float(os.getenv('MIN_CONFIDENCE', '0.5'))

# Feature names (must match Node.js service)
FEATURE_NAMES = [
    'purchase_amount',
    'user_age_days',
    'previous_trades',
    'disputes_ratio',
    'time_since_last_trade_hours',
    'trades_last_24h',
    'trades_last_hour',
    'avg_transaction_value',
    'location_changes_24h',
    'device_fingerprint_change',  # 1 or 0
]

# Global models (loaded on startup)
rf_model = None
gb_model = None
scaler = None
model_metadata = None


# ============================================================================
# MODEL MANAGEMENT
# ============================================================================


def load_models():
    """Load trained models from disk"""
    global rf_model, gb_model, scaler, model_metadata

    try:
        rf_model = joblib.load(f'{MODEL_DIR}/random_forest_{MODEL_VERSION}.pkl')
        gb_model = joblib.load(f'{MODEL_DIR}/gradient_boosting_{MODEL_VERSION}.pkl')
        scaler = joblib.load(f'{MODEL_DIR}/scaler_{MODEL_VERSION}.pkl')

        with open(f'{MODEL_DIR}/metadata_{MODEL_VERSION}.json', 'r') as f:
            model_metadata = json.load(f)

        print(f'✓ Models loaded: {MODEL_VERSION}')
        print(f'  - Random Forest: {model_metadata.get("rf_accuracy", "N/A")}')
        print(f'  - Gradient Boosting: {model_metadata.get("gb_accuracy", "N/A")}')

    except FileNotFoundError as e:
        print(f'✗ Model files not found: {e}')
        print('  Training dummy models for demonstration...')
        train_dummy_models()


def train_dummy_models():
    """Train dummy models for demonstration (replace with real data)"""
    global rf_model, gb_model, scaler, model_metadata

    # Generate synthetic training data
    np.random.seed(42)
    n_samples = 10000

    X = np.random.randn(n_samples, len(FEATURE_NAMES))
    # Labels: fraud (1) if suspicious patterns in features
    y = (
        (X[:, 0] > 1.5)  # large amount
        | (X[:, 1] < -1.0)  # new account
        | (X[:, 5] > 2.0)  # many trades in 24h
    ).astype(int)

    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42, stratify=y
    )

    # Scale features
    scaler = StandardScaler()
    X_train_scaled = scaler.fit_transform(X_train)
    X_test_scaled = scaler.transform(X_test)

    # Train Random Forest
    rf_model = RandomForestClassifier(
        n_estimators=100,
        max_depth=10,
        min_samples_split=5,
        random_state=42,
        n_jobs=-1,
    )
    rf_model.fit(X_train_scaled, y_train)
    rf_accuracy = rf_model.score(X_test_scaled, y_test)

    # Train Gradient Boosting
    gb_model = GradientBoostingClassifier(
        n_estimators=100,
        learning_rate=0.1,
        max_depth=5,
        random_state=42,
    )
    gb_model.fit(X_train_scaled, y_train)
    gb_accuracy = gb_model.score(X_test_scaled, y_test)

    # Classification report
    y_pred = rf_model.predict(X_test_scaled)
    report = classification_report(y_test, y_pred, output_dict=True)

    model_metadata = {
        'version': MODEL_VERSION,
        'trained_at': datetime.now().isoformat(),
        'training_samples': n_samples,
        'test_samples': len(X_test),
        'rf_accuracy': float(rf_accuracy),
        'gb_accuracy': float(gb_accuracy),
        'precision': float(report['1']['precision']),
        'recall': float(report['1']['recall']),
        'f1_score': float(report['1']['f1-score']),
        'feature_names': FEATURE_NAMES,
    }

    # Save to disk
    os.makedirs(MODEL_DIR, exist_ok=True)
    joblib.dump(rf_model, f'{MODEL_DIR}/random_forest_{MODEL_VERSION}.pkl')
    joblib.dump(gb_model, f'{MODEL_DIR}/gradient_boosting_{MODEL_VERSION}.pkl')
    joblib.dump(scaler, f'{MODEL_DIR}/scaler_{MODEL_VERSION}.pkl')

    with open(f'{MODEL_DIR}/metadata_{MODEL_VERSION}.json', 'w') as f:
        json.dump(model_metadata, f, indent=2)

    print(f'✓ Dummy models trained and saved')
    print(f'  - RF Accuracy: {rf_accuracy:.4f}')
    print(f'  - GB Accuracy: {gb_accuracy:.4f}')


# ============================================================================
# PREDICTION & INFERENCE
# ============================================================================


def extract_features(data: Dict) -> np.ndarray:
    """Extract features from request data"""
    features = []
    for name in FEATURE_NAMES:
        if name not in data:
            raise ValueError(f'Missing feature: {name}')
        features.append(float(data[name]))

    return np.array([features])


def predict_fraud(X: np.ndarray) -> Tuple[float, float, float]:
    """
    Predict fraud probability using ensemble methods
    Returns: (ensemble_probability, rf_prob, gb_prob)
    """
    global rf_model, gb_model, scaler

    if rf_model is None or gb_model is None:
        raise RuntimeError('Models not loaded')

    # Scale features
    X_scaled = scaler.transform(X)

    # Get predictions from both models
    rf_proba = rf_model.predict_proba(X_scaled)[0][1]  # Probability of fraud
    gb_proba = gb_model.predict_proba(X_scaled)[0][1]

    # Ensemble: weighted average
    ensemble_proba = 0.6 * rf_proba + 0.4 * gb_proba

    return ensemble_proba, rf_proba, gb_proba


# ============================================================================
# API ENDPOINTS
# ============================================================================


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        'status': 'ok',
        'version': MODEL_VERSION,
        'models_loaded': rf_model is not None and gb_model is not None,
    })


@app.route('/predict', methods=['POST'])
def predict():
    """
    Predict fraud probability
    
    Request body:
    {
        "features": {
            "purchase_amount": 15000.50,
            "user_age_days": 3,
            "previous_trades": 0,
            ...
        }
    }
    
    Response:
    {
        "fraud_probability": 0.85,
        "risk_level": "critical",
        "model_version": "v1.0.0",
        "confidence": 0.95
    }
    """
    try:
        data = request.get_json()
        features_dict = data.get('features', {})

        # Validate required fields
        if not features_dict:
            return jsonify({'error': 'Missing features'}), 400

        # Extract and validate features
        X = extract_features(features_dict)

        # Get ensemble prediction
        fraud_prob, rf_prob, gb_prob = predict_fraud(X)

        # Determine risk level
        if fraud_prob >= 0.85:
            risk_level = 'critical'
        elif fraud_prob >= 0.65:
            risk_level = 'high'
        elif fraud_prob >= 0.45:
            risk_level = 'medium'
        else:
            risk_level = 'low'

        # Calculate confidence
        confidence = abs(rf_prob - gb_prob)

        return jsonify({
            'fraud_probability': float(fraud_prob),
            'risk_level': risk_level,
            'model_version': MODEL_VERSION,
            'confidence': float(confidence),
            'rf_probability': float(rf_prob),
            'gb_probability': float(gb_prob),
            'features_received': len(features_dict),
        })

    except Exception as e:
        return jsonify({'error': str(e)}), 400


@app.route('/batch-predict', methods=['POST'])
def batch_predict():
    """
    Predict fraud for multiple transactions (bulk analysis)
    
    Request body:
    {
        "transactions": [
            {"purchase_amount": 5000, "user_age_days": 10, ...},
            {"purchase_amount": 50000, "user_age_days": 2, ...}
        ]
    }
    """
    try:
        data = request.get_json()
        transactions = data.get('transactions', [])

        if not type(transactions) == list:
            return jsonify({'error': 'transactions must be a list'}), 400

        predictions = []
        for tx in transactions:
            try:
                X = extract_features(tx)
                fraud_prob, rf_prob, gb_prob = predict_fraud(X)

                predictions.append({
                    'fraud_probability': float(fraud_prob),
                    'rf_probability': float(rf_prob),
                    'gb_probability': float(gb_prob),
                })

            except Exception as e:
                predictions.append({
                    'error': str(e),
                    'fraud_probability': None,
                })

        return jsonify({
            'model_version': MODEL_VERSION,
            'predictions_count': len(predictions),
            'predictions': predictions,
        })

    except Exception as e:
        return jsonify({'error': str(e)}), 400


@app.route('/model-info', methods=['GET'])
def model_info():
    """Get information about loaded models"""
    return jsonify({
        'version': MODEL_VERSION,
        'metadata': model_metadata,
        'feature_names': FEATURE_NAMES,
        'feature_count': len(FEATURE_NAMES),
    })


@app.route('/feature-importance', methods=['GET'])
def feature_importance():
    """Get feature importance from Random Forest model"""
    global rf_model

    if rf_model is None:
        return jsonify({'error': 'Model not loaded'}), 503

    importances = rf_model.feature_importances_
    feature_importance_list = [
        {
            'feature': name,
            'importance': float(importance),
            'importance_percent': float(importance * 100),
        }
        for name, importance in zip(FEATURE_NAMES, importances)
    ]

    # Sort by importance
    feature_importance_list.sort(key=lambda x: x['importance'], reverse=True)

    return jsonify({
        'model_version': MODEL_VERSION,
        'features': feature_importance_list,
    })


# ============================================================================
# STARTUP
# ============================================================================


@app.before_request
def before_request():
    """Ensure models are loaded"""
    global rf_model

    if rf_model is None:
        load_models()


if __name__ == '__main__':
    load_models()
    app.run(
        host='0.0.0.0',
        port=int(os.getenv('PORT', 8000)),
        debug=os.getenv('FLASK_ENV', 'production') == 'development',
    )
