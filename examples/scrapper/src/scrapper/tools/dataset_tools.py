# scrapper/tools/dataset_tools.py

from pydantic import BaseModel, Field, ConfigDict
from crewai.tools import BaseTool
from typing import Type, Dict, Any, List, Optional
from datetime import datetime
from pathlib import Path
import json
import subprocess  # <-- FIX: Added missing import

try:
    from datasets import load_dataset
    DATASETS_AVAILABLE = True
except ImportError:
    load_dataset = None
    DATASETS_AVAILABLE = False

class KaggleDatasetArgs(BaseModel):
    dataset_name: str = Field(..., description="Kaggle dataset name")
    model_config = ConfigDict(extra="allow")

class KaggleDatasetTool(BaseTool):
    # FIX: Added ': str' type annotation
    name: str = "KaggleDatasetDownloader"
    description: str = "Download and unzip fashion datasets from Kaggle."
    args_schema: Type[BaseModel] = KaggleDatasetArgs

    def _run(self, dataset_name: str, **kwargs) -> Dict[str, Any]:
        output_path = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured/kaggle_datasets"
        # ... (rest of the Kaggle tool code is correct) ...
        # NOTE: The rest of your Kaggle tool code was already robust.
        # The only required fix was the type annotation on 'name' and 'description'.
        try:
            output_dir = Path(output_path)
            output_dir.mkdir(parents=True, exist_ok=True)
            
            try:
                subprocess.run(["kaggle", "--version"], capture_output=True, check=True, text=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                return {"success": False, "error": "Kaggle CLI not found."}
            
            cmd = ["kaggle", "datasets", "download", "-d", dataset_name, "-p", str(output_path), "--unzip"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode == 0:
                return {"success": True, "message": f"Dataset {dataset_name} downloaded."}
            else:
                return {"success": False, "error": result.stderr or result.stdout}
        except Exception as e:
            return {"success": False, "error": str(e)}


class HuggingFaceDatasetArgs(BaseModel):
    dataset_name: str = Field(..., description="HuggingFace dataset name")
    split: str = Field(default="train", description="Dataset split to download")
    max_samples: int = Field(default=1000, description="Maximum samples to save")
    model_config = ConfigDict(extra="allow")

class HuggingFaceDatasetTool(BaseTool):
    # FIX: Added ': str' type annotation
    name: str = "HuggingFaceDatasetDownloader"
    description: str = "Download datasets from the HuggingFace Hub."
    args_schema: Type[BaseModel] = HuggingFaceDatasetArgs

    def _run(self, dataset_name: str, split: str = "train", max_samples: int = 1000, **kwargs) -> Dict[str, Any]:
        if not DATASETS_AVAILABLE:
            return {"success": False, "error": "'datasets' library not installed."}

        try:
            dataset = load_dataset(dataset_name, split=split, streaming=True) # type: ignore
            output_dir = Path("C:/Users/saran/Videos/Projects/FDM/dataCollector/structured/huggingface_datasets")
            output_dir.mkdir(parents=True, exist_ok=True)
            dataset_file = output_dir / f"{dataset_name.replace('/', '_')}_{split}.jsonl"

            count = 0
            with open(dataset_file, "w", encoding="utf-8") as f:
                for item in dataset:
                    if count >= max_samples: break
                    f.write(json.dumps(item, ensure_ascii=False) + "\\n")
                    count += 1

            return {"success": True, "message": f"Saved {count} samples to {dataset_file}."}
        except Exception as e:
            return {"success": False, "error": str(e)}
