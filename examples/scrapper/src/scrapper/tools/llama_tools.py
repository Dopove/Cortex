# scrapper/tools/llama_tools.py

from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, Dict, Any
from datetime import datetime
import json
from pathlib import Path

# ------------------------------
# --- Pydantic Argument Models ---
# ------------------------------

class QualityAssessmentArgs(BaseModel):
    content: str = Field(..., description="The content to be assessed for quality.")
    # FIX: Use 'allow' to be compatible with CrewAI's internal context passing
    model_config = ConfigDict(extra='allow')

class RelevanceFilterArgs(BaseModel):
    content: str = Field(..., description="The content to be filtered for relevance.")
    url: str = Field(default="", description="The source URL of the content.")
    model_config = ConfigDict(extra='allow')

class MetadataExtractorArgs(BaseModel):
    content: str = Field(..., description="The content from which to extract metadata.")
    content_type: str = Field(default="text", description="The type of the content (e.g., 'text', 'image').")
    model_config = ConfigDict(extra='allow')

class CategorizationArgs(BaseModel):
    content: str = Field(..., description="The content to be categorized.")
    content_type: str = Field(default="textual", description="The primary type of the content.")
    model_config = ConfigDict(extra='allow')

# --------------------
# --- Custom Tools ---
# --------------------

class LlamaQualityAssessmentTool(BaseTool):
    name: str = "LlamaQualityAssessment"
    description: str = "Assesses the quality of fashion content for AI training suitability."
    args_schema: Type[BaseModel] = QualityAssessmentArgs

    def _run(self, content: str, **kwargs) -> Dict[str, Any]:
        # Placeholder for an actual LLM call.
        assessment_result = {
            "fashion_llm_score": 7.8,
            "overall_quality": 8.2,
            "key_concepts": ["fashion trends", "clothing styles", "design elements"],
            "training_suitability": "Excellent",
            "content_length": len(content),
            "assessed_at": datetime.now().isoformat()
        }
        self._save_assessment(assessment_result)
        return assessment_result

    def _save_assessment(self, assessment: Dict[str, Any]):
        data_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/metadata")
        data_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = data_dir / f"quality_assessment_{timestamp}.json"
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(assessment, f, indent=2)
        print(f"✅ Assessment saved to: {filename}")

class LlamaRelevanceFilterTool(BaseTool):
    name: str = "LlamaRelevanceFilter"
    description: str = "Filters content based on its relevance to the fashion domain."
    args_schema: Type[BaseModel] = RelevanceFilterArgs

    def _run(self, content: str, url: str = "", **kwargs) -> Dict[str, Any]:
        fashion_keywords = ["fashion", "clothing", "apparel", "style", "design", "fabric", "textile", "garment", "outfit", "trend"]
        content_lower = content.lower()
        found_keywords = [kw for kw in fashion_keywords if kw in content_lower]
        
        return {
            "is_relevant": len(found_keywords) > 0,
            "confidence": min(0.95, len(found_keywords) * 0.15),
            "keywords_found": found_keywords,
            "url": url,
            "processed_at": datetime.now().isoformat()
        }

class FashionMetadataExtractor(BaseTool):
    name: str = "FashionMetadataExtractor"
    description: str = "Extracts structured metadata from fashion-related content."
    args_schema: Type[BaseModel] = MetadataExtractorArgs

    def _run(self, content: str, content_type: str = "text", **kwargs) -> Dict[str, Any]:
        if content.strip().startswith('{'):
            try:
                content_data = json.loads(content)
            except json.JSONDecodeError:
                content_data = {"text": content}
        else:
            content_data = {"text": content}
        
        metadata = {
            "url": content_data.get("url", "N/A"),
            "title": content_data.get("title", "N/A"),
            "word_count": len(content_data.get("text", "").split()),
            "content_type": content_type,
            "has_images": bool(content_data.get("images")),
            "extracted_at": datetime.now().isoformat()
        }
        self._save_metadata(metadata)
        return metadata

    def _save_metadata(self, metadata: Dict[str, Any]):
        data_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/metadata")
        data_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = data_dir / f"metadata_{timestamp}.json"
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(metadata, f, indent=2)
        print(f"✅ Metadata saved to: {filename}")

class LlamaCategorizationTool(BaseTool):
    name: str = "LlamaCategorizationTool"
    description: str = "Categorizes fashion content into the correct folder structure."
    args_schema: Type[BaseModel] = CategorizationArgs

    def _run(self, content: str, content_type: str = "textual", **kwargs) -> Dict[str, Any]:
        content_lower = content.lower()
        
        if any(kw in content_lower for kw in ["trend", "forecast", "season"]):
            category, subcategory = "textual", "trend_reports"
        elif any(kw in content_lower for kw in ["theory", "principle", "concept"]):
            category, subcategory = "textual", "fashion_theory"
        elif any(kw in content_lower for kw in ["review", "rating", "opinion"]):
            category, subcategory = "textual", "consumer_feedback"
        else:
            category, subcategory = "textual", "fashion_theory"
        
        result = {
            "category": category,
            "subcategory": subcategory,
            "confidence": 0.85,
            "reasoning": f"Content classified as '{subcategory}' based on keyword matching.",
            "processed_at": datetime.now().isoformat()
        }
        self._save_categorization(result)
        return result

    def _save_categorization(self, result: Dict[str, Any]):
        data_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/metadata")
        data_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = data_dir / f"categorization_{timestamp}.json"
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(result, f, indent=2)
        print(f"✅ Categorization saved to: {filename}")

# ------------------------------------
# --- Tool Collection ---
# ------------------------------------

class FashionDataPipelineTools:
    """A class to encapsulate and provide all data processing tools."""
    def __init__(self):
        self.llama_quality_assessment = LlamaQualityAssessmentTool()
        self.llama_relevance_filter = LlamaRelevanceFilterTool()
        self.fashion_metadata_extractor = FashionMetadataExtractor()
        self.llama_categorization = LlamaCategorizationTool()

    def get_all_tools(self) -> list:
        """Returns a list of all tool instances for easy agent integration."""
        return [
            self.llama_quality_assessment,
            self.llama_relevance_filter,
            self.fashion_metadata_extractor,
            self.llama_categorization
        ]
