# scrapper/tools/search_tools.py

from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, Dict, Any, List, Optional
from ddgs import DDGS
from scrapper.config import FASHION_KEYWORDS

class URLDiscoveryArgs(BaseModel):
    """Arguments for the FashionURLDiscoveryTool."""
    query: str = Field(..., description="Search query for fashion URLs")
    max_results: int = Field(default=10, description="Maximum results to return")
    model_config = ConfigDict(extra="allow")

class FashionURLDiscoveryTool(BaseTool):
    # FIX: Added ': str' type annotation to override BaseTool fields correctly
    name: str = "FashionURLDiscovery"
    description: str = "Discover fashion-related URLs based on a query or predefined keywords."
    args_schema: Type[BaseModel] = URLDiscoveryArgs

    def _run(self, query: str, max_results: int = 10, **kwargs) -> Dict[str, Any]:
        search_terms = [query] if query else (kwargs.get("keywords", []))
        
        all_urls = []
        try:
            with DDGS() as ddgs:
                for term in search_terms[:5]:
                    if not term: continue
                    results = ddgs.text(term, max_results=max_results)
                    for result in results:
                        all_urls.append({
                            "title": result.get("title", ""),
                            "url": result.get("href", ""),
                            "snippet": result.get("body", ""),
                            "source": "duckduckgo",
                            "search_keyword": term
                        })
            return {"success": True, "urls": all_urls}
        except Exception as e:
            return {"success": False, "error": str(e)}

class URLValidatorArgs(BaseModel):
    """Arguments for the URLValidatorTool."""
    urls: List[Dict[str, Any]] = Field(..., description="A list of URL data dictionaries to validate.")
    model_config = ConfigDict(extra="allow")

class URLValidatorTool(BaseTool):
    # FIX: Added ': str' type annotation
    name: str = "URLValidator"
    description: str = "Validate a list of URLs for their relevance to fashion."
    args_schema: Type[BaseModel] = URLValidatorArgs

    def _run(self, urls: List[Dict[str, Any]], **kwargs) -> Dict[str, Any]:
        valid_urls, invalid_urls = [], []
        fashion_keywords = {kw for cat in FASHION_KEYWORDS.values() for kw in cat}

        for url_data in urls:
            try:
                url = url_data.get("url", "")
                title = url_data.get("title", "").lower()
                snippet = url_data.get("snippet", "").lower()
                
                if not url or not url.startswith(("http://", "https://")):
                    invalid_urls.append({"url": url, "reason": "Invalid URL format"})
                    continue
                
                if any(keyword in f"{title} {snippet}" for keyword in fashion_keywords):
                    valid_urls.append(url_data)
                else:
                    invalid_urls.append({"url": url, "reason": "Low relevance to fashion."})
            except Exception as e:
                invalid_urls.append({"url": url_data.get("url", ""), "reason": f"Validation error: {e}"})
        
        return {"success": True, "valid_urls": valid_urls, "invalid_urls": invalid_urls}


def validate_fashion_url(url, title, snippet):
    fashion_keywords = [
        "fashion", "style", "clothing", "outfit", "trend", "runway", 
        "designer", "apparel", "garment", "dress", "shoes", "accessories"
    ]
    
    content = f"{title} {snippet}".lower()
    keyword_matches = sum(1 for keyword in fashion_keywords if keyword in content)
    
    # More lenient validation
    is_relevant = keyword_matches >= 1
    confidence = min(keyword_matches / len(fashion_keywords), 1.0)
    
    return {
        "is_relevant": is_relevant,
        "confidence": confidence,
        "keywords_found": [kw for kw in fashion_keywords if kw in content]
    }