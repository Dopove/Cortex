import os
import subprocess
import random
import json
import tarfile
import zstandard as zstd
from pathlib import Path

BIN = "./target/release/cortex"
FUZZ_DIR = "fuzz_payloads"
os.makedirs(FUZZ_DIR, exist_ok=True)

def run_cortex(cmd, payload):
    try:
        result = subprocess.run([BIN, cmd, str(payload)], capture_output=True, text=True, timeout=10)
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "TIMEOUT"
    except Exception as e:
        return -2, "", str(e)

def create_bundle(name, files):
    tar_path = f"{name}.tar"
    with tarfile.open(tar_path, "w") as tar:
        for fname, content in files.items():
            fpath = Path(fname)
            fpath.write_text(content)
            tar.add(fname)
            os.remove(fname)
    
    zstd_path = f"{FUZZ_DIR}/{name}.cortex"
    cctx = zstd.ZstdCompressor()
    with open(tar_path, "rb") as f_in:
        with open(zstd_path, "wb") as f_out:
            f_out.write(cctx.compress(f_in.read()))
    os.remove(tar_path)
    return zstd_path

def test_payload(name, payload_path, expected_fail=True):
    print(f"Testing {name}...")
    rc, out, err = run_cortex("turbo", payload_path)
    
    # Check for panic
    if "panic" in out.lower() or "panic" in err.lower() or rc == 101:
        print(f"FAILED: Panic detected in {name}!")
        print(f"STDOUT: {out}")
        print(f"STDERR: {err}")
        return False
    
    if expected_fail and rc == 0:
        print(f"FAILED: Expected failure but got success in {name}")
        return False

    print(f"PASSED: {name} handled gracefully (RC: {rc})")
    return True

# 1. Random Garbage
garbage_path = f"{FUZZ_DIR}/null_garbage.cortex"
with open(garbage_path, "wb") as f:
    f.write(os.urandom(1024))
test_payload("Null Garbage", garbage_path)

# 2. Valid Zstd, Garbage Tar
tar_garbage = f"{FUZZ_DIR}/valid_zstd_garbage_tar.cortex"
cctx = zstd.ZstdCompressor()
with open(tar_garbage, "wb") as f:
    f.write(cctx.compress(os.urandom(1024)))
test_payload("Valid Zstd / Garbage Tar", tar_garbage)

# 3. Missing manifest
create_bundle("missing_manifest", {"main.py": "print('hello')"})
test_payload("Missing Manifest", f"{FUZZ_DIR}/missing_manifest.cortex")

# 4. Invalid JSON manifest
create_bundle("invalid_json", {"bundle.json": "{ invalid json }", "main.py": "print('hello')"})
test_payload("Invalid JSON Manifest", f"{FUZZ_DIR}/invalid_json.cortex")

# 5. Path Traversal Attempt in Tar (Simulated via script)
# Tar crate in Rust usually guards this, but let's see if we crash.
# Note: creating malicious tar with python is harder, skipping for 10-min MVP but noted.

# 6. manifest with zero agents
create_bundle("zero_agents", {"bundle.json": json.dumps({"agents": [], "models": []})})
test_payload("Zero Agents", f"{FUZZ_DIR}/zero_agents.cortex", expected_fail=False) # Should be OK (graceful exit)

print("--- Adversarial Arena MVP Complete ---")
