//! Production-Ready Vietnamese Text Processing API
//!
//! This module provides a high-level, user-friendly API that automatically
//! selects the best available optimization strategy while maintaining
//! comprehensive safety guarantees and monitoring capabilities.

use crate::optimization_selector::{
    OptimizationSelector, OptimizationStrategy, VietnameseProcessor,
};
use crate::runtime_detection::CpuCapabilities;
use crate::safety::AssemblyError;
use std::time::Instant;

/// High-level Vietnamese text processor with automatic optimization selection
pub struct VietnameseTextProcessor {
    /// The underlying processor implementation
    processor: Box<dyn VietnameseProcessor>,
    /// Processing statistics
    stats: ProcessingStats,
    /// Configuration options
    config: ProcessorConfig,
}

/// User preference for optimization strategy selection
///
/// This enum may be extended with additional optimization preferences in future versions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum OptimizationPreference {
    /// Automatically select the best available strategy (default)
    #[default]
    Auto,
    /// Force Rust-only processing (no assembly)
    ForceRustOnly,
    /// Force assembly if available, error if not available
    ForceAssembly,
    /// Prefer Rust over assembly but allow assembly fallback
    PreferRust,
    /// Prefer assembly over Rust but allow Rust fallback
    PreferAssembly,
    /// Force a specific optimization strategy
    ForceSpecific(OptimizationStrategy),
}

/// Configuration options for the Vietnamese processor
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Timeout for individual operations (in milliseconds)
    pub operation_timeout_ms: u64,
    /// Enable automatic fallback on errors
    pub enable_fallback: bool,
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// User preference for optimization strategy selection
    pub optimization_preference: OptimizationPreference,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        ProcessorConfig {
            enable_monitoring: true,
            operation_timeout_ms: 5000, // 5 seconds
            enable_fallback: true,
            max_retries: 2,
            optimization_preference: OptimizationPreference::default(),
        }
    }
}

/// Processing statistics for monitoring and diagnostics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Total number of characters processed
    pub total_chars_processed: u64,
    /// Total number of strings processed
    pub total_strings_processed: u64,
    /// Total processing time in nanoseconds
    pub total_processing_time_ns: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Number of fallback operations
    pub fallback_operations: u64,
    /// Average processing time per character (nanoseconds)
    pub avg_time_per_char_ns: f64,
    /// Peak processing rate (chars/second)
    pub peak_processing_rate: f64,
}

/// Builder for creating configured Vietnamese processors
#[must_use = "ProcessorBuilder must be built with .build() to create a processor"]
pub struct ProcessorBuilder {
    config: ProcessorConfig,
}

impl ProcessorBuilder {
    /// Create a new processor builder with default configuration
    pub fn new() -> Self {
        ProcessorBuilder {
            config: ProcessorConfig::default(),
        }
    }

    /// Enable or disable performance monitoring
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_monitoring(mut self, enable: bool) -> Self {
        self.config.enable_monitoring = enable;
        self
    }

    /// Set operation timeout in milliseconds
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.operation_timeout_ms = timeout_ms;
        self
    }

    /// Enable or disable automatic fallback on errors
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_fallback(mut self, enable: bool) -> Self {
        self.config.enable_fallback = enable;
        self
    }

    /// Set maximum retry attempts
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set optimization strategy preference
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_optimization_preference(mut self, preference: OptimizationPreference) -> Self {
        self.config.optimization_preference = preference;
        self
    }

    /// Force Rust-only processing (no assembly optimizations)
    ///
    /// This is useful for:
    /// - Security/audit requirements where assembly code is not allowed
    /// - Deployment environments that don't support assembly
    /// - Predictable behavior across all platforms
    /// - Debugging and development scenarios
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn force_rust_only(mut self) -> Self {
        self.config.optimization_preference = OptimizationPreference::ForceRustOnly;
        self
    }

    /// Force assembly optimizations if available, error if not available
    ///
    /// This will use the best available assembly optimization for the current
    /// architecture, or return an error if no assembly optimizations are available.
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn force_assembly(mut self) -> Self {
        self.config.optimization_preference = OptimizationPreference::ForceAssembly;
        self
    }

    /// Prefer Rust implementations over assembly
    ///
    /// This will use Rust implementations when available, but fall back to
    /// assembly if Rust implementations fail or are not available.
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn prefer_rust(mut self) -> Self {
        self.config.optimization_preference = OptimizationPreference::PreferRust;
        self
    }

    /// Prefer assembly implementations over Rust
    ///
    /// This will use assembly implementations when available, but fall back to
    /// Rust if assembly implementations fail or are not available.
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn prefer_assembly(mut self) -> Self {
        self.config.optimization_preference = OptimizationPreference::PreferAssembly;
        self
    }

    /// Force a specific optimization strategy
    ///
    /// This will attempt to use the exact strategy specified, or return an error
    /// if the strategy is not available on the current platform.
    #[must_use = "Builder methods return a new builder and must be chained or assigned"]
    pub fn with_strategy(mut self, strategy: OptimizationStrategy) -> Self {
        self.config.optimization_preference = OptimizationPreference::ForceSpecific(strategy);
        self
    }

    /// Build the Vietnamese processor
    #[must_use = "build() consumes the builder and returns the processor"]
    pub fn build(self) -> Result<VietnameseTextProcessor, AssemblyError> {
        VietnameseTextProcessor::with_config(self.config)
    }
}

