# Assembly Safety API Reference

This document provides a comprehensive reference for the vi-rust assembly safety system API.

## Overview

The assembly safety system provides robust protection against runaway assembly code through cooperative cancellation, timeout protection, signal handling, and comprehensive error management.

## Core Types

### SafeAssemblyProcessor

The main interface for safe Vietnamese text processing.

```rust
pub struct SafeAssemblyProcessor {
    control: Arc<AssemblyControl>,
    watchdog_handle: Option<thread::JoinHandle<()>>,
    metrics: Arc<SafetyMetrics>,
}
```

#### Constructors

```rust
// Create with default settings (5 second timeout)
let processor = SafeAssemblyProcessor::new();

// Create with custom timeout
let processor = SafeAssemblyProcessor::with_timeout(1000); // 1 second
```

#### Processing Methods

```rust
// Process a string safely
fn process_string_safe(&self, input: &str) -> Result<String, AssemblyError>;

// Process character array safely
fn process_chars_safe(&self, input: &[char]) -> Result<Vec<char>, AssemblyError>;

// Get safety metrics
fn get_metrics(&self) -> &SafetyMetrics;

// Cancel current operations
fn cancel(&self);
```

#### Example Usage

```rust
use vi::safety::{SafeAssemblyProcessor, AssemblyError};

let processor = SafeAssemblyProcessor::with_timeout(2000);

match processor.process_string_safe("Tiếng Việt") {
    Ok(result) => println!("Processed: {}", result), // "Tieng Viet"
    Err(AssemblyError::Timeout) => println!("Operation timed out"),
    Err(AssemblyError::Cancelled) => println!("Operation was cancelled"),
    Err(e) => println!("Error: {}", e),
}
```

### AssemblyControl

Low-level control structure for assembly coordination.

```rust
#[repr(C, align(64))]
pub struct AssemblyControl {
    pub cancel_flag: AtomicBool,
    pub timeout_flag: AtomicBool,
    pub panic_flag: AtomicBool,
    pub max_iterations: AtomicUsize,
    pub current_iteration: AtomicUsize,
    pub heartbeat: AtomicU64,
    pub start_time: AtomicU64,
    pub timeout_ms: AtomicU64,
}
```

#### Key Methods

```rust
// Reset for new operation
fn reset_for_operation(&self, expected_size: usize);

// Check if operation should continue (called from assembly)
fn should_continue(&self) -> bool;

// Check cancellation status
fn was_cancelled(&self) -> bool;

// Signal cancellation
fn cancel_all(&self);

// Update progress (called from assembly)
fn update_heartbeat(&self);
```

#### Global Instance

```rust
use vi::safety::GLOBAL_ASSEMBLY_CONTROL;

// Access global control
let control = &*GLOBAL_ASSEMBLY_CONTROL;
control.cancel_all(); // Cancel all operations
```

### AssemblyError

Comprehensive error types for assembly operations.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AssemblyError {
    Cancelled,           // Operation was cancelled
    Timeout,            // Operation exceeded timeout
    IterationLimit,     // Exceeded iteration limit
    Panic,              // Panic occurred during operation
    InvalidInput,       // Invalid input parameters
    ExecutionError(String), // Assembly execution error
}
```

#### Error Handling

```rust
use vi::safety::{SafeAssemblyProcessor, AssemblyError};

let processor = SafeAssemblyProcessor::new();
let large_input = "à".repeat(1_000_000);

match processor.process_string_safe(&large_input) {
    Ok(result) => {
        println!("Successfully processed {} characters", result.len());
    }
    Err(AssemblyError::Timeout) => {
        println!("Processing timed out - input too large");
    }
    Err(AssemblyError::Cancelled) => {
        println!("Processing was cancelled by user or system");
    }
    Err(AssemblyError::IterationLimit) => {
        println!("Processing exceeded iteration limit");
    }
    Err(e) => {
        println!("Unexpected error: {}", e);
    }
}
```

### SafetyMetrics

Performance and safety monitoring.

```rust
pub struct SafetyMetrics {
    pub operations_started: AtomicU64,
    pub operations_completed: AtomicU64,
    pub operations_cancelled: AtomicU64,
    pub operations_timed_out: AtomicU64,
    pub total_safety_overhead_ns: AtomicU64,
}
```

#### Metrics Methods

```rust
// Get success rate (0.0 to 1.0)
fn get_success_rate(&self) -> f64;

// Get average safety overhead in nanoseconds
fn get_average_overhead_ns(&self) -> u64;

