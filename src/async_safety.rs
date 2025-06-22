//! Async-compatible interfaces for assembly safety
//!
//! This module provides async versions of the assembly safety interfaces,
//! integrating with tokio's async runtime and cancellation mechanisms.

#[cfg(feature = "async")]
use std::sync::Arc;
#[cfg(feature = "async")]
use std::time::Duration;
#[cfg(feature = "async")]
use tokio::sync::Notify;
#[cfg(feature = "async")]
use tokio::time::{sleep, timeout, Instant};

#[cfg(feature = "async")]
use crate::safety::{
    AssemblyControl, AssemblyError, SafetyMetrics, WatchdogConfig, GLOBAL_ASSEMBLY_CONTROL,
};

/// Async-compatible assembly processor
#[cfg(feature = "async")]
pub struct AsyncSafeAssemblyProcessor {
    control: Arc<AssemblyControl>,
    watchdog: Option<AsyncAssemblyWatchdog>,
    metrics: Arc<SafetyMetrics>,
    cancellation_notify: Arc<Notify>,
}

#[cfg(feature = "async")]
impl AsyncSafeAssemblyProcessor {
    /// Create new async safe assembly processor
    pub fn new() -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());
        let cancellation_notify = Arc::new(Notify::new());
        let watchdog = Some(AsyncAssemblyWatchdog::new(
            control.clone(),
            cancellation_notify.clone(),
        ));

        Self {
            control,
            watchdog,
            metrics,
            cancellation_notify,
        }
    }

    /// Create async processor with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        let processor = Self::new();
        processor
            .control
            .timeout_ms
            .store(timeout_ms, std::sync::atomic::Ordering::SeqCst);
        processor
    }

    /// Create async processor with custom watchdog configuration
    pub fn with_watchdog_config(config: WatchdogConfig) -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());
        let cancellation_notify = Arc::new(Notify::new());
        let watchdog = Some(AsyncAssemblyWatchdog::with_config(
            control.clone(),
            config,
            cancellation_notify.clone(),
        ));

        Self {
            control,
            watchdog,
            metrics,
            cancellation_notify,
        }
    }

    /// Create async processor without watchdog (for maximum performance)
    pub fn without_watchdog() -> Self {
        let control = GLOBAL_ASSEMBLY_CONTROL.clone();
        let metrics = Arc::new(SafetyMetrics::new());
        let cancellation_notify = Arc::new(Notify::new());

        Self {
            control,
            watchdog: None,
            metrics,
            cancellation_notify,
        }
    }

    /// Process string safely with async support
    pub async fn process_string_safe(&self, input: &str) -> Result<String, AssemblyError> {
        let chars: Vec<char> = input.chars().collect();
        let processed = self.process_chars_safe(&chars).await?;
        Ok(processed.into_iter().collect())
    }

    /// Process character array safely with async support
    pub async fn process_chars_safe(&self, input: &[char]) -> Result<Vec<char>, AssemblyError> {
        // Check for cancellation before starting
        if self.control.was_cancelled() {
            return Err(AssemblyError::Cancelled);
        }

        // Record operation start
        self.metrics
            .operations_started
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let start_time = Instant::now();

        // Setup operation with timeout
        let timeout_ms = self
            .control
            .timeout_ms
            .load(std::sync::atomic::Ordering::Relaxed);

        let operation = async {
            // Reset control for new operation
            self.control.reset_for_operation(input.len());

            // Process characters in chunks to allow for async cancellation
            let mut result = Vec::with_capacity(input.len());
            const CHUNK_SIZE: usize = 1024;

            for chunk in input.chunks(CHUNK_SIZE) {
                // Check for cancellation
                if self.control.was_cancelled() {
                    return Err(AssemblyError::Cancelled);
                }

                // Process chunk synchronously (assembly operations are inherently sync)
                for &ch in chunk {
                    // Simulate character processing (replace with actual assembly call)
                    let processed_char = self.process_char_internal(ch)?;
                    result.push(processed_char);
                }

                // Yield control to allow other async tasks to run
                tokio::task::yield_now().await;
            }

            Ok(result)
        };

        // Apply timeout if specified
        let result = if timeout_ms > 0 {
            match timeout(Duration::from_millis(timeout_ms), operation).await {
                Ok(result) => result,
                Err(_) => {
                    self.control
                        .timeout_flag
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                    self.control.cancel_all();
                    Err(AssemblyError::Timeout)
                }
            }
        } else {
            operation.await
        };

        // Record operation completion
        let duration = start_time.elapsed();
        match &result {
            Ok(_) => {
                self.metrics
                    .operations_completed
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(AssemblyError::Cancelled) => {
                self.metrics
                    .operations_cancelled
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(_) => {
                self.metrics
                    .operations_timed_out
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        // Update timing metrics
        self.metrics.total_safety_overhead_ns.fetch_add(
            duration.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        result
    }

    /// Process string with tokio cancellation token
    pub async fn process_string_with_cancellation(
        &self,
        input: &str,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<String, AssemblyError> {
        let operation = self.process_string_safe(input);
        let cancellation = cancellation_token.cancelled();

        tokio::select! {
            result = operation => result,
            _ = cancellation => {
                self.cancel().await;
                Err(AssemblyError::Cancelled)
            }
        }
    }

    /// Cancel current operations asynchronously
    pub async fn cancel(&self) {
        self.control.cancel_all();
        self.cancellation_notify.notify_waiters();
    }

    /// Get safety metrics
    pub fn get_metrics(&self) -> &SafetyMetrics {
        &self.metrics
    }

    /// Check if watchdog is enabled
    pub fn has_watchdog(&self) -> bool {
        self.watchdog.is_some()
    }

    /// Get watchdog configuration if enabled
    pub fn watchdog_config(&self) -> Option<&WatchdogConfig> {
        self.watchdog.as_ref().map(|w| w.config())
    }

    /// Internal character processing using actual Vietnamese processing
    fn process_char_internal(&self, ch: char) -> Result<char, AssemblyError> {
        // Check for cancellation
        if self.control.was_cancelled() {
            return Err(AssemblyError::Cancelled);
        }

        // Use the actual Vietnamese character processing function
        Ok(crate::util::clean_char(ch))
    }
}

#[cfg(feature = "async")]
impl Default for AsyncSafeAssemblyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Async-compatible assembly watchdog
#[cfg(feature = "async")]
pub struct AsyncAssemblyWatchdog {
    #[allow(dead_code)] // Used in watchdog_task
    control: Arc<AssemblyControl>,
    config: WatchdogConfig,
    #[allow(dead_code)] // Used in watchdog_task
    cancellation_notify: Arc<Notify>,
    shutdown_notify: Arc<Notify>,
}

#[cfg(feature = "async")]
impl AsyncAssemblyWatchdog {
    /// Create new async watchdog with default configuration
    pub fn new(control: Arc<AssemblyControl>, cancellation_notify: Arc<Notify>) -> Self {
        Self::with_config(control, WatchdogConfig::default(), cancellation_notify)
    }

    /// Create new async watchdog with custom configuration
    pub fn with_config(
        control: Arc<AssemblyControl>,
        config: WatchdogConfig,
        cancellation_notify: Arc<Notify>,
    ) -> Self {
        let shutdown_notify = Arc::new(Notify::new());

        if config.enabled {
            // Spawn async watchdog task
            let control_clone = control.clone();
            let config_clone = config.clone();
            let shutdown_clone = shutdown_notify.clone();
            let cancellation_clone = cancellation_notify.clone();

            tokio::spawn(async move {
                Self::watchdog_task(
                    control_clone,
                    config_clone,
                    shutdown_clone,
                    cancellation_clone,
                )
                .await;
            });
        }

        Self {
            control,
            config,
            cancellation_notify,
            shutdown_notify,
        }
    }

    /// Async watchdog monitoring task
    async fn watchdog_task(
        control: Arc<AssemblyControl>,
        config: WatchdogConfig,
        shutdown_notify: Arc<Notify>,
        cancellation_notify: Arc<Notify>,
    ) {
        let mut last_heartbeat = 0u64;
        let mut last_heartbeat_time = Instant::now();

        loop {
            tokio::select! {
                _ = shutdown_notify.notified() => {
                    log::debug!("Async watchdog shutting down");
                    break;
                }
                _ = sleep(Duration::from_millis(config.check_interval_ms)) => {
                    // Check if there's an active operation
                    let start_time = control.start_time.load(std::sync::atomic::Ordering::Relaxed);
                    if start_time == 0 {
                        // No operation in progress, reset tracking
                        last_heartbeat = 0;
                        last_heartbeat_time = Instant::now();
                        continue;
                    }

                    // Check for timeout
                    let timeout_ms = control.timeout_ms.load(std::sync::atomic::Ordering::Relaxed);
                    if timeout_ms > 0 {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        let elapsed_ms = (now - start_time) / 1_000_000;

                        if elapsed_ms > timeout_ms {
                            log::debug!("Async watchdog detected timeout after {}ms", elapsed_ms);
                            control.timeout_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                            control.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                            cancellation_notify.notify_waiters();
                            continue;
                        }
                    }

                    // Check for stalled operations
                    let current_heartbeat = control.heartbeat.load(std::sync::atomic::Ordering::Relaxed);
                    let now = Instant::now();

                    if current_heartbeat != last_heartbeat {
                        // Heartbeat updated, operation is making progress
                        last_heartbeat = current_heartbeat;
                        last_heartbeat_time = now;
                    } else if last_heartbeat > 0 && now.duration_since(last_heartbeat_time).as_millis() as u64 > config.stall_timeout_ms {
                        // No heartbeat update for too long - operation may be stalled
                        log::debug!(
                            "Async watchdog detected stalled operation (no heartbeat for {}ms)",
                            now.duration_since(last_heartbeat_time).as_millis()
                        );
                        control.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                        cancellation_notify.notify_waiters();

                        // Reset tracking to avoid repeated warnings
                        last_heartbeat_time = now;
                    } else if last_heartbeat == 0 {
                        // First time seeing this operation, start tracking
                        last_heartbeat = current_heartbeat;
                        last_heartbeat_time = now;
                    }
                }
            }
        }
    }

    /// Shutdown the async watchdog
    pub async fn shutdown(&self) {
        self.shutdown_notify.notify_waiters();
    }

    /// Get watchdog configuration
    pub fn config(&self) -> &WatchdogConfig {
        &self.config
    }
}
