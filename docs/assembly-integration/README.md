# Assembly Code Integration Documentation

This comprehensive guide covers the assembly code integration in the vi-rust Vietnamese Input Method Engine (IME), including architecture, safety model, lifecycle management, error handling, and platform-specific considerations.

## Table of Contents

1. [Assembly-Rust Integration Architecture](#assembly-rust-integration-architecture)
2. [Isolation and Safety Model](#isolation-and-safety-model)
3. [Assembly Function Lifecycle](#assembly-function-lifecycle)
4. [Error Handling and Recovery](#error-handling-and-recovery)
5. [Platform-Specific Considerations](#platform-specific-considerations)
6. [Code Examples and Best Practices](#code-examples-and-best-practices)
7. [Troubleshooting Guide](#troubleshooting-guide)

## Assembly-Rust Integration Architecture

### Overview

The vi-rust project integrates hand-optimized assembly code with Rust for maximum performance Vietnamese character processing. The integration uses a sophisticated build system and safety layer to maintain Rust's memory safety guarantees while achieving >11M characters/sec processing speed.

### Build System Architecture

The integration is managed through `build.rs` using the `cc` crate:

```rust
// build.rs - Assembly compilation logic
fn compile_aarch64_assembly() {
    let asm_file = "src/asm/aarch64_kernels.s";
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_cross_compile = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default()
        != env::var("HOST").unwrap_or_default().split('-').next().unwrap_or("");

    if Path::new(asm_file).exists() {
        let mut build = cc::Build::new();
        build.file(asm_file)
             .flag("-x")
             .flag("assembler")
             .flag("-mcpu=native")
             .flag("-w");
        
        build.compile("aarch64_kernels");
        println!("cargo:rustc-link-lib=static=aarch64_kernels");
        println!("cargo:rustc-cfg=feature=\"aarch64_assembly\"");
    }
}
```

### Assembly File Structure

The project contains platform-specific assembly implementations:

```
src/asm/
├── aarch64_apple_silicon.s     # Apple Silicon optimized (M1/M2/M3)
├── aarch64_kernels.s          # Generic ARM64 implementation
├── aarch64_generic.s          # Portable ARM64 for cross-compilation
└── x86_64_kernels.s           # x86_64 with BMI2/AVX-512 support
```

Note: The assembly functions are compiled and linked by `build.rs` but accessed through the existing safety infrastructure rather than a dedicated `mod.rs` interface.

### Feature Flag System

The build system automatically sets feature flags based on compilation target:

```rust
// Conditional compilation based on available assembly
#[cfg(feature = "apple_silicon_assembly")]
extern "C" {
    fn _apple_hybrid_clean_char_optimized(ch: u32) -> u32;
    fn _apple_hybrid_clean_chars_bulk_neon_optimized(
        input: *const u32, 
        output: *mut u32, 
        len: usize
    ) -> usize;
}

#[cfg(feature = "aarch64_assembly")]
extern "C" {
    fn generic_clean_char_aarch64(ch: u32) -> u32;
    fn generic_clean_chars_bulk_aarch64(
        input: *const u32, 
        output: *mut u32, 
        len: usize
    ) -> usize;
}

#[cfg(feature = "x86_64_assembly")]
extern "C" {
    fn hybrid_clean_char_x86_64(ch: u32) -> u32;
    fn hybrid_clean_chars_bulk_avx512(
        input: *const u32, 
        output: *mut u32, 
        len: usize
    ) -> usize;
}

#[cfg(feature = "no_assembly")]
fn clean_char_rust(ch: char) -> char {
    // Safe Rust fallback implementation
    crate::processor::clean_char_rust_impl(ch)
}
```

### Calling Convention Interface

All assembly functions follow a consistent C-compatible calling convention:

**Single Character Processing:**
- Input: `u32` (Unicode codepoint)
- Output: `u32` (cleaned Unicode codepoint)
- Registers: Platform-specific (w0/eax for input/output)

**Bulk Processing:**
- Input: `*const u32` (input array), `*mut u32` (output array), `usize` (length)
- Output: `usize` (number of characters processed)
- Memory: Caller-allocated buffers, no dynamic allocation in assembly

### Memory Layout and Alignment

Assembly functions expect specific memory alignment for optimal performance:

```rust
// Cache-line aligned control structure for assembly coordination
#[repr(C, align(64))]
pub struct AssemblyControl {
    pub cancel_flag: AtomicBool,        // Cooperative cancellation
    pub timeout_flag: AtomicBool,       // Timeout detection
    pub panic_flag: AtomicBool,         // Rust panic notification
    pub max_iterations: AtomicUsize,    // Iteration limit
    pub current_iteration: AtomicUsize, // Progress tracking
    pub heartbeat: AtomicU64,           // Liveness indicator
    pub start_time: AtomicU64,          // Operation start time
    pub timeout_ms: AtomicU64,          // Timeout duration
}
```

## Isolation and Safety Model

### Safety Boundaries

The assembly integration maintains strict safety boundaries:

1. **Memory Safety**: Assembly code operates only on caller-provided buffers
2. **Type Safety**: All data exchange uses well-defined C-compatible types
3. **Lifetime Safety**: Assembly functions are stateless and don't retain references
4. **Thread Safety**: Assembly operations are isolated per thread

### Cooperative Cancellation System

Assembly code integrates with Rust's safety system through cooperative cancellation:

```assembly
// ARM64 assembly with safety checks
_apple_hybrid_clean_chars_bulk_neon_optimized:
    // ... setup code ...
    
.loop_start:
    // Check cancellation every 1024 iterations
    and x10, x9, #0x3FF          // x9 & 1023
    cbnz x10, .skip_safety_check
    
    // Load global cancellation flag
    adrp x11, _GLOBAL_ASSEMBLY_CONTROL@PAGE
    add x11, x11, _GLOBAL_ASSEMBLY_CONTROL@PAGEOFF
    ldrb w12, [x11]              // Load cancel_flag
    cbnz w12, .operation_cancelled
    
.skip_safety_check:
    // ... processing logic ...
    add x9, x9, #1               // Increment iteration counter
    cmp x9, x2                   // Compare with total length
    b.lt .loop_start
    
    // Normal completion
    mov x0, x9                   // Return processed count
    ret
    
.operation_cancelled:
    mov x0, x9                   // Return partial count
    ret
```

### Panic Integration

When Rust code panics, assembly operations are immediately cancelled:

```rust
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Signal all assembly operations to stop immediately
        GLOBAL_ASSEMBLY_CONTROL.panic_flag.store(true, Ordering::SeqCst);
        GLOBAL_ASSEMBLY_CONTROL.cancel_flag.store(true, Ordering::SeqCst);
        
        // Give assembly brief time to see flags
        std::thread::sleep(Duration::from_millis(10));
        
        // Call original panic hook
        original_hook(panic_info);
    }));
}
```

### Stack Unwinding Implications

Assembly functions are designed to be unwind-safe:

1. **No Destructors**: Assembly code doesn't manage Rust objects with destructors
2. **No Allocations**: Assembly functions don't perform dynamic memory allocation
3. **Stateless Design**: No persistent state that could be corrupted during unwinding
4. **Early Return**: Assembly functions can return early without cleanup requirements

## Assembly Function Lifecycle

### Initialization Phase

Assembly functions are stateless but rely on global control structures:

```rust
// Initialize assembly safety system
pub fn initialize_assembly_safety() -> Result<(), Box<dyn std::error::Error>> {
    // Setup panic hook for automatic cancellation
    setup_panic_hook();
    
    // Setup signal handling for graceful shutdown
    setup_signal_handling()?;
    
    // Initialize global control structure
    GLOBAL_ASSEMBLY_CONTROL.reset_for_operation(0);
    
    Ok(())
}
```

### Execution Phase

During execution, assembly functions coordinate with Rust through shared memory:

```rust
impl SafeAssemblyProcessor {
    pub fn process_chars_safe(&self, input: &[char]) -> Result<Vec<char>, AssemblyError> {
        // Reset control structure for new operation
        self.control.reset_for_operation(input.len());
        
        // Convert to assembly-compatible format
        let input_u32: Vec<u32> = input.iter().map(|&c| c as u32).collect();
        let mut output = vec![0u32; input.len()];
        
        // Call assembly function with safety wrapper
        let processed = unsafe {
            #[cfg(feature = "apple_silicon_assembly")]
            {
                _apple_hybrid_clean_chars_bulk_neon_optimized(
                    input_u32.as_ptr(),
                    output.as_mut_ptr(),
                    input_u32.len()
                )
            }
            #[cfg(not(feature = "apple_silicon_assembly"))]
            {
                // Fallback to Rust implementation
                self.process_chars_rust(&input_u32, &mut output)?
            }
        };
        
        // Check for cancellation or timeout
        if self.control.was_cancelled() {
            return Err(AssemblyError::Cancelled);
        }
        
        // Convert back to Rust types
        let result: Vec<char> = output.into_iter()
            .take(processed)
            .filter_map(|c| char::from_u32(c))
            .collect();
            
        Ok(result)
    }
}
```

### Cleanup and Termination Phase

Assembly functions handle termination gracefully:

```rust
impl Drop for SafeAssemblyProcessor {
    fn drop(&mut self) {
        // Cancel any ongoing operations
        self.control.cancel_all();

        // Wait for watchdog thread to finish
        if let Some(handle) = self.watchdog_handle.take() {
            let _ = handle.join();
        }
    }
}
```

### Resource Management

Assembly functions follow strict resource management principles:

1. **No Dynamic Allocation**: Assembly code never calls malloc/free
2. **Stack-Only Data**: All temporary data uses stack allocation
3. **Caller-Owned Buffers**: Input/output buffers are managed by Rust
4. **No File Handles**: Assembly code doesn't open files or network connections

## Error Handling and Recovery

### Error Types and Classification

The safety system defines comprehensive error types:

```rust
#[derive(Debug, Clone, PartialEq)]
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
            AssemblyError::IterationLimit => write!(f, "Assembly operation exceeded iteration limit"),
            AssemblyError::Panic => write!(f, "Panic occurred during assembly operation"),
            AssemblyError::InvalidInput => write!(f, "Invalid input parameters"),
            AssemblyError::ExecutionError(msg) => write!(f, "Assembly execution error: {}", msg),
        }
    }
}
```

### Assembly Exception Handling

Assembly code includes built-in error detection:

```assembly
// x86_64 assembly with bounds checking
hybrid_clean_chars_bulk_avx512:
    // ... setup code ...

    // Validate input parameters
    test rdi, rdi                    // Check input pointer
    jz .invalid_input_error
    test rsi, rsi                    // Check output pointer
    jz .invalid_input_error
    test rdx, rdx                    // Check length
    jz .empty_input_ok

    // Check for reasonable bounds (prevent DoS)
    cmp rdx, 0x10000000             // 256MB limit
    ja .input_too_large_error

    // ... processing loop ...

.invalid_input_error:
    mov rax, -1                      // Error code
    ret

.input_too_large_error:
    mov rax, -2                      // Error code
    ret

.empty_input_ok:
    xor rax, rax                     // Return 0 processed
    ret
```

### Recovery Strategies

The system implements multiple recovery strategies:

```rust
impl SafeAssemblyProcessor {
    /// Process with automatic retry on recoverable errors
    pub fn process_with_retry(&self, input: &str, max_retries: u32) -> Result<String, AssemblyError> {
        let mut attempts = 0;

        loop {
            match self.process_string_safe(input) {
                Ok(result) => return Ok(result),
                Err(AssemblyError::Timeout) if attempts < max_retries => {
                    attempts += 1;
                    // Increase timeout for retry
                    let new_timeout = self.control.timeout_ms.load(Ordering::SeqCst) * 2;
                    self.control.timeout_ms.store(new_timeout, Ordering::SeqCst);
                    continue;
                }
                Err(AssemblyError::IterationLimit) if attempts < max_retries => {
                    attempts += 1;
                    // Increase iteration limit for retry
                    let new_limit = self.control.max_iterations.load(Ordering::SeqCst) * 2;
                    self.control.max_iterations.store(new_limit, Ordering::SeqCst);
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Fallback to pure Rust implementation on assembly failure
    pub fn process_with_fallback(&self, input: &str) -> Result<String, AssemblyError> {
        match self.process_string_safe(input) {
            Ok(result) => Ok(result),
            Err(AssemblyError::ExecutionError(_)) => {
                // Fall back to Rust implementation
                let chars: Vec<char> = input.chars().collect();
                let result: String = chars.into_iter()
                    .map(|c| self.process_char_rust_fallback(c))
                    .collect();
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }
}
```

### Debugging Assembly Integration

The system provides comprehensive debugging support:

```rust
// Enable assembly debugging
#[cfg(debug_assertions)]
pub fn enable_assembly_debugging() {
    GLOBAL_ASSEMBLY_CONTROL.debug_mode.store(true, Ordering::SeqCst);
}

// Debug information collection
pub struct AssemblyDebugInfo {
    pub function_name: String,
    pub input_size: usize,
    pub processing_time_ns: u64,
    pub iterations_completed: usize,
    pub cancellation_reason: Option<String>,
    pub platform_info: PlatformInfo,
}

impl SafeAssemblyProcessor {
    pub fn get_debug_info(&self) -> AssemblyDebugInfo {
        AssemblyDebugInfo {
            function_name: self.get_active_function_name(),
            input_size: self.control.max_iterations.load(Ordering::SeqCst),
            processing_time_ns: self.get_elapsed_time_ns(),
            iterations_completed: self.control.current_iteration.load(Ordering::SeqCst),
            cancellation_reason: self.get_cancellation_reason(),
            platform_info: PlatformInfo::current(),
        }
    }
}
```

## Platform-Specific Considerations

### ARM64 vs x86_64 Differences

The integration handles significant architectural differences:

| Aspect | ARM64 | x86_64 |
|--------|-------|--------|
| **Calling Convention** | AAPCS64 (w0-w7 for args) | System V AMD64 (rdi, rsi, rdx, rcx) |
| **Register Usage** | 31 general-purpose (x0-x30) | 16 general-purpose (rax-r15) |
| **SIMD Instructions** | NEON (128-bit vectors) | AVX-512 (512-bit vectors) |
| **Memory Ordering** | Weak ordering, explicit barriers | Strong ordering, fewer barriers |
| **Cache Line Size** | 64 bytes (Apple Silicon) | 64 bytes (typical) |
| **Branch Prediction** | Advanced (Apple Silicon) | Complex (Intel/AMD) |

### Apple Silicon Specific Optimizations

Apple Silicon assembly includes specific optimizations:

```assembly
// Apple Silicon optimized character processing
_apple_hybrid_clean_char_neon:
    // Leverage unified memory architecture
    prfm pldl1keep, [x0, #64]       // Prefetch next cache line

    // Use Apple Silicon specific NEON optimizations
    dup v0.4s, w0                    // Duplicate input across vector

    // Apple Silicon has excellent branch prediction
    cmp w0, #0x1EA0                  // Vietnamese range start
    b.lo .ascii_fast_path
    cmp w0, #0x1EF9                  // Vietnamese range end
    b.hi .not_vietnamese

    // Optimized lookup using Apple Silicon cache hierarchy
    adrp x1, vietnamese_lookup_table@PAGE
    add x1, x1, vietnamese_lookup_table@PAGEOFF

    // Use post-indexed addressing (efficient on Apple Silicon)
    sub w2, w0, #0x1EA0              // Normalize to table index
    ldr w0, [x1, w2, lsl #2]         // Load replacement character

    ret
```

### x86_64 Specific Features

x86_64 assembly leverages advanced instruction sets:

```assembly
# x86_64 with BMI2 and AVX-512 support
hybrid_clean_chars_bulk_avx512:
    # Check for AVX-512 support
    mov eax, 7
    xor ecx, ecx
    cpuid
    test ebx, (1 << 16)              # AVX-512F support
    jz .fallback_to_sse

    # Use 512-bit vectors for bulk processing
    vmovdqu32 zmm0, [rdi]            # Load 16 characters at once

    # Parallel Vietnamese character detection
    vpcmpd k1, zmm0, zmm_viet_min, 5 # Compare >= Vietnamese range start
    vpcmpd k2, zmm0, zmm_viet_max, 2 # Compare <= Vietnamese range end
    kandw k3, k1, k2                 # Combine masks

    # Vectorized table lookup
    vpgatherdd zmm1{k3}, [vietnamese_table + zmm0*4]

    # Blend original and processed characters
    vpblendmd zmm0{k3}, zmm0, zmm1

    # Store results
    vmovdqu32 [rsi], zmm0

    add rdi, 64                      # Advance input pointer
    add rsi, 64                      # Advance output pointer
    sub rdx, 16                      # Decrease remaining count
    jnz .avx512_loop

    ret
```

### Cross-Compilation Implications

The build system handles cross-compilation automatically:

```rust
// build.rs - Cross-compilation detection
fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let host_arch = env::var("HOST").unwrap().split('-').next().unwrap();
    let is_cross_compile = target_arch != host_arch;

    if is_cross_compile {
        println!("cargo:warning=Assembly compilation disabled for cross-compilation");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
        return;
    }

    // Proceed with native assembly compilation
    match target_arch.as_str() {
        "aarch64" => compile_aarch64_assembly(),
        "x86_64" => compile_x86_64_assembly(),
        _ => println!("cargo:rustc-cfg=feature=\"no_assembly\""),
    }
}
```

### ABI Compatibility Matrix

| Source Platform | Target Platform | Assembly Support | Fallback |
|----------------|-----------------|------------------|----------|
| Apple Silicon | Apple Silicon | ✅ Native optimized | N/A |
| Apple Silicon | x86_64 Mac | ❌ Cross-compile | Rust impl |
| Apple Silicon | Linux ARM64 | ❌ Cross-compile | Rust impl |
| Apple Silicon | Linux x86_64 | ❌ Cross-compile | Rust impl |
| Intel Mac | Apple Silicon | ❌ Cross-compile | Rust impl |
| Intel Mac | Intel Mac | ✅ Native optimized | N/A |
| Linux x86_64 | Linux x86_64 | ✅ Native optimized | N/A |
| Linux ARM64 | Linux ARM64 | ✅ Generic ARM64 | N/A |

## Code Examples and Best Practices

### Basic Usage Pattern

```rust
use vi_rust::safety::{SafeAssemblyProcessor, AssemblyError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the assembly safety system
    vi::safety::initialize_assembly_safety()?;

    // Create a processor with 5-second timeout
    let processor = SafeAssemblyProcessor::new();

    // Process Vietnamese text
    let input = "Tiếng Việt rất đẹp và phong phú";
    match processor.process_string_safe(input) {
        Ok(result) => {
            println!("Original: {}", input);
            println!("Cleaned:  {}", result);
            // Output: "Tieng Viet rat dep va phong phu"
        }
        Err(AssemblyError::Timeout) => {
            eprintln!("Processing timed out - input too large");
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
        }
    }

    Ok(())
}
```

### High-Performance Bulk Processing

```rust
use vi_rust::safety::{SafeAssemblyProcessor, AssemblyError};
use std::time::Instant;

fn process_large_text(text: &str) -> Result<String, AssemblyError> {
    // Use longer timeout for large inputs
    let processor = SafeAssemblyProcessor::with_timeout(30_000); // 30 seconds

    let start = Instant::now();
    let result = processor.process_string_safe(text)?;
    let duration = start.elapsed();

    println!("Processed {} chars in {:?} ({:.2} M chars/sec)",
             text.len(),
             duration,
             text.len() as f64 / duration.as_secs_f64() / 1_000_000.0);

    Ok(result)
}

// Example usage with large text
fn main() -> Result<(), Box<dyn std::error::Error>> {
    vi_rust::safety::initialize_assembly_safety()?;

    // Generate large test input
    let large_input = "Xin chào thế giới! ".repeat(100_000); // ~1.8M characters

    match process_large_text(&large_input) {
        Ok(result) => println!("Successfully processed {} characters", result.len()),
        Err(e) => eprintln!("Failed to process large text: {}", e),
    }

    Ok(())
}
```

### Error Handling with Retry Logic

```rust
use vi_rust::safety::{SafeAssemblyProcessor, AssemblyError};
use std::time::Duration;
use std::thread;

fn robust_processing(input: &str) -> Result<String, AssemblyError> {
    let processor = SafeAssemblyProcessor::new();
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        match processor.process_string_safe(input) {
            Ok(result) => return Ok(result),
            Err(AssemblyError::Timeout) if attempt < max_retries => {
                attempt += 1;
                println!("Attempt {} timed out, retrying with longer timeout...", attempt);

                // Exponential backoff
                let new_timeout = 5000 * (1 << attempt); // 5s, 10s, 20s
                let retry_processor = SafeAssemblyProcessor::with_timeout(new_timeout);

                match retry_processor.process_string_safe(input) {
                    Ok(result) => return Ok(result),
                    Err(e) if attempt == max_retries => return Err(e),
                    Err(_) => continue,
                }
            }
            Err(AssemblyError::ExecutionError(_)) => {
                // Fall back to Rust implementation
                println!("Assembly failed, falling back to Rust implementation");
                return Ok(fallback_rust_processing(input));
            }
            Err(e) => return Err(e),
        }
    }
}

fn fallback_rust_processing(input: &str) -> String {
    // Pure Rust implementation as fallback
    input.chars()
        .map(|c| vi_rust::processor::clean_char_rust_impl(c))
        .collect()
}
```

### Concurrent Processing Pattern

```rust
use vi_rust::safety::{SafeAssemblyProcessor, AssemblyError};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc;

fn concurrent_processing(inputs: Vec<String>) -> Result<Vec<String>, AssemblyError> {
    let processor = Arc::new(SafeAssemblyProcessor::new());
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Spawn worker threads
    for (i, input) in inputs.into_iter().enumerate() {
        let processor_clone = Arc::clone(&processor);
        let tx_clone = tx.clone();

        let handle = thread::spawn(move || {
            let result = processor_clone.process_string_safe(&input);
            tx_clone.send((i, result)).unwrap();
        });

        handles.push(handle);
    }

    drop(tx); // Close the sending end

    // Collect results
    let mut results = vec![String::new(); handles.len()];
    for (index, result) in rx {
        match result {
            Ok(processed) => results[index] = processed,
            Err(e) => return Err(e),
        }
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(results)
}
```

### Performance Monitoring and Metrics

```rust
use vi_rust::safety::{SafeAssemblyProcessor, GLOBAL_SAFETY_METRICS};
use std::time::Instant;

fn benchmark_processing() {
    let processor = SafeAssemblyProcessor::new();
    let test_cases = vec![
        ("Short", "Xin chào"),
        ("Medium", "Tiếng Việt là ngôn ngữ rất đẹp".repeat(10)),
        ("Large", "Văn học Việt Nam phong phú".repeat(1000)),
    ];

    for (name, input) in test_cases {
        let start = Instant::now();

        match processor.process_string_safe(&input) {
            Ok(result) => {
                let duration = start.elapsed();
                let chars_per_sec = input.len() as f64 / duration.as_secs_f64();

                println!("{} test: {} chars in {:?} ({:.2} chars/sec)",
                         name, input.len(), duration, chars_per_sec);
            }
            Err(e) => {
                println!("{} test failed: {}", name, e);
            }
        }
    }

    // Print global metrics
    let metrics = &*GLOBAL_SAFETY_METRICS;
    println!("\nGlobal Metrics:");
    println!("Success rate: {:.2}%", metrics.get_success_rate() * 100.0);
    println!("Average overhead: {} ns", metrics.get_average_overhead_ns());
}
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Assembly Compilation Failures

**Issue**: Assembly files fail to compile during build
```
error: unknown directive
  --> src/asm/aarch64_kernels.s:42:5
```

**Causes and Solutions**:
- **Platform mismatch**: Ensure you're compiling on the correct platform
  ```bash
  # Check current platform
  rustc --version --verbose

  # Clean and rebuild
  cargo clean
  cargo build --release
  ```

- **Missing development tools**: Install required build tools
  ```bash
  # macOS
  xcode-select --install

  # Linux
  sudo apt-get install build-essential
  ```

- **Cross-compilation detected**: Assembly is automatically disabled for cross-compilation
  ```bash
  # This is expected behavior - assembly will fall back to Rust
  cargo build --target x86_64-apple-darwin  # On ARM64 Mac
  ```

#### 2. Runtime Assembly Errors

**Issue**: Assembly functions return error codes or crash
```rust
AssemblyError::ExecutionError("Invalid input parameters")
```

**Debugging Steps**:
```rust
// Enable debug mode
#[cfg(debug_assertions)]
vi_rust::safety::enable_assembly_debugging();

// Check input validity
fn debug_assembly_input(input: &str) {
    println!("Input length: {}", input.len());
    println!("Input chars: {:?}", input.chars().take(10).collect::<Vec<_>>());

    // Check for problematic characters
    for (i, ch) in input.chars().enumerate() {
        let code = ch as u32;
        if code > 0x10FFFF {
            println!("Invalid Unicode at position {}: U+{:X}", i, code);
        }
    }
}
```

#### 3. Performance Issues

**Issue**: Assembly processing is slower than expected

**Diagnostic Steps**:
```rust
use std::time::Instant;
use vi_rust::safety::{SafeAssemblyProcessor, GLOBAL_ASSEMBLY_CONTROL};

fn diagnose_performance() {
    let processor = SafeAssemblyProcessor::new();
    let test_input = "Tiếng Việt".repeat(10000);

    // Measure processing time
    let start = Instant::now();
    match processor.process_string_safe(&test_input) {
        Ok(_) => {
            let duration = start.elapsed();
            let chars_per_sec = test_input.len() as f64 / duration.as_secs_f64();

            if chars_per_sec < 1_000_000.0 {
                println!("Performance warning: {:.0} chars/sec (expected >1M)", chars_per_sec);

                // Check safety overhead
                let control = &*GLOBAL_ASSEMBLY_CONTROL;
                let iterations = control.current_iteration.load(std::sync::atomic::Ordering::Relaxed);
                println!("Safety checks performed: {}", iterations / 1024);
            }
        }
        Err(e) => println!("Performance test failed: {}", e),
    }
}
```

#### 4. Memory Safety Violations

**Issue**: Segmentation faults or memory corruption

**Prevention and Detection**:
```rust
// Use AddressSanitizer for debugging
// Add to Cargo.toml:
// [profile.dev]
// opt-level = 1
//
// Then run with:
// RUSTFLAGS="-Z sanitizer=address" cargo run

// Enable bounds checking in assembly
fn enable_strict_bounds_checking() {
    use vi_rust::safety::GLOBAL_ASSEMBLY_CONTROL;

    // Set conservative limits
    GLOBAL_ASSEMBLY_CONTROL.max_iterations.store(1_000_000, std::sync::atomic::Ordering::SeqCst);

    // Enable timeout protection
    GLOBAL_ASSEMBLY_CONTROL.timeout_ms.store(1000, std::sync::atomic::Ordering::SeqCst);
}
```

#### 5. Signal Handling Issues

**Issue**: Assembly doesn't respond to cancellation signals

**Solution**:
```rust
// Verify signal handling is working
fn test_signal_handling() {
    use vi_rust::safety::{SafeAssemblyProcessor, GLOBAL_ASSEMBLY_CONTROL};
    use std::thread;
    use std::time::Duration;

    let processor = SafeAssemblyProcessor::new();
    let large_input = "Test ".repeat(1_000_000);

    // Start processing in background
    let handle = thread::spawn(move || {
        processor.process_string_safe(&large_input)
    });

    // Cancel after 100ms
    thread::sleep(Duration::from_millis(100));
    GLOBAL_ASSEMBLY_CONTROL.cancel_all();

    // Check if cancellation worked
    match handle.join().unwrap() {
        Err(vi_rust::safety::AssemblyError::Cancelled) => {
            println!("Signal handling working correctly");
        }
        _ => {
            println!("Warning: Signal handling may not be working");
        }
    }
}
```

### Best Practices

#### 1. Input Validation

Always validate input before passing to assembly:

```rust
fn safe_process_text(input: &str) -> Result<String, AssemblyError> {
    // Validate input size
    if input.len() > 10_000_000 {
        return Err(AssemblyError::InvalidInput);
    }

    // Check for null bytes (can cause issues in C-style strings)
    if input.contains('\0') {
        return Err(AssemblyError::InvalidInput);
    }

    // Validate Unicode
    for ch in input.chars() {
        if ch as u32 > 0x10FFFF {
            return Err(AssemblyError::InvalidInput);
        }
    }

    let processor = SafeAssemblyProcessor::new();
    processor.process_string_safe(input)
}
```

#### 2. Timeout Configuration

Set appropriate timeouts based on input size:

```rust
fn adaptive_timeout_processing(input: &str) -> Result<String, AssemblyError> {
    // Calculate timeout based on input size (1ms per 1000 chars, minimum 1s)
    let timeout_ms = std::cmp::max(1000, input.len() as u64 / 1000);

    let processor = SafeAssemblyProcessor::with_timeout(timeout_ms);
    processor.process_string_safe(input)
}
```

#### 3. Error Recovery

Implement graceful degradation:

```rust
fn resilient_processing(input: &str) -> String {
    let processor = SafeAssemblyProcessor::new();

    match processor.process_string_safe(input) {
        Ok(result) => result,
        Err(AssemblyError::Timeout) => {
            // Try with Rust fallback for timeout
            fallback_rust_processing(input)
        }
        Err(AssemblyError::ExecutionError(_)) => {
            // Assembly failed, use Rust
            fallback_rust_processing(input)
        }
        Err(_) => {
            // Other errors, return input unchanged
            input.to_string()
        }
    }
}
```

#### 4. Resource Management

Properly manage processor lifecycle:

```rust
struct TextProcessor {
    processor: SafeAssemblyProcessor,
}

impl TextProcessor {
    fn new() -> Self {
        Self {
            processor: SafeAssemblyProcessor::with_timeout(5000),
        }
    }

    fn process_batch(&self, inputs: &[String]) -> Vec<Result<String, AssemblyError>> {
        inputs.iter()
            .map(|input| self.processor.process_string_safe(input))
            .collect()
    }
}

// Processor is automatically cleaned up when dropped
```

#### 5. Performance Monitoring

Monitor assembly performance in production:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static TOTAL_CHARS_PROCESSED: AtomicU64 = AtomicU64::new(0);
static TOTAL_PROCESSING_TIME_NS: AtomicU64 = AtomicU64::new(0);

fn monitored_processing(input: &str) -> Result<String, AssemblyError> {
    let start = Instant::now();
    let processor = SafeAssemblyProcessor::new();

    let result = processor.process_string_safe(input)?;

    let duration_ns = start.elapsed().as_nanos() as u64;
    TOTAL_CHARS_PROCESSED.fetch_add(input.len() as u64, Ordering::Relaxed);
    TOTAL_PROCESSING_TIME_NS.fetch_add(duration_ns, Ordering::Relaxed);

    Ok(result)
}

fn get_performance_stats() -> (f64, u64) {
    let total_chars = TOTAL_CHARS_PROCESSED.load(Ordering::Relaxed);
    let total_time_ns = TOTAL_PROCESSING_TIME_NS.load(Ordering::Relaxed);

    let chars_per_sec = if total_time_ns > 0 {
        (total_chars as f64 * 1_000_000_000.0) / total_time_ns as f64
    } else {
        0.0
    };

    (chars_per_sec, total_chars)
}
```

### Platform-Specific Troubleshooting

#### Apple Silicon (M1/M2/M3)

```bash
# Verify Apple Silicon detection
cargo build --release -v 2>&1 | grep -i "apple\|silicon\|aarch64"

# Check for Rosetta interference
arch -arm64 cargo build --release

# Verify assembly compilation
ls -la target/release/build/*/out/lib*.a
```

#### x86_64 Systems

```bash
# Check CPU feature support
grep -E "(avx|bmi)" /proc/cpuinfo  # Linux
sysctl -a | grep -E "(avx|bmi)"    # macOS

# Verify assembly features
cargo build --release -v 2>&1 | grep -E "(avx|bmi|x86_64)"
```

#### Cross-Compilation

```bash
# Verify cross-compilation is detected
cargo build --target x86_64-apple-darwin -v 2>&1 | grep "no_assembly"

# Expected output should include:
# cargo:rustc-cfg=feature="no_assembly"
```

This comprehensive documentation provides developers with everything needed to understand, use, and troubleshoot the assembly integration in the vi-rust Vietnamese IME project.
```
```
```