impl Default for ProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VietnameseTextProcessor {
    /// Create a new Vietnamese processor with automatic optimization selection
    pub fn new() -> Result<Self, AssemblyError> {
        Self::with_config(ProcessorConfig::default())
    }

    /// Create a Vietnamese processor with custom configuration
    pub fn with_config(config: ProcessorConfig) -> Result<Self, AssemblyError> {
        let processor = Self::create_processor_with_preference(&config.optimization_preference)?;

        Ok(VietnameseTextProcessor {
            processor,
            stats: ProcessingStats::default(),
            config,
        })
    }

    /// Create a processor based on optimization preference
    fn create_processor_with_preference(
        preference: &OptimizationPreference,
    ) -> Result<Box<dyn VietnameseProcessor>, AssemblyError> {
        let selector = OptimizationSelector::get();

        match preference {
            OptimizationPreference::Auto => {
                // Use automatic selection (existing behavior)
                selector.create_processor()
            }

            OptimizationPreference::ForceRustOnly => {
                // Force Rust-only processing
                Self::create_rust_processor()
            }

            OptimizationPreference::ForceAssembly => {
                // Force assembly, error if not available
                Self::create_assembly_processor(selector)
            }

            OptimizationPreference::PreferRust => {
                // Try Rust first, fall back to assembly
                Self::create_rust_processor().or_else(|_| selector.create_processor())
            }

            OptimizationPreference::PreferAssembly => {
                // Try assembly first, fall back to Rust
                Self::create_assembly_processor(selector).or_else(|_| Self::create_rust_processor())
            }

            OptimizationPreference::ForceSpecific(strategy) => {
                // Force specific strategy
                Self::create_specific_processor(strategy, selector)
            }
        }
    }

    /// Create a Rust-only processor (optimized or standard)
    fn create_rust_processor() -> Result<Box<dyn VietnameseProcessor>, AssemblyError> {
        // Try optimized Rust first, fall back to standard Rust
        let processor = crate::optimization_selector::RustOptimizedProcessor::new();
        Ok(Box::new(processor))
    }

    /// Create an assembly processor (best available)
    fn create_assembly_processor(
        selector: &OptimizationSelector,
    ) -> Result<Box<dyn VietnameseProcessor>, AssemblyError> {
        // Find the best available assembly strategy
        let assembly_strategies = [
            OptimizationStrategy::AppleSiliconAssembly,
            OptimizationStrategy::GenericArm64Assembly,
            OptimizationStrategy::X86_64Assembly,
        ];

        for strategy in &assembly_strategies {
            if let Some(profile) = selector.profiles().iter().find(|p| &p.strategy == strategy) {
                if profile.available {
                    return Self::create_specific_processor(strategy, selector);
                }
            }
        }

        Err(AssemblyError::ExecutionError(
            "No assembly optimizations available on this platform".to_string(),
        ))
    }

