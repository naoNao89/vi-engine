//! Assembly Safety Infrastructure
//!
//! This module provides comprehensive safety mechanisms to ensure assembly code
//! stops gracefully when Rust stops, regardless of the reason (panic, signal,
//! timeout, early return, etc.).
//!
//! The safety system provides:
//! - Cooperative cancellation through atomic flags
//! - Signal handling for graceful shutdown
//! - Panic hook integration for emergency stops
//! - Timeout protection with watchdog monitoring
//! - Performance monitoring with minimal overhead

use once_cell::sync::Lazy;
use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Global assembly control instance for coordinating safety across all operations
pub static GLOBAL_ASSEMBLY_CONTROL: Lazy<Arc<AssemblyControl>> =
    Lazy::new(|| Arc::new(AssemblyControl::new()));

/// Assembly control structure for coordinating between Rust and assembly code
/// Cache-line aligned for optimal performance on Apple Silicon and `x86_64`
#[repr(C, align(64))]
pub struct AssemblyControl {
    /// Cooperative cancellation flag - checked by assembly loops
    pub cancel_flag: AtomicBool,
    /// Timeout flag - set when operation exceeds time limit
    pub timeout_flag: AtomicBool,
    /// Panic flag - set when Rust panic occurs
    pub panic_flag: AtomicBool,
    /// Maximum iterations allowed for assembly loops
    pub max_iterations: AtomicUsize,
    /// Current iteration count
    pub current_iteration: AtomicUsize,
    /// Heartbeat counter for progress monitoring
    pub heartbeat: AtomicU64,
    /// Operation start time (nanoseconds since epoch)
    pub start_time: AtomicU64,
    /// Timeout duration in milliseconds
    pub timeout_ms: AtomicU64,
}

impl AssemblyControl {
    /// Create new assembly control structure
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancel_flag: AtomicBool::new(false),
            timeout_flag: AtomicBool::new(false),
            panic_flag: AtomicBool::new(false),
            max_iterations: AtomicUsize::new(usize::MAX),
            current_iteration: AtomicUsize::new(0),
            heartbeat: AtomicU64::new(0),
            start_time: AtomicU64::new(0),
            timeout_ms: AtomicU64::new(5000), // 5 second default timeout
        }
    }

    /// Reset control structure for new operation
    pub fn reset_for_operation(&self, expected_size: usize) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        self.timeout_flag.store(false, Ordering::SeqCst);
        self.panic_flag.store(false, Ordering::SeqCst);
        self.current_iteration.store(0, Ordering::SeqCst);
        self.heartbeat.store(0, Ordering::SeqCst);

        // Set reasonable iteration limit based on input size
        let max_iters = expected_size.saturating_mul(10).max(1000);
        self.max_iterations.store(max_iters, Ordering::SeqCst);

        // Record start time
        let start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.start_time.store(start, Ordering::SeqCst);
    }

    /// Check if operation was cancelled
    #[must_use = "Cancellation status should be checked to handle operation state"]
    pub fn was_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Acquire)
            || self.timeout_flag.load(Ordering::Acquire)
            || self.panic_flag.load(Ordering::Acquire)
    }

    /// Check if operation timed out
    #[must_use = "Timeout status should be checked to handle operation state"]
    pub fn timed_out(&self) -> bool {
        self.timeout_flag.load(Ordering::Acquire)
    }

    /// Signal cancellation of all operations
    pub fn cancel_all(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// Update heartbeat (called from assembly)
    pub fn update_heartbeat(&self) {
        self.heartbeat.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if operation should continue (called from assembly safety checks)
    #[must_use = "Continue status must be checked to determine if operation should proceed"]
    pub fn should_continue(&self) -> bool {
        // Check cancellation flags first (fastest check)
        if self.was_cancelled() {
            return false;
        }

        // Check iteration limit
        let current = self.current_iteration.fetch_add(1, Ordering::Relaxed);
        if current >= self.max_iterations.load(Ordering::Relaxed) {
            self.cancel_flag.store(true, Ordering::SeqCst);
            return false;
        }

        // Check timeout (less frequent check for performance)
        if current % 1024 == 0 {
            let timeout_ms = self.timeout_ms.load(Ordering::Relaxed);
            if timeout_ms > 0 {
                let start = self.start_time.load(Ordering::Relaxed);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;

                if (now - start) / 1_000_000 > timeout_ms {
                    self.timeout_flag.store(true, Ordering::SeqCst);
                    return false;
                }
            }
        }

        true
    }
}

impl Default for AssemblyControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for assembly safety operations
///
/// This enum may be extended with additional error types in future versions.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AssemblyError {
    /// Operation was cancelled by user or system
    Cancelled,
    /// Operation exceeded timeout limit
    Timeout,
    /// Operation exceeded iteration limit
    IterationLimit,
    /// Panic occurred during operation
    Panic,
    /// Invalid input parameters
    InvalidInput,
    /// Assembly execution error
    ExecutionError(String),
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyError::Cancelled => write!(f, "Assembly operation was cancelled"),
            AssemblyError::Timeout => write!(f, "Assembly operation timed out"),
            AssemblyError::IterationLimit => {
                write!(f, "Assembly operation exceeded iteration limit")
            }
            AssemblyError::Panic => write!(f, "Panic occurred during assembly operation"),
            AssemblyError::InvalidInput => write!(f, "Invalid input parameters"),
            AssemblyError::ExecutionError(msg) => write!(f, "Assembly execution error: {msg}"),
        }
    }
}

