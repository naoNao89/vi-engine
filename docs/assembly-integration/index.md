# Assembly Integration Documentation Index

Welcome to the comprehensive assembly integration documentation for the vi-rust Vietnamese Input Method Engine (IME). This documentation covers all aspects of the sophisticated assembly-Rust integration that enables >11M characters/sec processing performance while maintaining memory safety.

## Quick Start

For developers new to the vi-rust assembly integration:

1. **Start Here**: [Main README](README.md) - Complete overview and examples
2. **Safety First**: [Safety Model](safety-model.md) - Understanding isolation and protection
3. **Platform Setup**: [Platform Guide](platform-guide.md) - Platform-specific configuration
4. **Integration Details**: [Calling Conventions](calling-conventions.md) - Technical interface details

## Documentation Structure

### Core Documentation

#### [README.md](README.md) - Main Integration Guide
**Comprehensive overview covering all aspects of assembly integration**

- Assembly-Rust Integration Architecture
- Isolation and Safety Model  
- Assembly Function Lifecycle
- Error Handling and Recovery
- Platform-Specific Considerations
- Code Examples and Best Practices
- Troubleshooting Guide

**Target Audience**: All developers working with vi-rust assembly integration  
**Prerequisites**: Basic understanding of Rust and assembly concepts  
**Estimated Reading Time**: 45-60 minutes

#### [safety-model.md](safety-model.md) - Safety and Isolation
**Detailed coverage of the multi-layered safety system**

- Isolation Boundaries and Memory Safety
- Cooperative Cancellation System
- Timeout Protection with Watchdog Threads
- Panic Integration and Stack Unwinding
- Resource Management and Cleanup
- Signal Handling for Graceful Shutdown

**Target Audience**: Developers implementing safety-critical features  
**Prerequisites**: Understanding of concurrent programming and system signals  
**Estimated Reading Time**: 30-40 minutes

#### [calling-conventions.md](calling-conventions.md) - ABI and Interface
**Technical reference for assembly-Rust interface**

- Platform-Specific Calling Conventions (ARM64, x86_64)
- Function Signatures and Register Usage
- Memory Layout and Alignment Requirements
- FFI Interface and Type Safety
- Error Handling Conventions
- Cross-Platform Compatibility

**Target Audience**: Systems programmers and assembly developers  
**Prerequisites**: Knowledge of assembly language and ABI concepts  
**Estimated Reading Time**: 25-35 minutes

#### [platform-guide.md](platform-guide.md) - Platform-Specific Guide
**Detailed platform-specific implementation and optimization guide**

- Apple Silicon (ARM64) Optimizations
- x86_64 Intel/AMD Implementations
- Generic ARM64 for Linux
- Cross-Compilation Handling
- Platform-Specific Debugging
- Performance Benchmarking

**Target Audience**: Platform specialists and performance engineers  
**Prerequisites**: Platform-specific assembly knowledge  
**Estimated Reading Time**: 35-45 minutes

## Quick Reference

### Assembly Function Signatures

```rust
// Apple Silicon optimized functions
#[cfg(feature = "apple_silicon_assembly")]
extern "C" {
    fn _apple_hybrid_clean_char_optimized(ch: u32) -> u32;
    fn _apple_hybrid_clean_chars_bulk_neon_optimized(
        input: *const u32, output: *mut u32, len: usize
    ) -> usize;
}

// x86_64 optimized functions  
#[cfg(feature = "x86_64_assembly")]
extern "C" {
    fn hybrid_clean_char_x86_64(ch: u32) -> u32;
    fn hybrid_clean_chars_bulk_avx512(
        input: *const u32, output: *mut u32, len: usize
    ) -> usize;
}

// Generic ARM64 functions
#[cfg(feature = "aarch64_assembly")]
extern "C" {
    fn generic_clean_char_aarch64(ch: u32) -> u32;
    fn generic_clean_chars_bulk_aarch64(
        input: *const u32, output: *mut u32, len: usize
    ) -> usize;
}
```

### Basic Usage Pattern

```rust
use vi_rust::safety::{SafeAssemblyProcessor, AssemblyError};

// Initialize safety system
vi_rust::safety::initialize_assembly_safety()?;

// Create processor
let processor = SafeAssemblyProcessor::new();

// Process Vietnamese text
match processor.process_string_safe("Tiếng Việt") {
    Ok(result) => println!("Result: {}", result), // "Tieng Viet"
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Types Quick Reference

```rust
pub enum AssemblyError {
    Cancelled,              // Operation cancelled by user/system
    Timeout,               // Operation exceeded timeout limit
    IterationLimit,        // Exceeded iteration limit
    Panic,                 // Rust panic occurred during operation
    InvalidInput,          // Invalid input parameters
    ExecutionError(String), // Assembly execution error
}
```

## Architecture Overview

### High-Level Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust Code     │    │   Safety Layer   │    │  Assembly Code  │
│                 │    │                  │    │                 │
│ • Type Safety   │◄──►│ • Cancellation   │◄──►│ • Performance   │
│ • Memory Safety │    │ • Timeout        │    │ • Optimization  │
│ • Error Handling│    │ • Panic Hook     │    │ • SIMD/Vector   │
│ • Resource Mgmt │    │ • Signal Handler │    │ • Platform Opt  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Safety Boundaries

```
High-Level Rust API
├── SafeAssemblyProcessor (Public Interface)
├── Type Safety Layer (char ↔ u32 conversion)
├── Memory Safety Layer (Buffer validation)
├── Control Layer (Cancellation, timeout)
└── Assembly Interface (Unsafe FFI calls)
    └── Platform-Specific Assembly Functions
