//! Dynamic Assembly Optimization Selection System
//!
//! This module provides automatic selection of the best available optimization
//! strategy based on runtime CPU detection and performance characteristics.

use crate::runtime_detection::{CpuArchitecture, CpuCapabilities, PerformanceTier};
use crate::safety::{AssemblyError, SafeAssemblyProcessor};
use std::sync::OnceLock;

/// Available optimization strategies in order of preference
///
/// This enum may be extended with additional optimization strategies in future versions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OptimizationStrategy {
    /// Apple Silicon assembly kernels with NEON vectorization
    AppleSiliconAssembly,
    /// Generic ARM64 assembly with NEON support
    GenericArm64Assembly,
    /// x86_64 assembly with SIMD optimizations
    X86_64Assembly,
    /// Optimized Rust implementation with compiler vectorization
    RustOptimized,
    /// Standard Rust implementation (fallback)
    RustStandard,
}

/// Performance characteristics for each optimization strategy
#[derive(Debug, Clone)]
pub struct OptimizationProfile {
    /// Strategy identifier
    pub strategy: OptimizationStrategy,
    /// Expected performance tier
    pub performance_tier: PerformanceTier,
    /// Estimated characters per second throughput
    pub estimated_throughput: u64,
    /// Initialization overhead in nanoseconds
    pub init_overhead_ns: u64,
    /// Per-operation overhead in nanoseconds
    pub operation_overhead_ns: u64,
    /// Whether this strategy is available on current platform
    pub available: bool,
    /// Reason if not available
    pub unavailable_reason: Option<String>,
}

/// Optimization selector that chooses the best strategy
pub struct OptimizationSelector {
    /// Available optimization profiles
    profiles: Vec<OptimizationProfile>,
    /// Selected optimization strategy
    selected_strategy: OptimizationStrategy,
    /// CPU capabilities used for selection
    cpu_capabilities: &'static CpuCapabilities,
}

/// Global optimization selector instance
static OPTIMIZATION_SELECTOR: OnceLock<OptimizationSelector> = OnceLock::new();

