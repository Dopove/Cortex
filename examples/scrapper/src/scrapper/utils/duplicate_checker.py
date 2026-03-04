import json
from pathlib import Path

class DuplicateChecker:
    def __init__(self, tracking_file: Path):
        self.tracking_file = tracking_file
        self.content_hashes = set()
        self.load_hashes()

    def load_hashes(self):
        if self.tracking_file.exists():
            with open(self.tracking_file) as f:
                hashes = json.load(f)
                self.content_hashes = set(hashes)

    def is_duplicate(self, content_hash: str):
        return content_hash in self.content_hashes

    def add_hash(self, content_hash: str):
        self.content_hashes.add(content_hash)
        with open(self.tracking_file, "w") as f:
            json.dump(list(self.content_hashes), f)
