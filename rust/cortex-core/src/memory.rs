use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Failed to open file: {0}")]
    FileOpenError(#[from] std::io::Error),
    #[error("Model not found in cache: {0}")]
    ModelNotFound(String),
}

/// A read-only, non-owning view into the global mapped model memory.
/// This allows multiple agents/processes to reference the exact same
/// physical memory pages without copying the weight data.
#[derive(Clone)]
pub struct ModelHandle {
    model_id: String,
    // Arc to the raw Mmap ensures safe reference counting within the local process.
    // In a multi-process context, the OS manages the shared page tables!
    mmap_ptr: Arc<Mmap>,
}

impl ModelHandle {
    pub fn get_slice(&self) -> &[u8] {
        &self.mmap_ptr
    }

    pub fn id(&self) -> &str {
        &self.model_id
    }
}

/// Global registry for Memory-Mapped model weights.
/// Loads the model once from disk and distributes handles natively.
pub struct GlobalModelCache {
    mapped_models: RwLock<HashMap<String, Arc<Mmap>>>,
}

impl GlobalModelCache {
    pub fn new() -> Self {
        Self {
            mapped_models: RwLock::new(HashMap::new()),
        }
    }

    /// Loads a model into virtual memory via mmap lazily.
    pub fn load_model(&self, model_id: &str, file_path: &Path) -> Result<(), MemoryError> {
        let mut cache = self.mapped_models.write().unwrap();
        if cache.contains_key(model_id) {
            return Ok(()); // Already loaded
        }

        let file = File::open(file_path)?;
        // Safety: We assume the file is immutable while mapped.
        // For models like GGUFs, this is virtually guaranteed.
        let mmap = unsafe { Mmap::map(&file)? };

        cache.insert(model_id.to_string(), Arc::new(mmap));
        Ok(())
    }

    /// Fetches a zero-copy pointer wrapper to the mapped model.
    pub fn get_handle(&self, model_id: &str) -> Result<ModelHandle, MemoryError> {
        let cache = self.mapped_models.read().unwrap();

        if let Some(mmap_ptr) = cache.get(model_id) {
            Ok(ModelHandle {
                model_id: model_id.to_string(),
                mmap_ptr: mmap_ptr.clone(),
            })
        } else {
            Err(MemoryError::ModelNotFound(model_id.to_string()))
        }
    }
}

/// Represents an immutable CoW (Copy-on-Write) chunk of KV cache state.
/// This allows agents sharing the same system prompt to reuse the
/// expensive attention prefix computations.
#[derive(Clone)]
pub struct CacheSlice {
    pub slice_id: String,
    pub token_length: usize,
    // We use Arc to share the memory pointer for the prefix safely.
    pub data: Arc<Vec<f32>>,
}

/// The Global KV pool limits token re-encoding overhead.
pub struct SharedKVCache {
    prefixes: RwLock<HashMap<String, CacheSlice>>,
}

impl SharedKVCache {
    pub fn new() -> Self {
        Self {
            prefixes: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a newly computed immutable prefix for global reuse.
    pub fn register_prefix(&self, signature: &str, tokens: usize, data: Vec<f32>) {
        let mut prefixes = self.prefixes.write().unwrap();
        prefixes.insert(
            signature.to_string(),
            CacheSlice {
                slice_id: signature.to_string(),
                token_length: tokens,
                data: Arc::new(data),
            },
        );
    }

    /// Requests an immutable pointer to an existing KV state string.
    pub fn get_prefix(&self, signature: &str) -> Option<CacheSlice> {
        let prefixes = self.prefixes.read().unwrap();
        prefixes.get(signature).cloned()
    }
}

impl Default for GlobalModelCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SharedKVCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_global_model_cache() {
        let cache = GlobalModelCache::new();

        // Create a dummy model file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"dummy weight data").unwrap();
        let path = temp_file.path().to_path_buf();

        // Load the model
        assert!(cache.load_model("bloom-560m", &path).is_ok());

        // Handle should point to exact same data
        let handle = cache.get_handle("bloom-560m").unwrap();
        assert_eq!(handle.get_slice(), b"dummy weight data");
        assert_eq!(handle.id(), "bloom-560m");

        // Try nonexistent model
        assert!(cache.get_handle("does-not-exist").is_err());
    }

    #[test]
    fn test_shared_kv_cache() {
        let kv_cache = SharedKVCache::new();

        let signature = "sys_prompt_v1";
        let dummy_data = vec![0.1, 0.2, 0.3, 0.4];

        kv_cache.register_prefix(signature, 100, dummy_data.clone());

        let cached = kv_cache.get_prefix(signature).unwrap();
        assert_eq!(cached.slice_id, signature);
        assert_eq!(cached.token_length, 100);
        assert_eq!(*cached.data, dummy_data);

        assert!(kv_cache.get_prefix("unknown").is_none());
    }
}