    /// Create a processor for a specific strategy
    fn create_specific_processor(
        strategy: &OptimizationStrategy,
        selector: &OptimizationSelector,
    ) -> Result<Box<dyn VietnameseProcessor>, AssemblyError> {
        // Check if the strategy is available
        if let Some(profile) = selector.profiles().iter().find(|p| &p.strategy == strategy) {
            if !profile.available {
                let reason = profile
                    .unavailable_reason
                    .as_deref()
                    .unwrap_or("Strategy not available on this platform");
                return Err(AssemblyError::ExecutionError(format!(
                    "Strategy {:?} not available: {}",
                    strategy, reason
                )));
            }
        }

        // Create the processor for the specific strategy
        match strategy {
            OptimizationStrategy::AppleSiliconAssembly => Ok(Box::new(
                crate::optimization_selector::AppleSiliconProcessor::new()?,
            )),
            OptimizationStrategy::GenericArm64Assembly => Ok(Box::new(
                crate::optimization_selector::GenericArm64Processor::new()?,
            )),
            OptimizationStrategy::X86_64Assembly => Ok(Box::new(
                crate::optimization_selector::X86_64Processor::new()?,
            )),
            OptimizationStrategy::RustOptimized => Ok(Box::new(
                crate::optimization_selector::RustOptimizedProcessor::new(),
            )),
            OptimizationStrategy::RustStandard => Ok(Box::new(
                crate::optimization_selector::RustStandardProcessor::new(),
            )),
        }
    }

    /// Process a single Vietnamese character, removing diacritics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vi::VietnameseTextProcessor;
    ///
    /// let mut processor = VietnameseTextProcessor::new()?;  // Fixed: added mut
    /// let result = processor.process_char('ế')?;
    /// assert_eq!(result, 'e');
    /// # Ok::<(), vi::AssemblyError>(())
    /// ```
    pub fn process_char(&mut self, ch: char) -> Result<char, AssemblyError> {
        let start_time = if self.config.enable_monitoring {
            Some(Instant::now())
        } else {
            None
        };

        let result = self.process_char_with_retry(ch, self.config.max_retries);

        if let Some(start) = start_time {
            self.update_stats_char(start, result.is_ok());
        }

        result
    }

    /// Process a Vietnamese string, removing diacritics from all characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vi::VietnameseTextProcessor;
    ///
    /// let mut processor = VietnameseTextProcessor::new()?;  // Fixed: added mut
    /// let result = processor.process_string("Tiếng Việt")?;
    /// assert_eq!(result, "Tieng Viet");
    /// # Ok::<(), vi::AssemblyError>(())
    /// ```
    pub fn process_string(&mut self, input: &str) -> Result<String, AssemblyError> {
        if input.is_empty() {
            return Ok(String::new());
        }

        let start_time = if self.config.enable_monitoring {
            Some(Instant::now())
        } else {
            None
        };

        let result = self.process_string_with_retry(input, self.config.max_retries);

        if let Some(start) = start_time {
            self.update_stats_string(start, input.chars().count(), result.is_ok());
        }

        result
    }

    /// Process character with retry logic
    fn process_char_with_retry(
        &mut self,
        ch: char,
        retries_left: u32,
    ) -> Result<char, AssemblyError> {
        match self.processor.process_char(ch) {
            Ok(result) => Ok(result),
            Err(error) if retries_left > 0 && self.config.enable_fallback => {
                // Try fallback or retry
                match error {
                    AssemblyError::Timeout | AssemblyError::ExecutionError(_) => {
                        // Try with fallback processor
                        self.fallback_process_char(ch)
                    }
                    _ => {
                        // Retry with same processor
                        self.process_char_with_retry(ch, retries_left - 1)
                    }
                }
            }
            Err(error) => Err(error),
        }
    }

    /// Process string with retry logic
    fn process_string_with_retry(
        &mut self,
        input: &str,
        retries_left: u32,
    ) -> Result<String, AssemblyError> {
        match self.processor.process_string(input) {
            Ok(result) => Ok(result),
            Err(error) if retries_left > 0 && self.config.enable_fallback => match error {
                AssemblyError::Timeout | AssemblyError::ExecutionError(_) => {
                    self.fallback_process_string(input)
                }
                _ => self.process_string_with_retry(input, retries_left - 1),
            },
            Err(error) => Err(error),
        }
    }

    /// Fallback character processing using pure Rust
    fn fallback_process_char(&mut self, ch: char) -> Result<char, AssemblyError> {
        self.stats.fallback_operations += 1;
        Ok(crate::util::clean_char(ch))
    }

    /// Fallback string processing using pure Rust
    fn fallback_process_string(&mut self, input: &str) -> Result<String, AssemblyError> {
        self.stats.fallback_operations += 1;
        Ok(crate::util::clean_string(input))
    }