impl std::error::Error for AssemblyError {}

/// Safety metrics for monitoring assembly operations
#[derive(Debug, Default)]
pub struct SafetyMetrics {
    /// Number of assembly operations that have been started
    pub operations_started: AtomicU64,
    /// Number of assembly operations that completed successfully
    pub operations_completed: AtomicU64,
    /// Number of assembly operations that were cancelled
    pub operations_cancelled: AtomicU64,
    /// Number of assembly operations that timed out
    pub operations_timed_out: AtomicU64,
    /// Total safety overhead in nanoseconds
    pub total_safety_overhead_ns: AtomicU64,
    /// Number of memory allocations tracked
    pub memory_allocations: AtomicU64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: AtomicU64,
}

impl SafetyMetrics {
    /// Creates a new safety metrics instance with all counters initialized to zero
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the start of an assembly operation
    pub fn record_start(&self) {
        self.operations_started.fetch_add(1, Ordering::Relaxed);
    }

    /// Records the successful completion of an assembly operation
    ///
    /// # Arguments
    /// * `overhead_ns` - Safety overhead in nanoseconds for this operation
    pub fn record_completion(&self, overhead_ns: u64) {
        self.operations_completed.fetch_add(1, Ordering::Relaxed);
        self.total_safety_overhead_ns
            .fetch_add(overhead_ns, Ordering::Relaxed);
    }

    /// Records the cancellation of an assembly operation
    pub fn record_cancellation(&self) {
        self.operations_cancelled.fetch_add(1, Ordering::Relaxed);
    }

