from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, Dict, Any
from datetime import datetime
import json
from pathlib import Path

class MetadataExtractorArgs(BaseModel):
    content: Any = Field(..., description="Content to extract metadata from")
    content_type: str = Field(default='text', description='Type of content')
    model_config = ConfigDict(extra='forbid')

class FashionMetadataExtractor(BaseTool):
    name: str = 'FashionMetadataExtractor'
    description: str = 'Extract structured metadata from fashion content'
    args_schema: Type[BaseModel] = MetadataExtractorArgs

    def _run(self, content: Any = None, content_type: str = 'text') -> Dict[str, Any]:
        if isinstance(content, str):
            try:
                content = json.loads(content)
            except json.JSONDecodeError:
                return {"error": "Invalid content format; expected a JSON dictionary string."}

        if not isinstance(content, dict):
            return {"error": "Content must be a dictionary or JSON string"}

        metadata = {
            "url": content.get("url", "N/A"),
            "title": content.get("title", "N/A"), 
            "word_count": len(content.get("text", "").split()),
            "content_type": content_type,
            "extracted_at": datetime.now().isoformat()
        }

        # Save metadata to file
        try:
            data_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/metadata")
            data_dir.mkdir(parents=True, exist_ok=True)
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"metadata_{timestamp}.json"
            with open(data_dir / filename, "w", encoding="utf-8") as f:
                json.dump(metadata, f, indent=2)
            print(f"✅ Metadata saved to: {data_dir / filename}")
        except Exception as e:
            print(f"❌ Failed to save metadata: {e}")

        return metadata
