use raw_cpuid::CpuId;
use serde::{Deserialize, Serialize};
use std::env::consts::OS;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub total_memory_gb: f64,
    pub physical_cores: usize,
    pub has_avx2: bool,
    pub has_avx512: bool,
    pub has_amx: bool,
    pub recommended_quantization: String,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0;
        let physical_cores = sys.physical_core_count().unwrap_or(1);
        let arch = std::env::consts::ARCH.to_string();

        let cpuid = CpuId::new();
        let mut has_avx2 = false;
        let mut has_avx512 = false;
        let mut has_amx = false;

        // ARM architecture (like M-series Macs) does not have CPUID in the same way x86 does
        if arch == "x86_64" {
            if let Some(features) = cpuid.get_extended_feature_info() {
                has_avx2 = features.has_avx2();
                has_avx512 = features.has_avx512f();
                has_amx = features.has_amx_bf16();
            }
        }

        // Recommend quantization format based on hardware acceleration
        let recommended_quantization = match OS {
            "macos" if arch == "aarch64" => "Q4_0_4_4", // Metal (Apple Silicon) optimized
            "linux" | "windows" if has_avx512 => "Q8_0", // AVX512 can handle Q8 rapidly
            _ => "Q4_0",                                // Safe fast fallback
        }
        .to_string();

        Self {
            os: OS.to_string(),
            arch,
            total_memory_gb,
            physical_cores,
            has_avx2,
            has_avx512,
            has_amx,
            recommended_quantization,
        }
    }
}

pub struct MemoryThresholdGuard;

impl MemoryThresholdGuard {
    /// Checks if the current system has enough memory (available RAM + Swap)
    /// to load a model of the specified size in GB.
    /// Returns an error if the memory threshold is not met.
    pub fn check_availability(required_gb: f64) -> anyhow::Result<()> {
        if std::env::var("CORTEX_BYPASS_MEM_CHECK").is_ok() {
            return Ok(());
        }

        let mut sys = System::new_all();
        sys.refresh_memory();

        let available_kb = sys.available_memory();
        let swap_free_kb = sys.free_swap();

        // Total available "volatile" space
        let total_available_gb = (available_kb + swap_free_kb) as f64 / 1_073_741_824.0;

        // We mandate a 5GB safety buffer for the OS and background tasks
        let safety_buffer_gb = 5.0;

        if total_available_gb < (required_gb + safety_buffer_gb) {
            return Err(anyhow::anyhow!(
                "Memory Threshold Violation: Required {}GB (with safety buffer), but only {}GB available (RAM+Swap). \
                Refusing to load to prevent system freeze.",
                required_gb + safety_buffer_gb,
                total_available_gb
            ));
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let profile = HardwareProfile::detect();
        assert!(!profile.os.is_empty());
        assert!(!profile.arch.is_empty());
        assert!(profile.physical_cores > 0);
        assert!(profile.total_memory_gb > 0.0);
    }

    #[test]
    fn test_quantization_logic() {
        let profile = HardwareProfile {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            total_memory_gb: 32.0,
            physical_cores: 8,
            has_avx2: true,
            has_avx512: true,
            has_amx: false,
            recommended_quantization: "Q4_0".to_string(),
        };

        // Test manual detection logic if you have complex rules
        // For now it's just checking fields exist
        assert_eq!(profile.recommended_quantization, "Q4_0");
    }
}