    /// Update statistics for character processing
    fn update_stats_char(&mut self, start_time: Instant, success: bool) {
        let elapsed = start_time.elapsed();
        self.stats.total_chars_processed += 1;
        self.stats.total_processing_time_ns += elapsed.as_nanos() as u64;

        if success {
            self.stats.successful_operations += 1;
        } else {
            self.stats.failed_operations += 1;
        }

        self.update_derived_stats();
    }

    /// Update statistics for string processing
    fn update_stats_string(&mut self, start_time: Instant, char_count: usize, success: bool) {
        let elapsed = start_time.elapsed();
        self.stats.total_strings_processed += 1;
        self.stats.total_chars_processed += char_count as u64;
        self.stats.total_processing_time_ns += elapsed.as_nanos() as u64;

        if success {
            self.stats.successful_operations += 1;
        } else {
            self.stats.failed_operations += 1;
        }

        // Calculate instantaneous processing rate
        if elapsed.as_nanos() > 0 {
            let rate = (char_count as f64) / elapsed.as_secs_f64();
            if rate > self.stats.peak_processing_rate {
                self.stats.peak_processing_rate = rate;
            }
        }

        self.update_derived_stats();
    }

    /// Update derived statistics
    fn update_derived_stats(&mut self) {
        if self.stats.total_chars_processed > 0 {
            self.stats.avg_time_per_char_ns = self.stats.total_processing_time_ns as f64
                / self.stats.total_chars_processed as f64;
        }
    }

    /// Get current processing statistics
    #[must_use = "Getting statistics should be used for monitoring or diagnostics"]
    pub fn stats(&self) -> &ProcessingStats {
        &self.stats
    }

    /// Get processor configuration
    #[must_use = "Getting configuration should be used for inspection or debugging"]
    pub fn config(&self) -> &ProcessorConfig {
        &self.config
    }

    /// Get information about the selected optimization strategy
    #[must_use = "Optimization info should be used for diagnostics or logging"]
    pub fn optimization_info(&self) -> String {
        let processor_name = self.processor.processor_name();
        let performance_info = self.processor.performance_info();
        let preference = &self.config.optimization_preference;

        format!(
            "Selected: {} ({:?})\nPreference: {:?}\nPerformance: {}\nThroughput: {:.1}M chars/sec",
            processor_name,
            performance_info.strategy,
            preference,
            self.cpu_info().performance_description(),
            performance_info.estimated_throughput as f64 / 1_000_000.0
        )
    }

    /// Get the optimization strategy that was actually selected
    pub fn selected_strategy(&self) -> OptimizationStrategy {
        self.processor.performance_info().strategy.clone()
    }

    /// Get the user's optimization preference
    pub fn optimization_preference(&self) -> &OptimizationPreference {
        &self.config.optimization_preference
    }

    /// Get CPU capabilities information
    pub fn cpu_info(&self) -> &CpuCapabilities {
        OptimizationSelector::get().cpu_capabilities()
    }

    /// Get processor name for diagnostics
    pub fn processor_name(&self) -> &str {
        self.processor.processor_name()
    }

    /// Reset processing statistics
    pub fn reset_stats(&mut self) {
        self.stats = ProcessingStats::default();
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total_ops = self.stats.successful_operations + self.stats.failed_operations;
        if total_ops > 0 {
            (self.stats.successful_operations as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get average processing rate in characters per second
    pub fn avg_processing_rate(&self) -> f64 {
        if self.stats.total_processing_time_ns > 0 {
            let total_time_secs = self.stats.total_processing_time_ns as f64 / 1_000_000_000.0;
            self.stats.total_chars_processed as f64 / total_time_secs
        } else {
            0.0
        }
    }

    /// Check if processor is performing optimally
    pub fn is_performing_optimally(&self) -> bool {
        let success_rate = self.success_rate();
        let fallback_rate = if self.stats.successful_operations + self.stats.failed_operations > 0 {
            (self.stats.fallback_operations as f64
                / (self.stats.successful_operations + self.stats.failed_operations) as f64)
                * 100.0
        } else {
            0.0
        };

        success_rate >= 95.0 && fallback_rate <= 5.0
    }
}

impl Default for VietnameseTextProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create default Vietnamese processor")
    }
}