impl OptimizationSelector {
    /// Get the global optimization selector instance
    pub fn get() -> &'static OptimizationSelector {
        OPTIMIZATION_SELECTOR.get_or_init(Self::initialize)
    }

    /// Initialize the optimization selector
    fn initialize() -> Self {
        let cpu_capabilities = CpuCapabilities::get();
        let profiles = Self::create_optimization_profiles(cpu_capabilities);
        let selected_strategy = Self::select_best_strategy(&profiles);

        OptimizationSelector {
            profiles,
            selected_strategy,
            cpu_capabilities,
        }
    }

    /// Create optimization profiles for all strategies
    fn create_optimization_profiles(
        cpu_capabilities: &CpuCapabilities,
    ) -> Vec<OptimizationProfile> {
        vec![
            Self::create_apple_silicon_profile(cpu_capabilities),
            Self::create_generic_arm64_profile(cpu_capabilities),
            Self::create_x86_64_profile(cpu_capabilities),
            Self::create_rust_optimized_profile(),
            Self::create_rust_standard_profile(),
        ]
    }

    /// Create Apple Silicon optimization profile
    fn create_apple_silicon_profile(cpu_capabilities: &CpuCapabilities) -> OptimizationProfile {
        let (available, unavailable_reason) = match &cpu_capabilities.architecture {
            CpuArchitecture::AppleSilicon { .. } => {
                // Check if assembly is compiled in
                #[cfg(feature = "apple_silicon_assembly")]
                {
                    (true, None)
                }
                #[cfg(not(feature = "apple_silicon_assembly"))]
                {
                    (
                        false,
                        Some("Apple Silicon assembly not compiled".to_string()),
                    )
                }
            }
            _ => (false, Some("Not running on Apple Silicon".to_string())),
        };

        OptimizationProfile {
            strategy: OptimizationStrategy::AppleSiliconAssembly,
            performance_tier: PerformanceTier::Tier1UltraHigh,
            estimated_throughput: 1_200_000_000, // 1.2B chars/sec
            init_overhead_ns: 1_000,
            operation_overhead_ns: 1,
            available,
            unavailable_reason,
        }
    }

    /// Create generic ARM64 optimization profile
    fn create_generic_arm64_profile(cpu_capabilities: &CpuCapabilities) -> OptimizationProfile {
        let (available, unavailable_reason) = match &cpu_capabilities.architecture {
            CpuArchitecture::GenericArm64 { has_neon, .. } => {
                if *has_neon {
                    #[cfg(feature = "aarch64_assembly")]
                    {
                        (true, None)
                    }
                    #[cfg(not(feature = "aarch64_assembly"))]
                    {
                        (false, Some("ARM64 assembly not compiled".to_string()))
                    }
                } else {
                    (false, Some("NEON not available".to_string()))
                }
            }
            CpuArchitecture::AppleSilicon { .. } => {
                // Apple Silicon can use generic ARM64 as fallback
                #[cfg(feature = "aarch64_assembly")]
                {
                    (true, None)
                }
                #[cfg(not(feature = "aarch64_assembly"))]
                {
                    (false, Some("ARM64 assembly not compiled".to_string()))
                }
            }
            _ => (false, Some("Not running on ARM64".to_string())),
        };

        OptimizationProfile {
            strategy: OptimizationStrategy::GenericArm64Assembly,
            performance_tier: PerformanceTier::Tier2High,
            estimated_throughput: 800_000_000, // 800M chars/sec
            init_overhead_ns: 2_000,
            operation_overhead_ns: 2,
            available,
            unavailable_reason,
        }
    }

    /// Create x86_64 optimization profile
    fn create_x86_64_profile(cpu_capabilities: &CpuCapabilities) -> OptimizationProfile {
        let (available, unavailable_reason) = match &cpu_capabilities.architecture {
            CpuArchitecture::X86_64 { has_avx2, .. } => {
                if *has_avx2 {
                    #[cfg(feature = "x86_64_assembly")]
                    {
                        (true, None)
                    }
                    #[cfg(not(feature = "x86_64_assembly"))]
                    {
                        (false, Some("x86_64 assembly not compiled".to_string()))
                    }
                } else {
                    (false, Some("AVX2 not available".to_string()))
                }
            }
            _ => (false, Some("Not running on x86_64".to_string())),
        };

        // Adjust throughput based on specific features
        let mut throughput = 600_000_000u64; // Base 600M chars/sec
        if let CpuArchitecture::X86_64 {
            has_avx2,
            has_bmi2,
            has_avx512f,
            ..
        } = &cpu_capabilities.architecture
        {
            if *has_avx2 {
                throughput += 100_000_000;
            }
            if *has_bmi2 {
                throughput += 50_000_000;
            }
            if *has_avx512f {
                throughput += 200_000_000;
            }
        }

        OptimizationProfile {
            strategy: OptimizationStrategy::X86_64Assembly,
            performance_tier: PerformanceTier::Tier2High,
            estimated_throughput: throughput,
            init_overhead_ns: 3_000,
            operation_overhead_ns: 3,
            available,
            unavailable_reason,
        }
    }

    /// Create optimized Rust profile
    fn create_rust_optimized_profile() -> OptimizationProfile {
        OptimizationProfile {
            strategy: OptimizationStrategy::RustOptimized,
            performance_tier: PerformanceTier::Tier3Good,
            estimated_throughput: 500_000_000, // 500M chars/sec
            init_overhead_ns: 100,
            operation_overhead_ns: 5,
            available: true, // Always available
            unavailable_reason: None,
        }
    }

    /// Create standard Rust profile
    fn create_rust_standard_profile() -> OptimizationProfile {
        OptimizationProfile {
            strategy: OptimizationStrategy::RustStandard,
            performance_tier: PerformanceTier::Tier4Basic,
            estimated_throughput: 200_000_000, // 200M chars/sec
            init_overhead_ns: 50,
            operation_overhead_ns: 10,
            available: true, // Always available
            unavailable_reason: None,
        }
    }

    /// Select the best available optimization strategy
    fn select_best_strategy(profiles: &[OptimizationProfile]) -> OptimizationStrategy {
        // Find the highest performance available strategy
        profiles
            .iter()
            .filter(|profile| profile.available)
            .max_by_key(|profile| profile.estimated_throughput)
            .map(|profile| profile.strategy.clone())
            .unwrap_or(OptimizationStrategy::RustStandard)
    }

    /// Get the selected optimization strategy
    #[must_use = "Selected strategy should be used for processor creation or diagnostics"]
    pub fn selected_strategy(&self) -> &OptimizationStrategy {
        &self.selected_strategy
    }

    /// Get all optimization profiles
    #[must_use = "Optimization profiles should be used for analysis or selection"]
    pub fn profiles(&self) -> &[OptimizationProfile] {
        &self.profiles
    }

    /// Get the profile for the selected strategy
    #[must_use = "Selected profile should be used for performance analysis"]
    pub fn selected_profile(&self) -> Option<&OptimizationProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.strategy == self.selected_strategy)
    }

    /// Get CPU capabilities used for selection
    #[must_use = "CPU capabilities should be used for analysis or diagnostics"]
    pub fn cpu_capabilities(&self) -> &CpuCapabilities {
        self.cpu_capabilities
    }

    /// Create a processor using the selected optimization strategy
    #[must_use = "Created processor should be used for text processing"]
    pub fn create_processor(&self) -> Result<Box<dyn VietnameseProcessor>, AssemblyError> {
        match &self.selected_strategy {
            OptimizationStrategy::AppleSiliconAssembly => {
                Ok(Box::new(AppleSiliconProcessor::new()?))
            }
            OptimizationStrategy::GenericArm64Assembly => {
                Ok(Box::new(GenericArm64Processor::new()?))
            }
            OptimizationStrategy::X86_64Assembly => Ok(Box::new(X86_64Processor::new()?)),
            OptimizationStrategy::RustOptimized => Ok(Box::new(RustOptimizedProcessor::new())),
            OptimizationStrategy::RustStandard => Ok(Box::new(RustStandardProcessor::new())),
        }
    }

    /// Get optimization summary for diagnostics
    #[must_use = "Optimization summary should be used for logging or diagnostics"]
    pub fn optimization_summary(&self) -> String {
        let selected_profile = self.selected_profile().unwrap();

        format!(
            "Selected: {:?}\nPerformance: {}\nThroughput: {:.1}M chars/sec\nCPU: {}",
            self.selected_strategy,
            self.cpu_capabilities.performance_description(),
            selected_profile.estimated_throughput as f64 / 1_000_000.0,
            self.cpu_capabilities.architecture_description()
        )
    }
}

