def save_scraped_data_to_tsv(crew_results, filename):
    """Export data in AI-training-ready format - with format detection"""
    
    import json
    import csv
    import os
    import glob
    from datetime import datetime
    from PIL import Image
    import gzip
    import numpy as np
    
    # DEBUG: Print the crew results structure first
    print("🔍 DEBUG: Crew results structure:")
    if isinstance(crew_results, dict):
        print(f"   Keys: {list(crew_results.keys())}")
        for key, value in crew_results.items():
            print(f"   {key}: {type(value)} - {len(str(value)[:100])} chars")
    else:
        print(f"   Type: {type(crew_results)}")
        print(f"   Value: {str(crew_results)[:200]}...")
    
    training_data = []
    
    # Create training directories
    os.makedirs("dataCollector/training_images", exist_ok=True)
    os.makedirs("dataCollector/training_text", exist_ok=True)
    
    # Process Fashion-MNIST for Visual Generation Model
    fashion_mnist_path = "dataCollector/structured/fashion_mnist/"
    if os.path.exists(fashion_mnist_path):
        print("🖼️ Processing Fashion-MNIST for AI training...")
        
        try:
            # Extract Fashion-MNIST images
            with gzip.open(f"{fashion_mnist_path}/train-images-idx3-ubyte.gz", 'rb') as f:
                f.read(16)  # Skip header
                images = np.frombuffer(f.read(), np.uint8).reshape(-1, 28, 28)
            
            with gzip.open(f"{fashion_mnist_path}/train-labels-idx1-ubyte.gz", 'rb') as f:
                f.read(8)  # Skip header  
                labels = np.frombuffer(f.read(), np.uint8)
            
            class_names = ['T-shirt', 'Trouser', 'Pullover', 'Dress', 'Coat',
                          'Sandal', 'Shirt', 'Sneaker', 'Bag', 'Ankle boot']
            
            # Process first 1000 images for AI training
            for i in range(min(1000, len(images))):
                img = Image.fromarray(images[i], mode='L')
                # Resize to standard ControlNet size
                img_resized = img.resize((512, 512), Image.Resampling.LANCZOS)
                
                filename_img = f"dataCollector/training_images/fashion_mnist_{i:05d}_{class_names[labels[i]].replace('/', '_')}.png"
                img_resized.save(filename_img)
                
                training_data.append({
                    'file_path': filename_img,
                    'data_type': 'image',
                    'ai_model_target': 'visual_gen',
                    'resolution': '512x512',
                    'format': 'PNG',
                    'quality_score': 9.0,
                    'category': 'garment',
                    'subcategory': class_names[labels[i]].lower().replace('/', '_'),
                    'text_content': f"Fashion garment: {class_names[labels[i]]}",
                    'metadata_json': json.dumps({
                        'garment_type': class_names[labels[i]],
                        'color_mode': 'grayscale',
                        'source': 'fashion_mnist',
                        'training_split': 'train' if i < 800 else 'validation',
                        'original_size': '28x28',
                        'upscaled': True
                    }),
                    'training_ready': True,
                    'created_at': datetime.now().isoformat()
                })
                
            print(f"✅ Processed {min(1000, len(images))} Fashion-MNIST images")
        except Exception as e:
            print(f"⚠️ Fashion-MNIST processing error: {e}")
    
    # Process scraped content with flexible format detection
    scraped_content = []
    
    # Try different possible structures
    if isinstance(crew_results, dict):
        # Try common CrewAI result structures
        possible_keys = ['scraped_content', 'result', 'output', 'data', 'content', 'tasks_output']
        
        for key in possible_keys:
            if key in crew_results:
                content = crew_results[key]
                print(f"📝 Found scraped content in key: '{key}'")
                
                if isinstance(content, list):
                    scraped_content = content
                    break
                elif isinstance(content, str):
                    # Try to parse as JSON
                    try:
                        parsed = json.loads(content)
                        if isinstance(parsed, list):
                            scraped_content = parsed
                            break
                    except:
                        # Treat as single text content
                        scraped_content = [{'content': content, 'quality_score': 8.0}]
                        break
                elif isinstance(content, dict) and 'items' in content:
                    scraped_content = content['items']
                    break
    
    # Process any scraped content found
    if scraped_content:
        print(f"📝 Processing {len(scraped_content)} scraped items for LLM training...")
        
        for i, item in enumerate(scraped_content):
            # Handle different item structures
            if isinstance(item, str):
                item = {'content': item, 'quality_score': 7.0}
            elif not isinstance(item, dict):
                continue
                
            content_text = item.get('content', str(item))
            quality = item.get('quality_score', item.get('fashion_llm_score', 7.0))
            
            if isinstance(quality, (int, float)) and quality >= 7.0 and len(content_text) > 100:
                # Save text content for LLM fine-tuning
                text_filename = f"dataCollector/training_text/fashion_scraped_{i:05d}.txt"
                
                try:
                    with open(text_filename, 'w', encoding='utf-8') as f:
                        f.write(content_text)
                    
                    content_preview = content_text[:200] + "..." if len(content_text) > 200 else content_text
                    
                    training_data.append({
                        'file_path': text_filename,
                        'data_type': 'text',
                        'ai_model_target': 'fashion_llm',
                        'resolution': 'N/A',
                        'format': 'TXT',
                        'quality_score': quality,
                        'category': 'fashion_theory',
                        'subcategory': item.get('category', 'scraped_content'),
                        'text_content': content_preview,
                        'metadata_json': json.dumps({
                            'word_count': len(content_text.split()),
                            'source_url': item.get('url', 'crew_scraped'),
                            'scrape_date': datetime.now().isoformat(),
                            'crew_agent': item.get('agent', 'unknown')
                        }),
                        'training_ready': True,
                        'created_at': datetime.now().isoformat()
                    })
                except Exception as e:
                    print(f"⚠️ Error processing text content {i}: {e}")
    
    # Export AI-training-ready TSV
    if training_data:
        fieldnames = ['file_path', 'data_type', 'ai_model_target', 'resolution', 
                     'format', 'quality_score', 'category', 'subcategory', 
                     'text_content', 'metadata_json', 'training_ready', 'created_at']
        
        with open(filename, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames, delimiter='\t')
            writer.writeheader()
            writer.writerows(training_data)
        
        print(f"✅ AI-training-ready TSV saved: {filename}")
        print(f"📊 Total training records: {len(training_data)}")
        
        # Print summary by AI model target
        summary = {}
        for item in training_data:
            target = item['ai_model_target']
            summary[target] = summary.get(target, 0) + 1
        
        print("🎯 Training data breakdown:")
        for target, count in summary.items():
            print(f"   - {target}: {count} records")
            
        return filename, len(training_data)
    
    else:
        print("⚠️ No training data generated - check crew results structure above")
        return filename, 0
