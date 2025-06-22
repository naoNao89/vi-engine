//! Runtime CPU Architecture Detection and Capability Analysis

use std::collections::HashMap;
use std::sync::OnceLock;

/// CPU architecture types with optimization capabilities
///
/// This enum may be extended with additional CPU architectures and capabilities in future versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CpuArchitecture {
    /// Apple Silicon ARM64 (M1/M2/M3/M4) with unified memory
    AppleSilicon {
        /// Apple Silicon generation (1 for M1, 2 for M2, etc.)
        generation: u8,
        /// Number of performance cores
        performance_cores: u8,
        /// Number of efficiency cores
        efficiency_cores: u8,
    },
    /// Generic ARM64 with NEON support
    GenericArm64 {
        /// Whether NEON SIMD instructions are available
        has_neon: bool,
        /// Whether advanced SIMD instructions are available
        has_advanced_simd: bool,
    },
    /// x86_64 with various SIMD capabilities
    X86_64 {
        /// Whether AVX2 instructions are available
        has_avx2: bool,
        /// Whether BMI2 instructions are available
        has_bmi2: bool,
        /// Whether AVX-512F instructions are available
        has_avx512f: bool,
        /// Whether FMA instructions are available
        has_fma: bool,
    },
    /// Other architectures with basic support
    Other {
        /// Name of the architecture
        arch_name: String,
    },
}

/// Performance tier classification for optimization selection
///
/// This enum may be extended with additional performance tiers in future versions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PerformanceTier {
    /// Ultra-high performance tier (>1B chars/sec) - Apple Silicon
    Tier1UltraHigh = 4,
    /// High performance tier (>800M chars/sec) - x86_64 with AVX-512
    Tier2High = 3,
    /// Good performance tier (>500M chars/sec) - ARM64 with NEON
    Tier3Good = 2,
    /// Basic performance tier (<500M chars/sec) - fallback implementations
    Tier4Basic = 1,
}

/// CPU capability detection results
#[derive(Debug, Clone)]
pub struct CpuCapabilities {
    /// The detected CPU architecture with specific capabilities
    pub architecture: CpuArchitecture,
    /// Performance tier classification for optimization selection
    pub performance_tier: PerformanceTier,
    /// Map of feature names to availability status
    pub features: HashMap<String, bool>,
    /// Numerical performance score for comparison
    pub performance_score: u32,
    /// Timestamp when capabilities were detected
    pub detected_at: std::time::SystemTime,
}

static CPU_CAPABILITIES: OnceLock<CpuCapabilities> = OnceLock::new();

