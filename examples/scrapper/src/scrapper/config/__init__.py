"""
Configuration for Fashion Data Scraper
"""
import os
from pathlib import Path

# Base paths - using your exact directory structure
BASE_DATA_DIR = Path(r"C:/Users/saran/Videos/Projects/FDM/dataCollector")

# Directory structure
COLLECTED_DATA_DIR = BASE_DATA_DIR / "collected_data"
DUPLICATE_TRACKING_DIR = BASE_DATA_DIR / "duplicate_tracking"
LOGS_DIR = BASE_DATA_DIR / "logs"
METADATA_DIR = BASE_DATA_DIR / "metadata"
PROCESSED_DATA_DIR = BASE_DATA_DIR / "processed_data"

# Ollama/LLaMA Configuration
OLLAMA_BASE_URL = "http://localhost:11434"
OLLAMA_MODEL = "llama3.1:8b"
LLAMA_MAX_TOKENS = 200
LLAMA_TEMPERATURE = 0.3

# Fashion Keywords for Search
FASHION_KEYWORDS = {
    "theory": [
        "fashion design principles", "color theory fashion", "garment construction",
        "fashion history", "textile properties", "draping techniques",
        "pattern making", "fashion sketching", "silhouette design"
    ],
    "trends": [
        "fashion trends 2025", "runway fashion", "street style trends",
        "color forecasting", "seasonal fashion", "fashion week reports",
        "consumer fashion preferences", "fashion market analysis"
    ],
    "visual": [
        "fashion sketches", "technical drawings fashion", "garment patterns",
        "textile designs", "fabric swatches", "fashion illustrations",
        "clothing photography", "runway photos", "fashion editorial"
    ],
    "academic": [
        "fashion research papers", "textile engineering", "fashion psychology",
        "sustainable fashion", "fashion marketing", "apparel manufacturing"
    ]
}

# Storage structure mapping - matches your folder structure exactly
STORAGE_STRUCTURE = {
    "structured": {
        "market_data": "collected_data/structured/market_data",
        "product_catalogs": "collected_data/structured/product_catalogs",
        "trend_metrics": "collected_data/structured/trend_metrics"
    },
    "textual": {
        "academic_papers": "collected_data/textual/academic_papers",
        "consumer_feedback": "collected_data/textual/consumer_feedback",
        "fashion_theory": "collected_data/textual/fashion_theory",
        "trend_reports": "collected_data/textual/trend_reports"
    },
    "visual": {
        "garment_images": "collected_data/visual/garment_images",
        "patterns": "collected_data/visual/patterns",
        "sketches": "collected_data/visual/sketches",
        "technical_drawings": "collected_data/visual/technical_drawings"
    }
}

# Quality Score Thresholds
MIN_QUALITY_SCORES = {
    "fashion_llm": 7,
    "classifier": 8,
    "sketch_gen": 6,
    "pattern_gen": 7
}

def create_directories():
    """Create directory structure matching your existing folders"""
    # Base directories
    directories = [
        COLLECTED_DATA_DIR,
        DUPLICATE_TRACKING_DIR, 
        LOGS_DIR,
        METADATA_DIR,
        PROCESSED_DATA_DIR
    ]
    
    # Add all subdirectories from STORAGE_STRUCTURE
    for category, subcategories in STORAGE_STRUCTURE.items():
        for subcategory, relative_path in subcategories.items():
            full_path = BASE_DATA_DIR / relative_path
            directories.append(full_path)
    
    # Create all directories
    for directory in directories:
        directory.mkdir(parents=True, exist_ok=True)
    
    print(f"[SUCCESS] Created directory structure at {BASE_DATA_DIR}")
    print(f"   - structured: market_data, product_catalogs, trend_metrics")
    print(f"   - textual: academic_papers, consumer_feedback, fashion_theory, trend_reports")
    print(f"   - visual: garment_images, patterns, sketches, technical_drawings")

# Export all configuration
__all__ = [
    "create_directories", "OLLAMA_BASE_URL", "OLLAMA_MODEL", 
    "LLAMA_MAX_TOKENS", "LLAMA_TEMPERATURE", "FASHION_KEYWORDS",
    "STORAGE_STRUCTURE", "BASE_DATA_DIR", "COLLECTED_DATA_DIR",
    "DUPLICATE_TRACKING_DIR", "LOGS_DIR", "METADATA_DIR", "PROCESSED_DATA_DIR"
]
