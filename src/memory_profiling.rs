//! Memory Profiling Infrastructure
//!
//! This module provides optional memory usage tracking and profiling capabilities
//! for debugging and performance analysis of Vietnamese text processing operations.

use once_cell::sync::Lazy;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Memory allocation statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current bytes in use
    pub current_usage: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Number of allocations
    pub allocation_count: usize,
    /// Number of deallocations
    pub deallocation_count: usize,
    /// Average allocation size
    pub average_allocation_size: usize,
    /// Timestamp of last update
    pub last_updated: u64,
}

impl MemoryStats {
    /// Create new empty memory statistics
    pub fn new() -> Self {
        Self {
            total_allocated: 0,
            total_deallocated: 0,
            current_usage: 0,
            peak_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            average_allocation_size: 0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Calculate memory efficiency (deallocated / allocated)
    pub fn efficiency(&self) -> f64 {
        if self.total_allocated == 0 {
            1.0
        } else {
            self.total_deallocated as f64 / self.total_allocated as f64
        }
    }

    /// Calculate fragmentation ratio (current / peak)
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.peak_usage == 0 {
            0.0
        } else {
            self.current_usage as f64 / self.peak_usage as f64
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe memory profiler
pub struct MemoryProfiler {
    total_allocated: AtomicUsize,
    total_deallocated: AtomicUsize,
    current_usage: AtomicUsize,
    peak_usage: AtomicUsize,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    last_updated: AtomicU64,
    enabled: AtomicBool,
}

impl MemoryProfiler {
    /// Create new memory profiler
    pub fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_deallocated: AtomicUsize::new(0),
            current_usage: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            last_updated: AtomicU64::new(0),
            enabled: AtomicBool::new(false),
        }
    }

    /// Enable memory profiling
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// Disable memory profiling
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Record an allocation
    pub fn record_allocation(&self, size: usize) {
        if !self.is_enabled() {
            return;
        }

        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        let current = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak usage if necessary
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        self.update_timestamp();
    }

    /// Record a deallocation
    pub fn record_deallocation(&self, size: usize) {
        if !self.is_enabled() {
            return;
        }

        self.total_deallocated.fetch_add(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);

        self.update_timestamp();
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);
        let allocation_count = self.allocation_count.load(Ordering::Relaxed);

        MemoryStats {
            total_allocated,
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            allocation_count,
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
            average_allocation_size: if allocation_count > 0 {
                total_allocated / allocation_count
            } else {
                0
            },
            last_updated: self.last_updated.load(Ordering::Relaxed),
        }
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.total_allocated.store(0, Ordering::SeqCst);
        self.total_deallocated.store(0, Ordering::SeqCst);
        self.current_usage.store(0, Ordering::SeqCst);
        self.peak_usage.store(0, Ordering::SeqCst);
        self.allocation_count.store(0, Ordering::SeqCst);
        self.deallocation_count.store(0, Ordering::SeqCst);
        self.update_timestamp();
    }

    /// Update timestamp
    fn update_timestamp(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_updated.store(now, Ordering::Relaxed);
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Global memory profiler instance
static GLOBAL_MEMORY_PROFILER: Lazy<MemoryProfiler> = Lazy::new(MemoryProfiler::new);

/// Custom allocator that tracks memory usage
pub struct ProfilingAllocator;

unsafe impl GlobalAlloc for ProfilingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            GLOBAL_MEMORY_PROFILER.record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GLOBAL_MEMORY_PROFILER.record_deallocation(layout.size());
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            // Record as deallocation of old size and allocation of new size
            GLOBAL_MEMORY_PROFILER.record_deallocation(layout.size());
            GLOBAL_MEMORY_PROFILER.record_allocation(new_size);
        }
        new_ptr
    }
}

/// Memory profiling utilities
pub struct MemoryProfilerUtils;

impl MemoryProfilerUtils {
    /// Enable global memory profiling
    pub fn enable_profiling() {
        GLOBAL_MEMORY_PROFILER.enable();
    }

    /// Disable global memory profiling
    pub fn disable_profiling() {
        GLOBAL_MEMORY_PROFILER.disable();
    }

    /// Check if profiling is enabled
    pub fn is_profiling_enabled() -> bool {
        GLOBAL_MEMORY_PROFILER.is_enabled()
    }

    /// Get current memory statistics
    pub fn get_memory_stats() -> MemoryStats {
        GLOBAL_MEMORY_PROFILER.get_stats()
    }

    /// Reset memory statistics
    pub fn reset_memory_stats() {
        GLOBAL_MEMORY_PROFILER.reset();
    }

    /// Format memory statistics for display
    pub fn format_memory_stats(stats: &MemoryStats) -> String {
        format!(
            "Memory Stats:\n\
             - Total Allocated: {} bytes\n\
             - Total Deallocated: {} bytes\n\
             - Current Usage: {} bytes\n\
             - Peak Usage: {} bytes\n\
             - Allocations: {}\n\
             - Deallocations: {}\n\
             - Average Allocation: {} bytes\n\
             - Efficiency: {:.2}%\n\
             - Fragmentation: {:.2}%",
            stats.total_allocated,
            stats.total_deallocated,
            stats.current_usage,
            stats.peak_usage,
            stats.allocation_count,
            stats.deallocation_count,
            stats.average_allocation_size,
            stats.efficiency() * 100.0,
            stats.fragmentation_ratio() * 100.0
        )
    }

    /// Run a closure with memory profiling enabled
    pub fn profile_memory<F, R>(f: F) -> (R, MemoryStats)
    where
        F: FnOnce() -> R,
    {
        // Reset and enable profiling
        Self::reset_memory_stats();
        Self::enable_profiling();

        // Run the function
        let result = f();

        // Get stats and disable profiling
        let stats = Self::get_memory_stats();
        Self::disable_profiling();

        (result, stats)
    }
}

/// Scoped memory profiler for automatic cleanup
pub struct ScopedMemoryProfiler {
    was_enabled: bool,
}

impl Default for ScopedMemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopedMemoryProfiler {
    /// Create new scoped profiler and enable profiling
    pub fn new() -> Self {
        let was_enabled = MemoryProfilerUtils::is_profiling_enabled();
        MemoryProfilerUtils::enable_profiling();
        Self { was_enabled }
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryProfilerUtils::get_memory_stats()
    }
}

impl Drop for ScopedMemoryProfiler {
    fn drop(&mut self) {
        if !self.was_enabled {
            MemoryProfilerUtils::disable_profiling();
        }
    }
}
