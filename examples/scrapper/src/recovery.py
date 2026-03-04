import json
from datetime import datetime
from scrapper.utils.data_export import save_scraped_data_to_tsv

# Manually process the existing crew results
crew_results_file = "C:\\Users\\saran\\Videos\\Projects\\FDM\\dataCollector\\results\\crew_results_20250903_102101.json"

# Load the crew results
with open(crew_results_file, 'r', encoding='utf-8') as f:
    crew_data = json.load(f)

# Generate TSV with the new flexible function
timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
tsv_filename = f"C:\\Users\\saran\\Videos\\Projects\\FDM\\dataCollector\\results\\fashion_training_{timestamp}.tsv"

# Process with debugging
tsv_file, record_count = save_scraped_data_to_tsv(crew_data, tsv_filename)
print(f"🎯 Generated {record_count} AI-ready training records")
