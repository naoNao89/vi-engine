# Assembly Safety Troubleshooting and Debugging Guide

This comprehensive guide helps diagnose and resolve issues with the vi-rust assembly safety system, covering common problems, debugging techniques, and performance troubleshooting.

## Quick Diagnostics

### Safety System Health Check

```bash
# Test basic safety functionality
cargo test --test safety_tests test_safe_processor_creation
cargo test --test safety_tests test_cooperative_cancellation

# Check safety metrics
cargo run --example safe_processing --release

# Verify signal handling
cargo test --test safety_tests test_signal_handling_integration
```

### Safety State Inspection

```rust
use vi::safety::{GLOBAL_ASSEMBLY_CONTROL, SafeAssemblyProcessor};

// Check global control state
let control = &*GLOBAL_ASSEMBLY_CONTROL;
println!("Cancel flag: {}", control.cancel_flag.load(std::sync::atomic::Ordering::Relaxed));
println!("Timeout flag: {}", control.timeout_flag.load(std::sync::atomic::Ordering::Relaxed));
println!("Panic flag: {}", control.panic_flag.load(std::sync::atomic::Ordering::Relaxed));
println!("Current iteration: {}", control.current_iteration.load(std::sync::atomic::Ordering::Relaxed));
println!("Max iterations: {}", control.max_iterations.load(std::sync::atomic::Ordering::Relaxed));

// Check processor metrics
let processor = SafeAssemblyProcessor::new();
let metrics = processor.get_metrics();
println!("Operations started: {}", metrics.operations_started.load(std::sync::atomic::Ordering::Relaxed));
println!("Operations completed: {}", metrics.operations_completed.load(std::sync::atomic::Ordering::Relaxed));
println!("Operations cancelled: {}", metrics.operations_cancelled.load(std::sync::atomic::Ordering::Relaxed));
```

## Common Issues and Solutions

### 1. Timeout Issues

#### Issue: Operations timing out unexpectedly
```
Error: Assembly operation timed out
```

**Diagnosis**:
```rust
// Check timeout configuration
let processor = SafeAssemblyProcessor::new();
let control = &*GLOBAL_ASSEMBLY_CONTROL;
let timeout_ms = control.timeout_ms.load(std::sync::atomic::Ordering::Relaxed);
println!("Current timeout: {} ms", timeout_ms);

// Test with longer timeout
let processor = SafeAssemblyProcessor::with_timeout(10000); // 10 seconds
let result = processor.process_string_safe("test input");
```

**Solutions**:
1. **Increase timeout for large inputs**:
   ```rust
   let processor = SafeAssemblyProcessor::with_timeout(30000); // 30 seconds
   ```

2. **Use adaptive timeout based on input size**:
   ```rust
   fn get_adaptive_timeout(input_len: usize) -> u64 {
       std::cmp::max(5000, input_len as u64 / 100) // Min 5s, +10ms per char
   }
   
   let timeout = get_adaptive_timeout(input.len());
   let processor = SafeAssemblyProcessor::with_timeout(timeout);
   ```

3. **Process in chunks for very large inputs**:
   ```rust
   fn process_large_input_chunked(input: &str, chunk_size: usize) -> Result<String, AssemblyError> {
       let processor = SafeAssemblyProcessor::with_timeout(5000);
       let mut result = String::new();
       
       for chunk in input.chars().collect::<Vec<_>>().chunks(chunk_size) {
           let chunk_str: String = chunk.iter().collect();
           let processed = processor.process_string_safe(&chunk_str)?;
           result.push_str(&processed);
       }
       
       Ok(result)
   }
   ```

#### Issue: Timeout protection not working
```
Operation should have timed out but didn't
```

**Diagnosis**:
```bash
# Check if timeout tests pass
cargo test --test safety_tests test_timeout_protection -- --nocapture

# Run timeout benchmark
cargo bench --bench safety_benchmark benchmark_timeout_overhead
```

**Solutions**:
1. **Verify timeout initialization**:
   ```rust
   use vi::safety::initialize_assembly_safety;
   
   // Ensure safety system is initialized
   initialize_assembly_safety()?;
   ```

