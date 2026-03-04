class URLValidator:
    def __init__(self):
        self.fashion_keywords = [
            "fashion", "style", "clothing", "outfit", "trend", "runway",
            "designer", "apparel", "garment", "dress", "shoes", "accessories",
            "vogue", "elle", "harper", "bazaar", "glamour", "beauty"
        ]
    
    def validate_urls(self, urls):
        valid_urls = []
        invalid_urls = []
        
        for url_data in urls:
            url = url_data.get("url", "")
            title = url_data.get("title", "")
            snippet = url_data.get("snippet", "")
            
            # Check content for fashion keywords
            content = f"{title} {snippet} {url}".lower()
            keyword_matches = [kw for kw in self.fashion_keywords if kw in content]
            
            # More lenient validation - need at least 1 keyword
            if len(keyword_matches) >= 1:
                valid_urls.append({
                    "url": url,
                    "title": title,
                    "snippet": snippet,
                    "relevance_score": len(keyword_matches) / len(self.fashion_keywords),
                    "keywords_found": keyword_matches
                })
            else:
                invalid_urls.append({
                    "url": url,
                    "reason": f"No fashion keywords found. Content: {content[:100]}..."
                })
        
        return {
            "success": True,
            "valid_urls": valid_urls,
            "invalid_urls": invalid_urls
        }