/// Trait for Vietnamese text processors with different optimization strategies
pub trait VietnameseProcessor: Send + Sync {
    /// Process a single character
    fn process_char(&self, ch: char) -> Result<char, AssemblyError>;

    /// Process a string
    fn process_string(&self, input: &str) -> Result<String, AssemblyError>;

    /// Get processor name for diagnostics
    fn processor_name(&self) -> &'static str;

    /// Get estimated performance characteristics
    fn performance_info(&self) -> &OptimizationProfile;
}

// Processor implementations will be added in the next section
// For now, we'll create placeholder structs

/// Apple Silicon optimized processor
pub struct AppleSiliconProcessor {
    safe_processor: SafeAssemblyProcessor,
    profile: OptimizationProfile,
}

impl AppleSiliconProcessor {
    /// Creates a new Apple Silicon processor with optimized assembly kernels.
    pub fn new() -> Result<Self, AssemblyError> {
        let profile = OptimizationSelector::get()
            .profiles()
            .iter()
            .find(|p| p.strategy == OptimizationStrategy::AppleSiliconAssembly)
            .unwrap()
            .clone();

        Ok(AppleSiliconProcessor {
            safe_processor: SafeAssemblyProcessor::new(),
            profile,
        })
    }
}

/// Generic ARM64 processor
pub struct GenericArm64Processor {
    safe_processor: SafeAssemblyProcessor,
    profile: OptimizationProfile,
}

impl GenericArm64Processor {
    /// Creates a new generic ARM64 processor with NEON optimizations.
    pub fn new() -> Result<Self, AssemblyError> {
        let profile = OptimizationSelector::get()
            .profiles()
            .iter()
            .find(|p| p.strategy == OptimizationStrategy::GenericArm64Assembly)
            .unwrap()
            .clone();

        Ok(GenericArm64Processor {
            safe_processor: SafeAssemblyProcessor::new(),
            profile,
        })
    }
}

/// x86_64 optimized processor
pub struct X86_64Processor {
    safe_processor: SafeAssemblyProcessor,
    profile: OptimizationProfile,
}