2. **Check system clock**:
   ```rust
   // Verify system time is working correctly
   let start = std::time::SystemTime::now();
   std::thread::sleep(std::time::Duration::from_millis(100));
   let elapsed = start.elapsed().unwrap();
   println!("100ms sleep took: {:?}", elapsed);
   ```

### 2. Cancellation Issues

#### Issue: Operations not responding to cancellation
```
Operation continues after cancel() called
```

**Diagnosis**:
```rust
use std::sync::Arc;
use std::thread;
use std::time::Duration;

let processor = Arc::new(SafeAssemblyProcessor::new());
let processor_clone = processor.clone();

// Start long operation
let handle = thread::spawn(move || {
    let large_input = "à".repeat(1_000_000);
    processor_clone.process_string_safe(&large_input)
});

// Cancel after 100ms
thread::sleep(Duration::from_millis(100));
processor.cancel();

// Check if cancellation worked
match handle.join().unwrap() {
    Err(AssemblyError::Cancelled) => println!("Cancellation working correctly"),
    Ok(_) => println!("WARNING: Operation completed despite cancellation"),
    Err(e) => println!("Unexpected error: {:?}", e),
}
```

**Solutions**:
1. **Verify cancellation flag is being checked**:
   ```rust
   // Check cancellation frequency in assembly loops
   // Safety checks should occur every 1024 iterations
   let control = &*GLOBAL_ASSEMBLY_CONTROL;
   control.reset_for_operation(10000);
   
   for i in 0..10000 {
       if i % 1024 == 0 && !control.should_continue() {
           println!("Cancellation detected at iteration {}", i);
           break;
       }
   }
   ```

2. **Increase cancellation check frequency for testing**:
   ```rust
   // Temporarily reduce check interval for debugging
   // Note: This will impact performance
   for i in 0..input.len() {
       if i % 64 == 0 && !control.should_continue() { // Check every 64 instead of 1024
           break;
       }
       // Process character
   }
   ```

#### Issue: Global cancellation affecting other operations
```
Cancelling one operation stops all operations
```

**Diagnosis**:
```rust
// Check if global state is being shared inappropriately
let control = &*GLOBAL_ASSEMBLY_CONTROL;
println!("Global cancel flag: {}", control.was_cancelled());

// Test with multiple processors
let proc1 = SafeAssemblyProcessor::new();
let proc2 = SafeAssemblyProcessor::new();

proc1.cancel();
// proc2 should not be affected if isolation is working correctly
```

**Solutions**:
1. **Use separate processors for independent operations**:
   ```rust
   // Each processor should have independent cancellation
   let processor1 = SafeAssemblyProcessor::new();
   let processor2 = SafeAssemblyProcessor::new();
   
   // Cancelling processor1 should not affect processor2
   processor1.cancel();
   let result2 = processor2.process_string_safe("test"); // Should work
   ```

2. **Reset global state between operations**:
   ```rust
   let control = &*GLOBAL_ASSEMBLY_CONTROL;
   control.reset_for_operation(input.len()); // Clears cancellation flags
   ```

### 3. Signal Handling Issues

#### Issue: Signal handlers not working
```
SIGINT/SIGTERM not stopping assembly operations
```

**Diagnosis**:
```bash
# Test signal handling
cargo test --test safety_tests test_signal_handling_integration

# Manual signal test
cargo run --example safe_processing &
PID=$!
sleep 1
kill -INT $PID  # Should stop gracefully
```

**Solutions**:
1. **Verify signal handler initialization**:
   ```rust
   use vi::safety::initialize_assembly_safety;
   
   // This sets up signal handlers
   initialize_assembly_safety()?;
   
   // Verify handlers are installed
   println!("Signal handlers installed");
   ```

2. **Test signal delivery**:
   ```rust
   use signal_hook::{consts::*, iterator::Signals};
   
   // Test if signals are being received
   let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
   
   std::thread::spawn(move || {
       for sig in signals.forever() {
           println!("Received signal: {}", sig);
       }
   });
   ```

### 4. Performance Issues

#### Issue: Excessive safety overhead
```
Performance degraded significantly with safety enabled
```