    /// Records that an assembly operation timed out.
    pub fn record_timeout(&self) {
        self.operations_timed_out.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the success rate of assembly operations (completed/started).
    #[must_use = "Success rate should be used for monitoring or diagnostics"]
    pub fn get_success_rate(&self) -> f64 {
        let started = self.operations_started.load(Ordering::Relaxed);
        if started == 0 {
            return 1.0;
        }
        let completed = self.operations_completed.load(Ordering::Relaxed);
        {
            let completed_f64 = if completed > (1u64 << 53) {
                (1u64 << 53) as f64
            } else {
                completed as f64
            };
            let started_f64 = if started > (1u64 << 53) {
                (1u64 << 53) as f64
            } else {
                started as f64
            };
            completed_f64 / started_f64
        }
    }

    /// Gets the average safety overhead per operation in nanoseconds.
    #[must_use = "Average overhead should be used for performance analysis"]
    pub fn get_average_overhead_ns(&self) -> u64 {
        let completed = self.operations_completed.load(Ordering::Relaxed);
        if completed == 0 {
            return 0;
        }
        let total_overhead = self.total_safety_overhead_ns.load(Ordering::Relaxed);
        total_overhead / completed
    }

    /// Records a memory allocation of the specified size.
    pub fn record_memory_allocation(&self, size: u64) {
        self.memory_allocations.fetch_add(size, Ordering::Relaxed);
    }

    /// Updates the peak memory usage if the current usage is higher.
    pub fn update_peak_memory(&self, current_usage: u64) {
        let mut peak = self.peak_memory_usage.load(Ordering::Relaxed);
        while current_usage > peak {
            match self.peak_memory_usage.compare_exchange_weak(
                peak,
                current_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }

    /// Gets memory statistics as (`total_allocations`, `peak_usage`).
    #[must_use = "Memory statistics should be used for monitoring or diagnostics"]
    pub fn get_memory_stats(&self) -> (u64, u64) {
        (
            self.memory_allocations.load(Ordering::Relaxed),
            self.peak_memory_usage.load(Ordering::Relaxed),
        )
    }
}

/// Global safety metrics instance
pub static GLOBAL_SAFETY_METRICS: Lazy<SafetyMetrics> = Lazy::new(SafetyMetrics::new);

/// Global flag to ensure safety system is only initialized once
static SAFETY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Watchdog configuration for assembly monitoring
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    /// How often to check for stalls (milliseconds)
    pub check_interval_ms: u64,
    /// How long to wait for heartbeat updates before considering stalled (milliseconds)
    pub stall_timeout_ms: u64,
    /// Whether to enable the watchdog (can be disabled for performance)
    pub enabled: bool,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 100, // Check every 100ms
            stall_timeout_ms: 2000, // 2 second stall timeout
            enabled: !cfg!(test), // Disable watchdog in test environments to prevent thread exhaustion
        }
    }
}

/// Assembly watchdog for monitoring assembly operations
pub struct AssemblyWatchdog {
    #[allow(dead_code)]
    control: Arc<AssemblyControl>,
    config: WatchdogConfig,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AssemblyWatchdog {
    /// Create new watchdog with default configuration
    pub fn new(control: Arc<AssemblyControl>) -> Self {
        Self::with_config(control, WatchdogConfig::default())
    }

    /// Create new watchdog with custom configuration
    pub fn with_config(control: Arc<AssemblyControl>, config: WatchdogConfig) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let handle = if config.enabled {
            Some(Self::spawn_watchdog_thread(
                control.clone(),
                config.clone(),
                shutdown.clone(),
            ))
        } else {
            None
        };

        Self {
            control,
            config,
            shutdown,
            handle,
        }
    }

    /// Spawn the watchdog monitoring thread
    fn spawn_watchdog_thread(
        control: Arc<AssemblyControl>,
        config: WatchdogConfig,
        shutdown: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("assembly-watchdog".to_string())
            .spawn(move || {
                log::debug!("Assembly watchdog thread started");
                let mut last_heartbeat = 0u64;
                let mut last_heartbeat_time = Instant::now();

                while !shutdown.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(config.check_interval_ms));

                    // Check if there's an active operation
                    let start_time = control.start_time.load(Ordering::Relaxed);
                    if start_time == 0 {
                        // No operation in progress, reset tracking
                        last_heartbeat = 0;
                        last_heartbeat_time = Instant::now();
                        continue;
                    }

                    // Check for timeout (backup to main timeout mechanism)
                    let timeout_ms = control.timeout_ms.load(Ordering::Relaxed);
                    if timeout_ms > 0 {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        let elapsed_ms = (now - start_time) / 1_000_000;

                        if elapsed_ms > timeout_ms {
                            log::debug!("Watchdog detected timeout after {elapsed_ms}ms");
                            control.timeout_flag.store(true, Ordering::SeqCst);
                            control.cancel_flag.store(true, Ordering::SeqCst);
                            continue;
                        }
                    }

                    // Check for stalled operations (no heartbeat updates)
                    let current_heartbeat = control.heartbeat.load(Ordering::Relaxed);
                    let now = Instant::now();

                    if current_heartbeat == last_heartbeat {
                        // Check if we've been tracking this heartbeat long enough
                        if last_heartbeat > 0
                            && now.duration_since(last_heartbeat_time).as_millis() as u64
                                > config.stall_timeout_ms
                        {
                            // No heartbeat update for too long - operation may be stalled
                            log::debug!(
                                "Watchdog detected stalled operation (no heartbeat for {}ms)",
                                now.duration_since(last_heartbeat_time).as_millis()
                            );
                            control.cancel_flag.store(true, Ordering::SeqCst);

                            // Reset tracking to avoid repeated warnings
                            last_heartbeat_time = now;
                        } else if last_heartbeat == 0 {
                            // First time seeing this operation, start tracking
                            last_heartbeat = current_heartbeat;
                            last_heartbeat_time = now;
                        }
                    } else {
                        // Heartbeat updated, operation is making progress
                        last_heartbeat = current_heartbeat;
                        last_heartbeat_time = now;
                    }
                }

                log::debug!("Assembly watchdog thread stopped");
            })
            .unwrap_or_else(|e| {
                log::warn!("Failed to spawn watchdog thread: {e}. Continuing without watchdog.");
                // Return a dummy handle that immediately finishes
                thread::spawn(|| {})
            })
    }

    /// Shutdown the watchdog thread
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Check if watchdog is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get watchdog configuration
    #[must_use]
    pub fn config(&self) -> &WatchdogConfig {
        &self.config
    }
}