impl X86_64Processor {
    /// Creates a new x86_64 processor with AVX-512 and BMI2 optimizations.
    pub fn new() -> Result<Self, AssemblyError> {
        let profile = OptimizationSelector::get()
            .profiles()
            .iter()
            .find(|p| p.strategy == OptimizationStrategy::X86_64Assembly)
            .unwrap()
            .clone();

        Ok(X86_64Processor {
            safe_processor: SafeAssemblyProcessor::new(),
            profile,
        })
    }
}

/// Rust optimized processor
pub struct RustOptimizedProcessor {
    profile: OptimizationProfile,
}

impl Default for RustOptimizedProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustOptimizedProcessor {
    /// Creates a new Rust optimized processor with performance enhancements.
    pub fn new() -> Self {
        let profile = OptimizationSelector::get()
            .profiles()
            .iter()
            .find(|p| p.strategy == OptimizationStrategy::RustOptimized)
            .unwrap()
            .clone();

        RustOptimizedProcessor { profile }
    }
}

/// Standard Rust processor
pub struct RustStandardProcessor {
    profile: OptimizationProfile,
}

impl Default for RustStandardProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustStandardProcessor {
    /// Creates a new standard Rust processor with baseline implementation.
    pub fn new() -> Self {
        let profile = OptimizationSelector::get()
            .profiles()
            .iter()
            .find(|p| p.strategy == OptimizationStrategy::RustStandard)
            .unwrap()
            .clone();

        RustStandardProcessor { profile }
    }
}

// Implement VietnameseProcessor trait for all processor types

impl VietnameseProcessor for AppleSiliconProcessor {
    fn process_char(&self, ch: char) -> Result<char, AssemblyError> {
        // Use safe assembly processor for Apple Silicon optimized processing
        let input = vec![ch];
        let result = self.safe_processor.process_chars_safe(&input)?;
        Ok(result.into_iter().next().unwrap_or(ch))
    }

    fn process_string(&self, input: &str) -> Result<String, AssemblyError> {
        self.safe_processor.process_string_safe(input)
    }

    fn processor_name(&self) -> &'static str {
        "Apple Silicon Assembly"
    }

    fn performance_info(&self) -> &OptimizationProfile {
        &self.profile
    }
}

impl VietnameseProcessor for GenericArm64Processor {
    fn process_char(&self, ch: char) -> Result<char, AssemblyError> {
        let input = vec![ch];
        let result = self.safe_processor.process_chars_safe(&input)?;
        Ok(result.into_iter().next().unwrap_or(ch))
    }

    fn process_string(&self, input: &str) -> Result<String, AssemblyError> {
        self.safe_processor.process_string_safe(input)
    }

    fn processor_name(&self) -> &'static str {
        "Generic ARM64 Assembly"
    }

    fn performance_info(&self) -> &OptimizationProfile {
        &self.profile
    }
}

impl VietnameseProcessor for X86_64Processor {
    fn process_char(&self, ch: char) -> Result<char, AssemblyError> {
        let input = vec![ch];
        let result = self.safe_processor.process_chars_safe(&input)?;
        Ok(result.into_iter().next().unwrap_or(ch))
    }

    fn process_string(&self, input: &str) -> Result<String, AssemblyError> {
        self.safe_processor.process_string_safe(input)
    }

    fn processor_name(&self) -> &'static str {
        "x86_64 Assembly"
    }

    fn performance_info(&self) -> &OptimizationProfile {
        &self.profile
    }
}

impl VietnameseProcessor for RustOptimizedProcessor {
    fn process_char(&self, ch: char) -> Result<char, AssemblyError> {
        // Use optimized Rust implementation
        Ok(crate::util::clean_char(ch))
    }

    fn process_string(&self, input: &str) -> Result<String, AssemblyError> {
        Ok(crate::util::clean_string(input))
    }

    fn processor_name(&self) -> &'static str {
        "Rust Optimized"
    }

    fn performance_info(&self) -> &OptimizationProfile {
        &self.profile
    }
}

impl VietnameseProcessor for RustStandardProcessor {
    fn process_char(&self, ch: char) -> Result<char, AssemblyError> {
        // Use standard Rust implementation
        Ok(crate::util::clean_char(ch))
    }

    fn process_string(&self, input: &str) -> Result<String, AssemblyError> {
        Ok(crate::util::clean_string(input))
    }

    fn processor_name(&self) -> &'static str {
        "Rust Standard"
    }

    fn performance_info(&self) -> &OptimizationProfile {
        &self.profile
    }
}