**Diagnosis**:
```bash
# Benchmark safety overhead
cargo bench --bench safety_benchmark

# Compare safe vs unsafe performance
cargo bench --bench safety_benchmark benchmark_bulk_safety_overhead

# Profile safety checks
cargo bench --bench safety_benchmark benchmark_safety_check_overhead
```

**Solutions**:
1. **Optimize safety check frequency**:
   ```rust
   // Reduce check frequency for performance-critical code
   // Check every 4096 iterations instead of 1024
   if iteration % 4096 == 0 && !control.should_continue() {
       break;
   }
   ```

2. **Use release builds**:
   ```bash
   # Always use release builds for performance testing
   cargo build --release
   cargo bench --bench safety_benchmark
   ```

3. **Monitor safety metrics**:
   ```rust
   let metrics = processor.get_metrics();
   let overhead_ns = metrics.total_safety_overhead_ns.load(std::sync::atomic::Ordering::Relaxed);
   let operations = metrics.operations_completed.load(std::sync::atomic::Ordering::Relaxed);
   
   if operations > 0 {
       let avg_overhead = overhead_ns / operations;
       println!("Average safety overhead: {} ns per operation", avg_overhead);
   }
   ```

### 5. Memory and Resource Issues

#### Issue: Memory leaks in safety system
```
Memory usage increasing over time
```

**Diagnosis**:
```bash
# Run memory leak detection
cargo test --test safety_tests test_memory_cleanup

# Use valgrind (if available)
valgrind --leak-check=full cargo test --test safety_tests

# Monitor memory usage
cargo run --example safe_processing --release &
PID=$!
while kill -0 $PID 2>/dev/null; do
    ps -o pid,vsz,rss -p $PID
    sleep 1
done
```

**Solutions**:
1. **Verify proper cleanup**:
   ```rust
   {
       let processor = SafeAssemblyProcessor::new();
       // Use processor
   } // processor should be dropped here, cleaning up resources
   
   // Check that global state is clean
   let control = &*GLOBAL_ASSEMBLY_CONTROL;
   assert!(!control.was_cancelled());
   ```

2. **Monitor resource usage**:
   ```rust
   use std::sync::Arc;
   
   let processor = Arc::new(SafeAssemblyProcessor::new());
   println!("Reference count: {}", Arc::strong_count(&processor));
   
   // Ensure no circular references
   drop(processor);
   ```

## Advanced Debugging Techniques

### 1. Assembly Integration Debugging

```rust
// Debug assembly call interface
#[cfg(feature = "apple_silicon_assembly")]
unsafe {
    let input_u32: Vec<u32> = input.iter().map(|&c| c as u32).collect();
    let mut output_u32 = vec![0u32; input.len()];
    
    println!("Calling assembly with {} characters", input.len());
    let processed = apple_hybrid_clean_chars_bulk_safe(
        input_u32.as_ptr(),
        output_u32.as_mut_ptr(),
        input.len(),
        &*control as *const AssemblyControl,
    );
    println!("Assembly processed {} characters", processed);
}
```

### 2. Concurrent Operation Debugging

```rust
use std::sync::{Arc, Barrier};
use std::thread;

// Test concurrent safety
let processor = Arc::new(SafeAssemblyProcessor::new());
let barrier = Arc::new(Barrier::new(4));
let mut handles = vec![];

for i in 0..4 {
    let processor = processor.clone();
    let barrier = barrier.clone();
    
    let handle = thread::spawn(move || {
        barrier.wait(); // Synchronize start
        
        let input = format!("Thread {} input", i);
        let result = processor.process_string_safe(&input);
        println!("Thread {} result: {:?}", i, result);
        result
    });
    
    handles.push(handle);
}

// Collect results
for (i, handle) in handles.into_iter().enumerate() {
    match handle.join() {
        Ok(result) => println!("Thread {} completed: {:?}", i, result),
        Err(e) => println!("Thread {} panicked: {:?}", i, e),
    }
}
```

### 3. System-Level Debugging

