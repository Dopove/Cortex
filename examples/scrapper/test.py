import sys
import os
import json
import requests
import numpy as np
from pathlib import Path

# Add the scrapper directory to Python path
sys.path.append('C:/Users/saran/Videos/Projects/FDM/scrapper/src')

def test_direct_fashion_mnist_download():
    """Test direct Fashion-MNIST download"""
    print("🧪 Testing direct Fashion-MNIST download...")
    
    base_dir = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured"
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
            print(f"  Downloading: {url}")
            response = requests.get(url, timeout=30)
            if response.status_code == 200:
                with open(f"{base_dir}/fashion_mnist/{file}", "wb") as f:
                    f.write(response.content)
                print(f"  ✅ Downloaded: {file} ({len(response.content)} bytes)")
                success_count += 1
            else:
                print(f"  ❌ Failed: {file} (Status: {response.status_code})")
        except Exception as e:
            print(f"  ❌ Error downloading {file}: {e}")
    
    print(f"📊 Result: {success_count}/{len(files)} files downloaded successfully")
    return success_count > 0

def test_huggingface_download():
    """Test HuggingFace download with proper serialization"""
    print("🧪 Testing HuggingFace Fashion-MNIST download...")
    
    try:
        from datasets import load_dataset
        
        base_dir = "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured"
        os.makedirs(f"{base_dir}/fashion_mnist_hf", exist_ok=True)
        
        print("  Loading dataset from HuggingFace...")
        dataset = load_dataset("fashion_mnist")
        
        # Process small sample
        max_samples = 10  # Small test sample
        train_images = []
        train_labels = []
        
        print(f"  Processing {max_samples} samples...")
        for i, item in enumerate(dataset["train"]):
            if i >= max_samples:
                break
                
            # Convert PIL Image to numpy array then to list
            img_array = np.array(item["image"])
            train_images.append(img_array.tolist())
            train_labels.append(item["label"])
        
        # Save test files
        with open(f"{base_dir}/fashion_mnist_hf/test_images.json", "w") as f:
            json.dump(train_images, f)
        
        with open(f"{base_dir}/fashion_mnist_hf/test_labels.json", "w") as f:
            json.dump(train_labels, f)
            
        print(f"  ✅ Successfully processed {len(train_images)} samples")
        print(f"  ✅ Files saved to: {base_dir}/fashion_mnist_hf/")
        return True
        
    except ImportError:
        print("  ❌ HuggingFace datasets library not installed")
        print("  💡 Install with: pip install datasets")
        return False
    except Exception as e:
        print(f"  ❌ HuggingFace download failed: {e}")
        return False

def test_url_validation():
    """Test URL validation logic"""
    print("🧪 Testing URL validation logic...")
    
    fashion_keywords = [
        "fashion", "style", "clothing", "outfit", "trend", "runway",
        "designer", "apparel", "garment", "dress", "shoes", "accessories",
        "vogue", "elle", "harper", "bazaar", "glamour", "beauty"
    ]
    
    test_urls = [
        {"url": "https://www.vogue.com/fashion", "title": "Vogue Fashion", "snippet": "Latest fashion trends"},
        {"url": "https://www.wired.com/tech", "title": "WIRED Tech", "snippet": "Technology news and reviews"},
        {"url": "https://www.elle.com/beauty", "title": "Elle Beauty", "snippet": "Beauty and skincare tips"}
    ]
    
    for url_data in test_urls:
        url = url_data.get("url", "")
        title = url_data.get("title", "")
        snippet = url_data.get("snippet", "")
        
        content = f"{title} {snippet} {url}".lower()
        keyword_matches = [kw for kw in fashion_keywords if kw in content]
        
        is_relevant = len(keyword_matches) >= 1
        confidence = len(keyword_matches) / len(fashion_keywords)
        
        status = "✅ VALID" if is_relevant else "❌ INVALID"
        print(f"  {status}: {url}")
        print(f"    Keywords found: {keyword_matches}")
        print(f"    Confidence: {confidence:.2f}")
        print()

def test_directory_structure():
    """Test if required directories exist"""
    print("🧪 Testing directory structure...")
    
    required_dirs = [
        "C:/Users/saran/Videos/Projects/FDM/dataCollector",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/structured",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/textual",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/visual",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/metadata",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/logs",
        "C:/Users/saran/Videos/Projects/FDM/dataCollector/results"
    ]
    
    for dir_path in required_dirs:
        if os.path.exists(dir_path):
            print(f"  ✅ {dir_path}")
        else:
            print(f"  ❌ {dir_path} (missing)")
            try:
                os.makedirs(dir_path, exist_ok=True)
                print(f"    ✅ Created: {dir_path}")
            except Exception as e:
                print(f"    ❌ Failed to create: {e}")

def main():
    """Run all tests"""
    print("🚀 Running Fashion Data Collection Tests")
    print("=" * 50)
    
    # Test 1: Directory structure
    test_directory_structure()
    print()
    
    # Test 2: URL validation
    test_url_validation()
    print()
    
    # Test 3: Direct download
    direct_success = test_direct_fashion_mnist_download()
    print()
    
    # Test 4: HuggingFace download
    hf_success = test_huggingface_download()
    print()
    
    # Summary
    print("📋 Test Summary:")
    print(f"  Direct Fashion-MNIST download: {'✅ PASS' if direct_success else '❌ FAIL'}")
    print(f"  HuggingFace download: {'✅ PASS' if hf_success else '❌ FAIL'}")
    print()
    
    if direct_success or hf_success:
        print("🎉 At least one dataset download method is working!")
    else:
        print("⚠️ Both dataset download methods failed. Check your internet connection.")

if __name__ == "__main__":
    main()