```

## Performance Characteristics

### Target Performance by Platform

| Platform | Architecture | Target Performance | Actual Performance* |
|----------|-------------|-------------------|-------------------|
| Apple Silicon | ARM64 | >50M chars/sec | 45-55M chars/sec |
| Intel Mac | x86_64 | >20M chars/sec | 18-25M chars/sec |
| Linux ARM64 | ARM64 | >15M chars/sec | 12-18M chars/sec |
| Linux x86_64 | x86_64 | >25M chars/sec | 20-30M chars/sec |
| Rust Fallback | Any | >5M chars/sec | 8-12M chars/sec |

*Performance varies based on input characteristics and system configuration

### Safety Overhead

The safety system adds minimal overhead:

- **Single Character**: ~2-5 ns overhead per call
- **Bulk Processing**: ~0.1-0.5% overhead for safety checks
- **Cancellation Checks**: Every 1024 iterations (~0.01% overhead)
- **Timeout Monitoring**: Background thread, no direct overhead

## Development Workflow

### 1. Setup and Initialization

```rust
// Initialize safety system once at program startup
vi_rust::safety::initialize_assembly_safety()?;
```

### 2. Create Processor Instance

```rust
// Default 5-second timeout
let processor = SafeAssemblyProcessor::new();

// Custom timeout
let processor = SafeAssemblyProcessor::with_timeout(10_000); // 10 seconds
```

### 3. Process Text

```rust
// Single operation
let result = processor.process_string_safe(input)?;

// Batch processing with error handling
let results: Vec<_> = inputs.iter()
    .map(|input| processor.process_string_safe(input))
    .collect();
```

### 4. Monitor Performance

```rust
let metrics = processor.get_metrics();
println!("Success rate: {:.2}%", metrics.get_success_rate() * 100.0);
println!("Average overhead: {} ns", metrics.get_average_overhead_ns());
```

## Common Use Cases

### High-Performance Text Processing

```rust
// Process large documents efficiently
fn process_document(content: &str) -> Result<String, AssemblyError> {
    let processor = SafeAssemblyProcessor::with_timeout(60_000); // 1 minute
    processor.process_string_safe(content)
}
```

### Real-Time Input Processing

```rust
// Process user input with low latency
fn process_user_input(input: char) -> char {
    static PROCESSOR: Lazy<SafeAssemblyProcessor> = 
        Lazy::new(|| SafeAssemblyProcessor::with_timeout(100)); // 100ms
    
    PROCESSOR.process_char_safe(input).unwrap_or(input)
}
```

### Batch Processing with Fallback

```rust
// Robust batch processing with automatic fallback
fn robust_batch_process(inputs: &[String]) -> Vec<String> {
    let processor = SafeAssemblyProcessor::new();
    
    inputs.iter().map(|input| {
        processor.process_string_safe(input)
            .unwrap_or_else(|_| fallback_rust_processing(input))
    }).collect()
}
```

## Troubleshooting Quick Guide

### Common Issues

1. **Assembly Compilation Fails**
   - Check platform compatibility
   - Verify development tools installation
   - Review build script output

2. **Performance Lower Than Expected**
   - Check if assembly is actually being used
   - Verify CPU feature support
   - Monitor safety overhead

3. **Timeout Errors**
   - Increase timeout for large inputs
   - Check for infinite loops in assembly
   - Monitor system resource usage

4. **Cross-Compilation Issues**
   - Assembly is automatically disabled for cross-compilation
   - Verify Rust fallback is working correctly
   - Check feature flags in build output

### Debug Commands

```bash
# Check assembly compilation
cargo build --release -v 2>&1 | grep -i assembly

# Verify feature flags
cargo build --release -v 2>&1 | grep "rustc-cfg"

# Test assembly functions
cargo run --example check_assembly

# Performance benchmark
cargo bench --bench assembly_performance
```

## Contributing

When contributing to the assembly integration:

1. **Safety First**: All changes must maintain safety guarantees
2. **Cross-Platform**: Consider impact on all supported platforms
3. **Performance**: Benchmark changes against baseline
4. **Documentation**: Update relevant documentation sections
5. **Testing**: Add platform-specific tests where appropriate

## Support and Resources

- **Issues**: Report bugs and feature requests on GitHub
- **Performance**: Use built-in benchmarking tools
- **Platform Support**: Check platform compatibility matrix
- **Safety**: Review safety model documentation before modifications

This documentation provides comprehensive coverage of the vi-rust assembly integration system, enabling developers to safely and effectively utilize high-performance assembly optimizations while maintaining Rust's safety guarantees.
