use anyhow::{anyhow, Result};
use memmap2::{MmapMut, MmapOptions};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CacheSharingPolicy {
    pub enable_prefix_sharing: bool,
    pub similarity_threshold: f64,
    pub max_shared_agents: usize,
}

impl Default for CacheSharingPolicy {
    fn default() -> Self {
        Self {
            enable_prefix_sharing: true,
            similarity_threshold: 0.85,
            max_shared_agents: 4,
        }
    }
}

pub struct CacheEntry {
    pub agent_id: usize,
    pub cache_data: Arc<RwLock<MmapMut>>,
    pub size_bytes: usize,
    pub last_access: u64,
    pub access_count: usize,
    pub is_shared: bool,
    pub shared_with: HashSet<usize>,
}

impl CacheEntry {
    pub fn new(agent_id: usize, size_bytes: usize) -> Result<Self> {
        let mmap = MmapOptions::new().len(size_bytes).map_anon()?;
        Ok(Self {
            agent_id,
            cache_data: Arc::new(RwLock::new(mmap)),
            size_bytes,
            last_access: current_time_ms(),
            access_count: 1,
            is_shared: false,
            shared_with: HashSet::new(),
        })
    }

    pub fn update_access(&mut self) {
        self.last_access = current_time_ms();
        self.access_count += 1;
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct KVCacheManager {
    pub cache_pool: HashMap<usize, CacheEntry>,
    pub sharing_policy: CacheSharingPolicy,
    pub total_allocated: usize,
    pub max_cache_size: usize,
    pub total_hits: usize,
    pub total_misses: usize,
}

impl KVCacheManager {
    pub fn new(max_size_gb: usize) -> Self {
        let max_cache_size = max_size_gb * 1024 * 1024 * 1024;
        Self {
            cache_pool: HashMap::new(),
            sharing_policy: CacheSharingPolicy::default(),
            total_allocated: 0,
            max_cache_size,
            total_hits: 0,
            total_misses: 0,
        }
    }

    pub fn allocate_cache(&mut self, agent_id: usize, size_mb: usize) -> Result<()> {
        let size_bytes = size_mb * 1024 * 1024;

        if self.total_allocated + size_bytes > self.max_cache_size {
            tracing::warn!("[KVCache] Not enough space, attempting eviction...");
            self.evict_lru();

            // Recheck space after eviction
            if self.total_allocated + size_bytes > self.max_cache_size {
                return Err(anyhow!(
                    "OOM: Unable to allocate {} bytes for agent {}",
                    size_bytes,
                    agent_id
                ));
            }
        }

        let entry = CacheEntry::new(agent_id, size_bytes)?;
        self.cache_pool.insert(agent_id, entry);
        self.total_allocated += size_bytes;

        tracing::info!("[KVCache] Allocated {} MB for agent {}", size_mb, agent_id);
        Ok(())
    }

    pub fn share_cache(&mut self, source_agent: usize, target_agent: usize) -> Result<()> {
        let max_shared = self.sharing_policy.max_shared_agents;

        let source_entry = self
            .cache_pool
            .get_mut(&source_agent)
            .ok_or_else(|| anyhow!("Source agent {} not found in cache", source_agent))?;

        if source_entry.shared_with.len() >= max_shared {
            tracing::warn!(
                "[KVCache] Max shared agents reached for source {}",
                source_agent
            );
            return Err(anyhow!("Max shared agents limit reached"));
        }

        source_entry.is_shared = true;
        source_entry.shared_with.insert(target_agent);

        tracing::info!(
            "[KVCache] Cache shared: {} -> {}",
            source_agent,
            target_agent
        );
        Ok(())
    }

    pub fn invalidate_cache(&mut self, agent_id: usize) {
        if let Some(entry) = self.cache_pool.remove(&agent_id) {
            self.total_allocated -= entry.size_bytes;
            tracing::info!("[KVCache] Invalidated cache for agent {}", agent_id);
        }
    }

    pub fn evict_lru(&mut self) {
        let mut oldest_agent: Option<usize> = None;
        let mut oldest_time = u64::MAX;

        for (agent_id, entry) in self.cache_pool.iter() {
            if entry.last_access < oldest_time && !entry.is_shared {
                oldest_time = entry.last_access;
                oldest_agent = Some(*agent_id);
            }
        }

        if let Some(id) = oldest_agent {
            self.invalidate_cache(id);
            tracing::info!("[KVCache] Evicted LRU cache for agent {}", id);
        }
    }

    pub fn get_cache(&mut self, agent_id: usize) -> Result<Arc<RwLock<MmapMut>>> {
        if let Some(entry) = self.cache_pool.get_mut(&agent_id) {
            entry.update_access();
            self.total_hits += 1;
            Ok(entry.cache_data.clone())
        } else {
            self.total_misses += 1;
            Err(anyhow!("Cache not found for agent {}", agent_id))
        }
    }

    pub fn get_stats(&self) -> String {
        let total = self.total_hits + self.total_misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            (self.total_hits as f64 / total as f64) * 100.0
        };
        let allocated_mb = self.total_allocated / (1024 * 1024);
        let max_mb = self.max_cache_size / (1024 * 1024);

        format!(
            "KV Cache Stats:\n  Allocated: {} MB / {} MB\n  Active Entries: {}\n  Hit Rate: {:.2}%\n  Total Hits: {}\n  Total Misses: {}",
            allocated_mb, max_mb, self.cache_pool.len(), hit_rate, self.total_hits, self.total_misses
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_and_retrieval() -> Result<()> {
        let mut manager = KVCacheManager::new(1); // 1GB limit
        let agent_id = 1;
        let size_mb = 10;

        manager.allocate_cache(agent_id, size_mb)?;
        assert_eq!(manager.total_allocated, size_mb * 1024 * 1024);

        let retrieved = manager.get_cache(agent_id)?;
        let data = retrieved.read().map_err(|_| anyhow!("Poisoned lock"))?;
        assert_eq!(data.len(), size_mb * 1024 * 1024);
        Ok(())
    }

    #[test]
    fn test_cache_sharing() -> Result<()> {
        let mut manager = KVCacheManager::new(1);
        manager.allocate_cache(1, 10)?;
        manager.share_cache(1, 2)?;

        let entry = manager.cache_pool.get(&1).unwrap();
        assert!(entry.is_shared);
        assert!(entry.shared_with.contains(&2));
        Ok(())
    }

    #[test]
    fn test_eviction_lru() -> Result<()> {
        let mut manager = KVCacheManager::new(1);
        // Set max size to 20MB for testing
        manager.max_cache_size = 20 * 1024 * 1024;

        manager.allocate_cache(1, 10)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.allocate_cache(2, 10)?;

        // Now memory is full (20MB). Allocate 3 (5MB), should evict 1
        manager.allocate_cache(3, 5)?;

        assert!(manager.get_cache(1).is_err());
        assert!(manager.get_cache(2).is_ok());
        assert!(manager.get_cache(3).is_ok());
        Ok(())
    }

    #[test]
    fn test_allocation_oom() {
        let mut manager = KVCacheManager::new(1); // 1GB limit
                                                  // Try to allocate 2GB
        let result = manager.allocate_cache(1, 2048);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OOM"));
    }

    #[test]
    fn test_allocation_zero() -> Result<()> {
        let mut manager = KVCacheManager::new(1);
        manager.allocate_cache(1, 0)?;
        assert_eq!(manager.total_allocated, 0);
        Ok(())
    }
}
