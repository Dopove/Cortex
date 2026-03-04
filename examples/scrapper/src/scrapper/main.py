# scrapper/main.py
# type: ignore
import json
import logging
import os
import requests
import numpy as np
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

import yaml
from crewai import Agent, Task, Crew, Process
from crewai.tools import BaseTool
from crewai.llm import LLM

# Local imports
from scrapper.config import create_directories, OLLAMA_BASE_URL
from scrapper.tools.llama_tools import (
    LlamaQualityAssessmentTool,
    LlamaCategorizationTool,
    LlamaRelevanceFilterTool,
    FashionMetadataExtractor,
)
from scrapper.tools.search_tools import FashionURLDiscoveryTool, URLValidatorTool
from scrapper.tools.scrape_tools import BatchScrapingTool
from scrapper.tools.dataset_tools import KaggleDatasetTool, HuggingFaceDatasetTool
from scrapper.tools.image_processing_tools import PhotoToSketchTool
from scrapper.utils.data_export import save_scraped_data_to_tsv

# Setup logging
logging.basicConfig(
    filename="scraper_activity.log",
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def load_yaml_config(filename: str) -> Dict[str, Any]:
    """Load YAML config from the config subdirectory."""
    config_dir = Path(__file__).parent / "config"
    file_path = config_dir / filename
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f)
    except Exception as e:
        logging.error(f"Error loading YAML '{filename}': {e}")
        raise


def try_fallback_downloads():
    """Fallback method if API downloads fail"""
    base_dir = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured"
    
    print("🔄 Attempting fallback dataset downloads...")
    
    # Try direct Fashion-MNIST download
    try:
        download_fashion_mnist_direct(base_dir)
    except Exception as e:
        print(f"⚠️ Fallback Fashion-MNIST download failed: {e}")
        logging.error(f"Fallback download failed: {e}")
    
    # Try alternative HuggingFace approach
    try:
        download_fashion_mnist_hf_fallback(base_dir)
    except Exception as e:
        print(f"⚠️ Fallback HuggingFace download failed: {e}")
        logging.error(f"Fallback HF download failed: {e}")


def download_fashion_mnist_direct(base_dir: str):
    """Direct download without Kaggle API"""
    print("🔽 Trying direct Fashion-MNIST download...")
    
    base_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/"
    files = [
        "train-images-idx3-ubyte.gz",
        "train-labels-idx1-ubyte.gz", 
        "t10k-images-idx3-ubyte.gz",
        "t10k-labels-idx1-ubyte.gz"
    ]
    
    os.makedirs(f"{base_dir}/fashion_mnist", exist_ok=True)
    
    success_count = 0
    for file in files:
        try:
            url = base_url + file
            response = requests.get(url, timeout=30)
            if response.status_code == 200:
                with open(f"{base_dir}/fashion_mnist/{file}", "wb") as f:
                    f.write(response.content)
                print(f"✅ Downloaded: {file}")
                success_count += 1
            else:
                print(f"❌ Failed to download: {file} (Status: {response.status_code})")
        except Exception as e:
            print(f"❌ Error downloading {file}: {e}")
    
    if success_count > 0:
        print(f"✅ Successfully downloaded {success_count}/{len(files)} Fashion-MNIST files")
    else:
        raise Exception("No files were downloaded successfully")


def download_fashion_mnist_hf_fallback(base_dir: str):
    """Alternative HuggingFace download with proper serialization"""
    print("🔽 Trying HuggingFace Fashion-MNIST fallback...")
    
    try:
        from datasets import load_dataset
        
        dataset = load_dataset("fashion_mnist")
        
        os.makedirs(f"{base_dir}/fashion_mnist_hf", exist_ok=True)
        
        # Process train split (limited samples to avoid memory issues)
        max_samples = 1000
        train_images = []
        train_labels = []
        
        for i, item in enumerate(dataset["train"]): 
            if i >= max_samples:
                break
                
            # Convert PIL Image to numpy array then to list for JSON serialization
            img_array = np.array(item["image"])
            train_images.append(img_array.tolist())
            train_labels.append(item["label"])
        
        # Save as JSON files
        with open(f"{base_dir}/fashion_mnist_hf/train_images.json", "w") as f:
            json.dump(train_images, f)
        
        with open(f"{base_dir}/fashion_mnist_hf/train_labels.json", "w") as f:
            json.dump(train_labels, f)
        
        # Process test split
        test_images = []
        test_labels = []
        
        for i, item in enumerate(dataset["test"]):
            if i >= max_samples:
                break
                
            img_array = np.array(item["image"])
            test_images.append(img_array.tolist())
            test_labels.append(item["label"])
        
        with open(f"{base_dir}/fashion_mnist_hf/test_images.json", "w") as f:
            json.dump(test_images, f)
        
        with open(f"{base_dir}/fashion_mnist_hf/test_labels.json", "w") as f:
            json.dump(test_labels, f)
            
        print(f"✅ Successfully downloaded {len(train_images)} train + {len(test_images)} test samples from HuggingFace")
        
    except ImportError:
        print("⚠️ HuggingFace datasets library not installed")
        raise Exception("datasets library not available")