```bash
# Monitor system resources
top -pid $(pgrep -f "cargo.*safety")

# Check file descriptors
lsof -p $(pgrep -f "cargo.*safety")

# Monitor system calls
dtruss -p $(pgrep -f "cargo.*safety") 2>&1 | grep -E "(read|write|signal)"

# Check CPU usage
iostat -c 1 5
```

## Debugging Tools and Utilities

### 1. Safety State Inspector

```rust
pub fn inspect_safety_state() {
    let control = &*GLOBAL_ASSEMBLY_CONTROL;
    
    println!("=== Assembly Safety State ===");
    println!("Cancel flag: {}", control.cancel_flag.load(std::sync::atomic::Ordering::Relaxed));
    println!("Timeout flag: {}", control.timeout_flag.load(std::sync::atomic::Ordering::Relaxed));
    println!("Panic flag: {}", control.panic_flag.load(std::sync::atomic::Ordering::Relaxed));
    println!("Current iteration: {}", control.current_iteration.load(std::sync::atomic::Ordering::Relaxed));
    println!("Max iterations: {}", control.max_iterations.load(std::sync::atomic::Ordering::Relaxed));
    println!("Heartbeat: {}", control.heartbeat.load(std::sync::atomic::Ordering::Relaxed));
    println!("Start time: {}", control.start_time.load(std::sync::atomic::Ordering::Relaxed));
    println!("Timeout (ms): {}", control.timeout_ms.load(std::sync::atomic::Ordering::Relaxed));
    println!("============================");
}
```

### 2. Performance Profiler

```rust
pub fn profile_safety_overhead(input: &str, iterations: usize) {
    use std::time::Instant;
    
    let processor = SafeAssemblyProcessor::new();
    let mut total_time = std::time::Duration::new(0, 0);
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = processor.process_string_safe(input);
        total_time += start.elapsed();
    }
    
    let avg_time = total_time / iterations as u32;
    println!("Average processing time: {:?}", avg_time);
    println!("Characters per second: {:.0}", input.len() as f64 / avg_time.as_secs_f64());
}
```

### 3. Stress Test Utility

```rust
pub fn stress_test_safety(duration_secs: u64) {
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    
    let processor = Arc::new(SafeAssemblyProcessor::new());
    let start_time = Instant::now();
    let mut handles = vec![];
    
    // Spawn multiple worker threads
    for i in 0..4 {
        let processor = processor.clone();
        let handle = thread::spawn(move || {
            let mut operations = 0;
            let input = format!("Stress test input {}", i);
            
            while start_time.elapsed() < Duration::from_secs(duration_secs) {
                match processor.process_string_safe(&input) {
                    Ok(_) => operations += 1,
                    Err(e) => println!("Thread {} error: {}", i, e),
                }
            }
            
            println!("Thread {} completed {} operations", i, operations);
            operations
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut total_operations = 0;
    for handle in handles {
        total_operations += handle.join().unwrap_or(0);
    }
    
    println!("Stress test completed: {} total operations in {} seconds", 
             total_operations, duration_secs);
    println!("Operations per second: {:.2}", total_operations as f64 / duration_secs as f64);
}
```

## Getting Help

### Diagnostic Information to Collect

When reporting assembly safety issues, include:

```bash
# System and build information
rustc -vV
cargo --version
uname -a

# Safety system tests
cargo test --test safety_tests 2>&1

# Performance benchmarks
cargo bench --bench safety_benchmark -- --test 2>&1

# Example output
cargo run --example safe_processing 2>&1

# Build configuration
cargo build -v 2>&1 | grep -E "(rustc-cfg|feature)"
```

### Log Collection

```bash
# Enable debug logging
RUST_LOG=debug cargo test --test safety_tests 2>&1 | tee safety_debug.log

# Collect safety metrics
cargo run --example safe_processing --release 2>&1 | tee safety_metrics.log

# Performance data
cargo bench --bench safety_benchmark 2>&1 | tee safety_performance.log
```

This troubleshooting guide provides comprehensive coverage of assembly safety system debugging. For additional help, refer to the [Assembly Safety API Reference](assembly-safety-api.md) and [Safety Usage Guidelines](safety-usage-guidelines.md).