// Record operations (internal use)
fn record_start(&self);
fn record_completion(&self, overhead_ns: u64);
fn record_cancellation(&self);
fn record_timeout(&self);
```

#### Global Metrics

```rust
use vi::safety::GLOBAL_SAFETY_METRICS;

let metrics = &*GLOBAL_SAFETY_METRICS;
println!("Global success rate: {:.2}%", metrics.get_success_rate() * 100.0);
println!("Average overhead: {} ns", metrics.get_average_overhead_ns());
```

## Initialization

### System Initialization

```rust
use vi::safety::initialize_assembly_safety;

// Initialize safety system (call once at program startup)
initialize_assembly_safety()?;
```

This sets up:
- Panic hooks for automatic assembly cancellation
- Signal handlers for graceful shutdown
- Global control structures

### Signal Handling

The system automatically handles these signals:
- `SIGINT` (Ctrl+C)
- `SIGTERM` (termination request)
- `SIGQUIT` (quit signal)

When received, all assembly operations are cancelled gracefully.

## Advanced Usage

### Concurrent Processing

```rust
use std::sync::Arc;
use std::thread;
use vi::safety::SafeAssemblyProcessor;

let processor = Arc::new(SafeAssemblyProcessor::new());
let mut handles = vec![];

// Spawn multiple processing threads
for i in 0..4 {
    let processor_clone = processor.clone();
    let handle = thread::spawn(move || {
        let input = format!("Tiếng Việt {}", i).repeat(1000);
        processor_clone.process_string_safe(&input)
    });
    handles.push(handle);
}

// Cancel all operations after 100ms
thread::sleep(Duration::from_millis(100));
processor.cancel();

// Collect results
for handle in handles {
    match handle.join().unwrap() {
        Ok(result) => println!("Completed: {} chars", result.len()),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Custom Timeout Handling

```rust
use vi::safety::{SafeAssemblyProcessor, AssemblyError};
use std::time::Duration;

fn process_with_retry(input: &str, max_retries: usize) -> Result<String, AssemblyError> {
    for attempt in 0..max_retries {
        let timeout_ms = 1000 * (attempt + 1); // Increase timeout each retry
        let processor = SafeAssemblyProcessor::with_timeout(timeout_ms as u64);
        
        match processor.process_string_safe(input) {
            Ok(result) => return Ok(result),
            Err(AssemblyError::Timeout) if attempt < max_retries - 1 => {
                println!("Attempt {} timed out, retrying with longer timeout", attempt + 1);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(AssemblyError::Timeout)
}

// Usage
match process_with_retry("Very long Vietnamese text...", 3) {
    Ok(result) => println!("Success: {}", result),
    Err(e) => println!("Failed after retries: {}", e),
}
```

### Performance Monitoring

```rust
use vi::safety::SafeAssemblyProcessor;
use std::time::Instant;

let processor = SafeAssemblyProcessor::new();
let test_inputs = vec![
    "Short text",
    "Medium length Vietnamese: Tiếng Việt",
    "à".repeat(10000), // Large input
];

for input in test_inputs {
    let start = Instant::now();
    match processor.process_string_safe(input) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("Processed {} -> {} chars in {:?}", 
                     input.len(), result.len(), duration);
        }
        Err(e) => println!("Error processing input: {}", e),
    }
}

// Check metrics
let metrics = processor.get_metrics();
println!("Success rate: {:.2}%", metrics.get_success_rate() * 100.0);
println!("Average overhead: {} ns", metrics.get_average_overhead_ns());
```

## Safety Guarantees

The assembly safety system provides these guarantees:

1. **Bounded Execution**: No operation can run indefinitely
2. **Graceful Cancellation**: Operations can be cancelled cleanly
3. **Resource Cleanup**: Automatic cleanup on drop
4. **Thread Safety**: Safe concurrent access
5. **Signal Handling**: Graceful shutdown on system signals
6. **Panic Safety**: Assembly stops on Rust panics

## Performance Characteristics

- **Safety Overhead**: <1% performance impact
- **Cancellation Latency**: <10ms typical response time
- **Memory Usage**: Minimal additional allocation
- **Thread Overhead**: Lock-free atomic operations
- **Signal Latency**: <100ms graceful shutdown

## Cross-Platform Compatibility

The safety system works identically across:
- **aarch64-apple-darwin**: Native Mac ARM with assembly
- **x86_64-apple-darwin**: x86_64 simulation with Rust fallback
- **Future targets**: Generic ARM64 and x86_64 Linux

The API remains the same regardless of the underlying implementation.