impl CpuCapabilities {
    /// Gets the global CPU capabilities instance, detecting them if necessary.
    pub fn get() -> &'static CpuCapabilities {
        CPU_CAPABILITIES.get_or_init(Self::detect_runtime_capabilities)
    }

    /// Forces redetection of CPU capabilities (for testing purposes).
    #[cfg(test)]
    pub fn force_redetect() -> CpuCapabilities {
        Self::detect_runtime_capabilities()
    }

    fn detect_runtime_capabilities() -> CpuCapabilities {
        let mut features = HashMap::new();
        let architecture = Self::detect_architecture(&mut features);
        let (performance_tier, performance_score) =
            Self::calculate_performance_metrics(&architecture, &features);

        CpuCapabilities {
            architecture,
            performance_tier,
            features,
            performance_score,
            detected_at: std::time::SystemTime::now(),
        }
    }

    fn detect_architecture(features: &mut HashMap<String, bool>) -> CpuArchitecture {
        #[cfg(target_arch = "aarch64")]
        {
            Self::detect_arm64_architecture(features)
        }

        #[cfg(target_arch = "x86_64")]
        {
            Self::detect_x86_64_architecture(features)
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            CpuArchitecture::Other {
                arch_name: std::env::consts::ARCH.to_string(),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_arm64_architecture(features: &mut HashMap<String, bool>) -> CpuArchitecture {
        let has_neon = std::arch::is_aarch64_feature_detected!("neon");
        features.insert("neon".to_string(), has_neon);

        let has_advanced_simd = has_neon;
        features.insert("advanced_simd".to_string(), has_advanced_simd);

        if Self::is_apple_silicon() {
            let (generation, perf_cores, eff_cores) = Self::detect_apple_silicon_details();
            features.insert("apple_silicon".to_string(), true);
            features.insert("unified_memory".to_string(), true);

            CpuArchitecture::AppleSilicon {
                generation,
                performance_cores: perf_cores,
                efficiency_cores: eff_cores,
            }
        } else {
            CpuArchitecture::GenericArm64 {
                has_neon,
                has_advanced_simd,
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn detect_x86_64_architecture(features: &mut HashMap<String, bool>) -> CpuArchitecture {
        let has_avx2 = std::arch::is_x86_feature_detected!("avx2");
        let has_bmi2 = std::arch::is_x86_feature_detected!("bmi2");
        let has_avx512f = std::arch::is_x86_feature_detected!("avx512f");
        let has_fma = std::arch::is_x86_feature_detected!("fma");

        features.insert("avx2".to_string(), has_avx2);
        features.insert("bmi2".to_string(), has_bmi2);
        features.insert("avx512f".to_string(), has_avx512f);
        features.insert("fma".to_string(), has_fma);
        features.insert(
            "sse4.2".to_string(),
            std::arch::is_x86_feature_detected!("sse4.2"),
        );
        features.insert(
            "popcnt".to_string(),
            std::arch::is_x86_feature_detected!("popcnt"),
        );

        CpuArchitecture::X86_64 {
            has_avx2,
            has_bmi2,
            has_avx512f,
            has_fma,
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn is_apple_silicon() -> bool {
        cfg!(target_os = "macos")
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_apple_silicon_details() -> (u8, u8, u8) {
        (1, 4, 4) // Generation 1, 4 performance cores, 4 efficiency cores
    }

    fn calculate_performance_metrics(
        architecture: &CpuArchitecture,
        _features: &HashMap<String, bool>,
    ) -> (PerformanceTier, u32) {
        match architecture {
            CpuArchitecture::AppleSilicon { generation, .. } => {
                let base_score = 1000;
                let generation_bonus = (*generation as u32) * 100;
                (
                    PerformanceTier::Tier1UltraHigh,
                    base_score + generation_bonus,
                )
            }

            CpuArchitecture::X86_64 {
                has_avx2,
                has_bmi2,
                has_avx512f,
                has_fma,
            } => {
                let mut score = 600;
                if *has_avx2 {
                    score += 100;
                }
                if *has_bmi2 {
                    score += 50;
                }
                if *has_avx512f {
                    score += 150;
                }
                if *has_fma {
                    score += 75;
                }

                let tier = if score >= 800 {
                    PerformanceTier::Tier2High
                } else {
                    PerformanceTier::Tier3Good
                };

                (tier, score)
            }

            CpuArchitecture::GenericArm64 {
                has_neon,
                has_advanced_simd,
            } => {
                let mut score = 500;
                if *has_neon {
                    score += 100;
                }
                if *has_advanced_simd {
                    score += 50;
                }

                let tier = if score >= 600 {
                    PerformanceTier::Tier2High
                } else {
                    PerformanceTier::Tier3Good
                };

                (tier, score)
            }

            CpuArchitecture::Other { .. } => (PerformanceTier::Tier4Basic, 100),
        }
    }

    /// Checks if a specific CPU feature is available.
    #[must_use = "Feature check result should be used for conditional logic"]
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    /// Returns a human-readable description of the CPU architecture.
    #[must_use = "Architecture description should be used for logging or diagnostics"]
    pub fn architecture_description(&self) -> String {
        match &self.architecture {
            CpuArchitecture::AppleSilicon {
                generation,
                performance_cores,
                efficiency_cores,
            } => {
                format!(
                    "Apple Silicon M{} ({} performance + {} efficiency cores)",
                    generation, performance_cores, efficiency_cores
                )
            }
            CpuArchitecture::GenericArm64 { has_neon, .. } => {
                format!("Generic ARM64{}", if *has_neon { " with NEON" } else { "" })
            }
            CpuArchitecture::X86_64 {
                has_avx2,
                has_bmi2,
                has_avx512f,
                ..
            } => {
                let mut features = Vec::new();
                if *has_avx2 {
                    features.push("AVX2");
                }
                if *has_bmi2 {
                    features.push("BMI2");
                }
                if *has_avx512f {
                    features.push("AVX-512");
                }

                format!(
                    "x86_64{}",
                    if features.is_empty() {
                        String::new()
                    } else {
                        format!(" with {}", features.join(", "))
                    }
                )
            }
            CpuArchitecture::Other { arch_name } => {
                format!("Other ({})", arch_name)
            }
        }
    }

    /// Returns a human-readable description of the performance tier.
    #[must_use = "Performance description should be used for logging or diagnostics"]
    pub fn performance_description(&self) -> &'static str {
        match self.performance_tier {
            PerformanceTier::Tier1UltraHigh => "Ultra-High Performance (>1B chars/sec)",
            PerformanceTier::Tier2High => "High Performance (>800M chars/sec)",
            PerformanceTier::Tier3Good => "Good Performance (>500M chars/sec)",
            PerformanceTier::Tier4Basic => "Basic Performance (<500M chars/sec)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_detection() {
        let capabilities = CpuCapabilities::force_redetect();
        assert!(capabilities.performance_score > 0);
    }
}
