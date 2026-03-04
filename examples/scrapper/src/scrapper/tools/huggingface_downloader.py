#type: ignore
import numpy as np
import json
import os
from datasets import load_dataset

class HuggingFaceDatasetDownloader:
    def __init__(self):
        self.base_dir = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured"
    
    def download_fashion_mnist(self, split="train", max_samples=1000):
        try:
            dataset = load_dataset("fashion_mnist")
            
            os.makedirs(f"{self.base_dir}/fashion_mnist_hf", exist_ok=True)
            
            images = []
            labels = []
            
            for i, item in enumerate(dataset[split]):
                if i >= max_samples:
                    break
                    
                # Convert PIL Image to numpy array then to list for JSON serialization
                img_array = np.array(item["image"])
                images.append(img_array.tolist())
                labels.append(item["label"])
            
            # Save as JSON files instead of trying to serialize PIL images
            with open(f"{self.base_dir}/fashion_mnist_hf/{split}_images.json", "w") as f:
                json.dump(images, f)
            
            with open(f"{self.base_dir}/fashion_mnist_hf/{split}_labels.json", "w") as f:
                json.dump(labels, f)
                
            return {
                "success": True,
                "message": f"Downloaded {len(images)} samples from {split} split"
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