impl Drop for AssemblyWatchdog {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Initialize assembly safety system
/// This should be called once at program startup
pub fn initialize_assembly_safety() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure initialization only happens once
    if SAFETY_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // Already initialized
        return Ok(());
    }

    // In test environments, use minimal initialization to prevent thread exhaustion
    if cfg!(test) {
        setup_panic_hook();
        log::debug!("Assembly safety system initialized (test mode - no signal handling)");
    } else {
        setup_panic_hook();
        setup_signal_handling()?;
        log::info!("Assembly safety system initialized");
    }
    Ok(())
}

/// Setup panic hook to signal assembly operations to stop
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Signal all assembly operations to stop immediately
        GLOBAL_ASSEMBLY_CONTROL
            .panic_flag
            .store(true, Ordering::SeqCst);
        GLOBAL_ASSEMBLY_CONTROL
            .cancel_flag
            .store(true, Ordering::SeqCst);

        // Give assembly brief time to see flags
        std::thread::sleep(Duration::from_millis(10));

        // Call original panic hook
        original_hook(panic_info);
    }));
}

/// Setup signal handling for graceful shutdown
fn setup_signal_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Skip signal handling in test environments to prevent thread exhaustion
    if cfg!(test) {
        log::debug!("Skipping signal handling setup in test environment");
        return Ok(());
    }

    let mut signals = Signals::new([SIGINT, SIGTERM, SIGQUIT])?;

    // Use thread builder with error handling
    match std::thread::Builder::new()
        .name("signal-handler".to_string())
        .spawn(move || {
            loop {
                if let Some(sig) = signals.forever().next() {
                    match sig {
                        SIGINT | SIGTERM | SIGQUIT => {
                            log::warn!("Received signal {sig}, stopping assembly operations");
                            GLOBAL_ASSEMBLY_CONTROL
                                .cancel_flag
                                .store(true, Ordering::SeqCst);

                            // Give assembly time to stop gracefully
                            std::thread::sleep(Duration::from_millis(100));
                            std::process::exit(128 + sig);
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::warn!("Failed to spawn signal handler thread: {e}. Signal handling disabled.");
            Ok(()) // Don't fail initialization if signal handling can't be set up
        }
    }
}

/// Safe assembly processor that provides safety guarantees for assembly operations
pub struct SafeAssemblyProcessor {
    control: Arc<AssemblyControl>,
    watchdog: Option<AssemblyWatchdog>,
    metrics: Arc<SafetyMetrics>,
}

impl SafeAssemblyProcessor {
    /// Create new safe assembly processor with default watchdog
    pub fn new() -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());

        // In test environments, disable watchdog by default to prevent thread exhaustion
        let watchdog = if cfg!(test) {
            None
        } else {
            Some(AssemblyWatchdog::new(control.clone()))
        };

        Self {
            control,
            watchdog,
            metrics,
        }
    }

    /// Create safe processor with custom timeout
    #[must_use]
    pub fn with_timeout(timeout_ms: u64) -> Self {
        let processor = Self::new();
        processor
            .control
            .timeout_ms
            .store(timeout_ms, Ordering::SeqCst);
        processor
    }

    /// Create safe processor with custom watchdog configuration
    pub fn with_watchdog_config(config: WatchdogConfig) -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());
        let watchdog = Some(AssemblyWatchdog::with_config(control.clone(), config));

        Self {
            control,
            watchdog,
            metrics,
        }
    }

    /// Create safe processor without watchdog (for maximum performance)
    pub fn without_watchdog() -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());

        Self {
            control,
            watchdog: None,
            metrics,
        }
    }

    /// Process characters safely with comprehensive error handling
    pub fn process_chars_safe(&self, input: &[char]) -> Result<Vec<char>, AssemblyError> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        self.metrics.record_start();
        self.control.reset_for_operation(input.len());

        // Convert input to u32 for assembly interface
        let input_u32: Vec<u32> = input.iter().map(|&c| c as u32).collect();
        let mut output = vec![0u32; input.len()];

        // Process using safe assembly interface
        let processed = self.process_chars_internal(&input_u32, &mut output)?;

        // Check final state
        if self.control.was_cancelled() {
            self.metrics.record_cancellation();
            if self.control.timed_out() {
                return Err(AssemblyError::Timeout);
            }
            return Err(AssemblyError::Cancelled);
        }

        // Convert output back to chars
        let result: Vec<char> = output
            .into_iter()
            .take(processed)
            .filter_map(char::from_u32)
            .collect();

        let overhead_ns = start_time.elapsed().as_nanos() as u64;
        self.metrics.record_completion(overhead_ns);

        Ok(result)
    }

    /// Internal processing with safety checks using actual assembly
    fn process_chars_internal(
        &self,
        input: &[u32],
        output: &mut [u32],
    ) -> Result<usize, AssemblyError> {
        // Use actual assembly interface with safety integration
        let assembly_interface = crate::asm::direct_asm::get_assembly_interface();

        // Call assembly with safety control structure
        assembly_interface.process_chars_bulk_safe(input, output, &self.control)
    }

    /// Process single character with safety checks
    #[allow(dead_code)]
    fn process_char_safe(&self, ch: char) -> char {
        // Vietnamese character mapping with safety checks
        match ch {
            // Vietnamese lowercase vowels with diacritics
            'à' | 'á' | 'ả' | 'ã' | 'ạ' => 'a',
            'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' => 'a',
            'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
            'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' => 'o',
            'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' => 'o',
            'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'o',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' => 'u',
            'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',

            // Vietnamese uppercase vowels with diacritics
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' => 'A',
            'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' => 'A',
            'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' => 'E',
            'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' => 'O',
            'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' => 'O',
            'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' => 'U',
            'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
            'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',

            // Vietnamese consonants
            'đ' => 'd',
            'Đ' => 'D',

            // Extended Latin characters
            'ă' => 'a',
            'Ă' => 'A',
            'â' => 'a',
            'Â' => 'A',
            'ê' => 'e',
            'Ê' => 'E',
            'ô' => 'o',
            'Ô' => 'O',
            'ơ' => 'o',
            'Ơ' => 'O',
            'ư' => 'u',
            'Ư' => 'U',

            // Pass through all other characters unchanged
            _ => ch,
        }
    }

    /// Process string safely
    pub fn process_string_safe(&self, input: &str) -> Result<String, AssemblyError> {
        let chars: Vec<char> = input.chars().collect();
        let processed = self.process_chars_safe(&chars)?;
        Ok(processed.into_iter().collect())
    }

    /// Get safety metrics
    #[must_use]
    pub fn get_metrics(&self) -> &SafetyMetrics {
        &self.metrics
    }

    /// Cancel current operations
    pub fn cancel(&self) {
        self.control.cancel_all();
    }

    /// Check if watchdog is enabled
    #[must_use]
    pub fn has_watchdog(&self) -> bool {
        self.watchdog.is_some()
    }

    /// Get watchdog configuration if enabled
    #[must_use]
    pub fn watchdog_config(&self) -> Option<&WatchdogConfig> {
        self.watchdog.as_ref().map(AssemblyWatchdog::config)
    }
}

impl Default for SafeAssemblyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SafeAssemblyProcessor {
    fn drop(&mut self) {
        // Cancel any ongoing operations
        self.control.cancel_all();

        // Shutdown watchdog if present
        if let Some(mut watchdog) = self.watchdog.take() {
            watchdog.shutdown();
        }
    }
}
