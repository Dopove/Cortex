# scrapper/tools/scrape_tools.py

from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, List, Dict, Any
from datetime import datetime
from pathlib import Path
import requests
from bs4 import BeautifulSoup
# FIX: Import the json module to handle string-to-list conversion
import json

# ------------------------------
# --- Pydantic Argument Models ---
# ------------------------------

class BatchScrapingArgs(BaseModel):
    """Arguments for the BatchScrapingTool."""
    urls: List[str] = Field(..., description="A list of URLs to be scraped for content.")
    max_content_length: int = Field(default=10000, description="The maximum character length for the scraped content from each URL.")
    
    # This configuration is correct and allows the tool to work with CrewAI.
    model_config = ConfigDict(extra="allow")

# --------------------
# --- Custom Tools ---
# --------------------

class BatchScrapingTool(BaseTool):
    """A tool to scrape the textual content from a list of fashion websites."""
    name: str = "BatchScraping"
    description: str = "Scrapes the primary text content from a list of multiple fashion websites."
    args_schema: Type[BaseModel] = BatchScrapingArgs

    def _run(self, urls: List[str] | str, max_content_length: int = 10000, **kwargs) -> Dict[str, Any]:
        """
        Executes the batch scraping of URLs.
        
        This method is designed to be robust and can accept the 'urls'
        argument as either a proper list of strings or a JSON-formatted string
        that represents a list of strings.
        """
        
        # --- FIX: Robust URL input handling ---
        if isinstance(urls, str):
            try:
                # If 'urls' is a string, attempt to parse it as JSON.
                # This handles cases where the LLM or a previous tool returns a stringified list.
                parsed_urls = json.loads(urls)
                if not isinstance(parsed_urls, list):
                    return {"error": "Input string was valid JSON but not a list."}
                urls = parsed_urls
            except json.JSONDecodeError:
                # If parsing fails, it might be a single URL string.
                # Wrap it in a list to handle this case gracefully.
                urls = [urls]
        
        if not isinstance(urls, list):
            return {"error": f"Invalid type for 'urls'. Expected a list or a JSON string of a list, but got {type(urls).__name__}."}
        # --- End of FIX ---

        scraped_content = []
        failed_urls = []

        for url in urls[:10]:  # Limit to 10 URLs per batch for safety
            try:
                headers = {
                    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
                }
                response = requests.get(url, timeout=15, headers=headers)
                response.raise_for_status()
                
                soup = BeautifulSoup(response.content, 'html.parser')
                
                for script_or_style in soup(["script", "style"]):
                    script_or_style.decompose()
                
                text = ' '.join(line.strip() for line in soup.get_text().splitlines() if line.strip())
                
                if len(text) > max_content_length:
                    text = text[:max_content_length] + "..."

                scraped_content.append({
                    "url": url,
                    "title": soup.title.string.strip() if soup.title and soup.title.string else "No Title Found",
                    "content": text,
                    "scraped_at": datetime.now().isoformat()
                })

            except Exception as e:
                failed_urls.append({"url": url, "error": str(e)})

        return {
            "success": True,
            "scraped_content": scraped_content,
            "failed_urls": failed_urls
        }
