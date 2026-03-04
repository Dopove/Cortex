import requests
import os

class KaggleDatasetDownloader:
    def __init__(self):
        self.base_dir = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured"
    
    def download_fashion_mnist_direct(self):
        """Direct download without Kaggle API"""
        base_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/"
        files = [
            "train-images-idx3-ubyte.gz",
            "train-labels-idx1-ubyte.gz", 
            "t10k-images-idx3-ubyte.gz",
            "t10k-labels-idx1-ubyte.gz"
        ]
        
        os.makedirs(f"{self.base_dir}/fashion_mnist", exist_ok=True)
        
        for file in files:
            url = base_url + file
            response = requests.get(url)
            if response.status_code == 200:
                with open(f"{self.base_dir}/fashion_mnist/{file}", "wb") as f:
                    f.write(response.content)
                print(f"✅ Downloaded: {file}")
            else:
                print(f"❌ Failed to download: {file}")