def create_fashion_crew():
    """Create and configure the fashion data collection crew."""
    # Load configurations
    agents_config = load_yaml_config("agents.yaml")
    tasks_config = load_yaml_config("tasks.yaml")

    # --- LLM Configuration with Increased Timeout ---
    ollama_llm = LLM(
        model="ollama/llama3.1:8b",
        base_url=OLLAMA_BASE_URL,
        request_timeout=1200.0,  # 20 minutes timeout
    )

    # Instantiate all tools
    discovery_tools: List[BaseTool] = [FashionURLDiscoveryTool(), URLValidatorTool()]
    scraping_tools: List[BaseTool] = [BatchScrapingTool(), PhotoToSketchTool()]
    quality_tools: List[BaseTool] = [
        LlamaQualityAssessmentTool(),
        LlamaRelevanceFilterTool(),
        FashionMetadataExtractor(),
    ]
    categorization_tools: List[BaseTool] = [LlamaCategorizationTool()]
    dataset_tools: List[BaseTool] = [KaggleDatasetTool(), HuggingFaceDatasetTool()]

    # --- Agent Definitions ---
    discovery_agent = Agent(
        role=agents_config["discovery_specialist"]["role"],
        goal=agents_config["discovery_specialist"]["goal"],
        backstory=agents_config["discovery_specialist"]["backstory"],
        tools=discovery_tools,
        llm=ollama_llm,
        verbose=True,
        allow_delegation=False,
    )

    scraping_agent = Agent(
        role=agents_config["scraping_specialist"]["role"],
        goal=agents_config["scraping_specialist"]["goal"],
        backstory=agents_config["scraping_specialist"]["backstory"],
        tools=scraping_tools,
        llm=ollama_llm,
        verbose=True,
        allow_delegation=False,
    )

    quality_agent = Agent(
        role=agents_config["quality_assessor"]["role"],
        goal=agents_config["quality_assessor"]["goal"],
        backstory=agents_config["quality_assessor"]["backstory"],
        tools=quality_tools,
        llm=ollama_llm,
        verbose=True,
        allow_delegation=False,
    )

    categorization_agent = Agent(
        role=agents_config["content_categorizer"]["role"],
        goal=agents_config["content_categorizer"]["goal"],
        backstory=agents_config["content_categorizer"]["backstory"],
        tools=categorization_tools,
        llm=ollama_llm,
        verbose=True,
        allow_delegation=False,
    )

    dataset_agent = Agent(
        role=agents_config["dataset_specialist"]["role"],
        goal=agents_config["dataset_specialist"]["goal"],
        backstory=agents_config["dataset_specialist"]["backstory"],
        tools=dataset_tools,
        llm=ollama_llm,
        verbose=True,
        allow_delegation=False,
    )

    # --- Task Definitions ---
    discovery_task = Task(
        description=tasks_config["url_discovery"]["description"],
        agent=discovery_agent,
        expected_output=tasks_config["url_discovery"]["expected_output"],
    )
    
    scraping_task = Task(
        description=tasks_config["content_scraping"]["description"],
        agent=scraping_agent,
        expected_output=tasks_config["content_scraping"]["expected_output"],
    )
    
    quality_task = Task(
        description=tasks_config["quality_assessment"]["description"],
        agent=quality_agent,
        expected_output=tasks_config["quality_assessment"]["expected_output"],
    )
    
    categorization_task = Task(
        description=tasks_config["content_categorization"]["description"],
        agent=categorization_agent,
        expected_output=tasks_config["content_categorization"]["expected_output"],
    )
    
    dataset_collection_task = Task(
        description=tasks_config["dataset_collection"]["description"],
        agent=dataset_agent,
        expected_output=tasks_config["dataset_collection"]["expected_output"],
    )

    # Assemble the crew
    fashion_crew = Crew(
        agents=[
            discovery_agent,
            scraping_agent,
            quality_agent,
            categorization_agent,
            dataset_agent,
        ],
        tasks=[
            discovery_task,
            scraping_task,
            quality_task,
            categorization_task,
            dataset_collection_task,
        ],
        process=Process.sequential,
        verbose=True,
    )
    
    return fashion_crew


def main():
    """Main function to initialize and run the fashion data collection crew."""
    print("🚀 Starting Fashion Data Collection Crew...")
    
    # Create directories and setup
    create_directories()
    logging.info(f"Connecting to LLaMA at: {OLLAMA_BASE_URL}")

    # Try fallback downloads first (before crew execution)
    try_fallback_downloads()

    # Create and run the crew
    try:
        fashion_crew = create_fashion_crew()
        
        logging.info("Crew kickoff started.")
        print("🎬 Starting crew execution...")
        
        result = fashion_crew.kickoff()
        
        print("✅ Crew execution completed!")

        # --- Save Final Results ---
        output_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/results")
        output_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # 1. Save JSON
        json_filename = output_dir / f"crew_results_{timestamp}.json"
        try:
            with open(json_filename, "w", encoding="utf-8") as f:
                json.dump(result, f, indent=2, default=str)
            print(f"🎯 Raw JSON results saved to: {json_filename}")
        except Exception as e:
            logging.error(f"Failed to save JSON: {e}")
            print(f"❌ Failed to save JSON: {e}")

        # 2. Save TSV
        tsv_output_dir = Path(
            "C:/Users/saran/Videos/Projects/FDM/dataCollector/collected_data/structured/market_data"
        )
        tsv_filename = tsv_output_dir / f"fashion_data_export_{timestamp}.tsv"
        try:
            if isinstance(result, dict):
                tsv_file, record_count = save_scraped_data_to_tsv(result, tsv_filename)
                print(f"🎯 Generated {record_count} AI-ready training records")
            else:
                logging.warning("Crew result was not a dictionary, cannot save to TSV.")
                print("⚠️ Crew result format unexpected, TSV not saved.")
        except Exception as e:
            logging.error(f"Failed to save TSV: {e}")
            print(f"❌ Failed to save TSV: {e}")

        print("🎉 Fashion data collection completed successfully!")
        return result

    except Exception as e:
        logging.error(f"Crew execution failed: {e}")
        print(f"❌ Crew execution failed: {e}")
        print("💡 Check logs for more details and ensure Ollama is running.")
        return None


if __name__ == "__main__":
    main()
