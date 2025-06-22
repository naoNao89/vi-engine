# Production-Ready Vietnamese Text Processing API

This document provides a comprehensive guide to using the vi-rust library's production-ready API with automatic runtime optimization selection.

## Overview

The production API provides:

- **Automatic Architecture Detection**: Runtime detection of CPU capabilities (Apple Silicon, x86_64, ARM64)
- **Dynamic Optimization Selection**: Automatic selection of the best available optimization strategy
- **Seamless Fallback**: Graceful degradation from assembly to Rust implementations
- **Comprehensive Safety**: Production-ready error handling and timeout protection
- **Performance Monitoring**: Built-in statistics and performance tracking
- **Simple API**: High-level interface that abstracts optimization complexity

## Quick Start

### Basic Usage

```rust
use vi::VietnameseTextProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create processor with automatic optimization selection
    let mut processor = VietnameseTextProcessor::new()?;
    
    // Process Vietnamese text
    let result = processor.process_string("Tiếng Việt")?;
    assert_eq!(result, "Tieng Viet");
    
    // Get optimization information
    println!("Using: {}", processor.optimization_info());
    
    Ok(())
}
```

### Advanced Configuration

```rust
use vi::{ProcessorBuilder, VietnameseTextProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = ProcessorBuilder::new()
        .with_timeout(10000)        // 10 second timeout
        .with_monitoring(true)      // Enable performance monitoring
        .with_fallback(true)        // Enable automatic fallback
        .with_max_retries(3)        // Allow up to 3 retries
        .build()?;

    let result = processor.process_string("Xin chào Việt Nam")?;
    println!("Result: {}", result);

    // Check performance statistics
    println!("Success rate: {:.1}%", processor.success_rate());
    println!("Processing rate: {:.0} chars/sec", processor.avg_processing_rate());

    Ok(())
}
```

### Strategy Selection

The library now supports explicit control over optimization strategy selection:

#### Force Rust-Only Processing

```rust
use vi::ProcessorBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Force Rust-only processing (no assembly)
    let mut processor = ProcessorBuilder::new()
        .force_rust_only()
        .build()?;

    let result = processor.process_string("Tiếng Việt")?;
    println!("Result: {}", result);
    println!("Strategy: {}", processor.processor_name());

    Ok(())
}
```

#### Force Assembly Processing

```rust
use vi::ProcessorBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Force assembly if available, error if not
    let mut processor = ProcessorBuilder::new()
        .force_assembly()
        .build()?;

    let result = processor.process_string("Tiếng Việt")?;
    println!("Using assembly: {}", processor.processor_name());

    Ok(())
}
```

#### Specific Strategy Selection

```rust
use vi::{ProcessorBuilder, OptimizationStrategy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Force a specific optimization strategy
    let mut processor = ProcessorBuilder::new()
        .with_strategy(OptimizationStrategy::RustOptimized)
        .build()?;

    let result = processor.process_string("Tiếng Việt")?;
    println!("Strategy: {:?}", processor.selected_strategy());

    Ok(())
}
```

#### Strategy Preferences

```rust
use vi::ProcessorBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prefer Rust but allow assembly fallback
    let mut processor = ProcessorBuilder::new()
        .prefer_rust()
        .build()?;

    // Prefer assembly but allow Rust fallback
    let mut processor2 = ProcessorBuilder::new()
        .prefer_assembly()
        .build()?;

    Ok(())
}
```

## Architecture Detection

The library automatically detects your system's capabilities at runtime:

### Supported Architectures

| Architecture | Detection Method | Optimization Level |
|--------------|------------------|-------------------|
| **Apple Silicon** | macOS + ARM64 detection | Ultra-High (>1B chars/sec) |
| **Generic ARM64** | NEON capability detection | High (>800M chars/sec) |
| **x86_64** | AVX2/BMI2 feature detection | High (>600M chars/sec) |
| **Other** | Fallback detection | Good (>500M chars/sec) |

### CPU Capabilities API

```rust
use vi::CpuCapabilities;

fn main() {
    let capabilities = CpuCapabilities::get();
    
    println!("Architecture: {}", capabilities.architecture_description());
    println!("Performance: {}", capabilities.performance_description());
    println!("Score: {}", capabilities.performance_score);
    
    // Check specific features
    if capabilities.has_feature("neon") {
        println!("NEON SIMD available");
    }
    if capabilities.has_feature("avx2") {
        println!("AVX2 SIMD available");
    }
}
```

## Optimization Selection

The library automatically selects the best optimization strategy:

### Selection Hierarchy

1. **Apple Silicon Assembly** - Hand-optimized NEON kernels for M1/M2/M3
2. **Generic ARM64 Assembly** - NEON-optimized kernels for standard ARM64
3. **x86_64 Assembly** - AVX2/BMI2 optimized kernels for Intel/AMD
4. **Rust Optimized** - Compiler-optimized Rust with auto-vectorization
5. **Rust Standard** - Standard Rust implementation (fallback)

### Optimization Information

```rust
use vi::{OptimizationSelector, VietnameseTextProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = VietnameseTextProcessor::new()?;
    
    // Get detailed optimization information
    println!("Selected strategy: {:?}", 
             OptimizationSelector::get().selected_strategy());
    
    // Get all available profiles
    for profile in OptimizationSelector::get().profiles() {
        println!("Strategy: {:?}", profile.strategy);
        println!("  Available: {}", profile.available);
        println!("  Throughput: {} chars/sec", profile.estimated_throughput);
        if let Some(reason) = &profile.unavailable_reason {
            println!("  Reason: {}", reason);
        }
    }
    
    Ok(())
}
```

## Performance Monitoring

### Built-in Statistics

```rust
use vi::{ProcessorBuilder, VietnameseTextProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = ProcessorBuilder::new()
        .with_monitoring(true)
        .build()?;
    
    // Process some text
    processor.process_string("Tiếng Việt rất đẹp")?;
    
    // Get statistics
    let stats = processor.stats();
    println!("Characters processed: {}", stats.total_chars_processed);
    println!("Success rate: {:.1}%", processor.success_rate());
    println!("Average rate: {:.0} chars/sec", processor.avg_processing_rate());
    println!("Peak rate: {:.0} chars/sec", stats.peak_processing_rate);
    println!("Optimal performance: {}", processor.is_performing_optimally());
    
    Ok(())
}
```

### Performance Metrics

| Metric | Description |
|--------|-------------|
| `total_chars_processed` | Total characters processed |
| `total_strings_processed` | Total strings processed |
| `successful_operations` | Number of successful operations |
| `failed_operations` | Number of failed operations |
| `fallback_operations` | Number of fallback operations |
| `avg_time_per_char_ns` | Average processing time per character |
| `peak_processing_rate` | Peak processing rate (chars/sec) |

## Error Handling

### Comprehensive Error Management

```rust
use vi::{ProcessorBuilder, AssemblyError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = ProcessorBuilder::new()
        .with_fallback(true)
        .with_max_retries(2)
        .build()?;
    
    match processor.process_string("Tiếng Việt") {
        Ok(result) => println!("Success: {}", result),
        Err(AssemblyError::Timeout) => {
            println!("Operation timed out - input too large");
        }
        Err(AssemblyError::Cancelled) => {
            println!("Operation was cancelled");
        }
        Err(AssemblyError::ExecutionError(msg)) => {
            println!("Execution error: {}", msg);
        }
        Err(e) => println!("Other error: {}", e),
    }
    
    Ok(())
}
```

### Automatic Fallback

The library provides automatic fallback when assembly operations fail:

1. **Primary Strategy**: Try selected optimization (assembly if available)
2. **Retry**: Retry with same strategy if transient error
3. **Fallback**: Fall back to pure Rust implementation
4. **Error**: Return error if all strategies fail

## Configuration Options

### ProcessorConfig

```rust
use vi::{ProcessorConfig, ProcessorBuilder, OptimizationPreference};

let config = ProcessorConfig {
    enable_monitoring: true,        // Enable performance monitoring
    operation_timeout_ms: 5000,     // 5 second timeout
    enable_fallback: true,          // Enable automatic fallback
    max_retries: 2,                 // Maximum retry attempts
    optimization_preference: OptimizationPreference::ForceRustOnly,
};

let processor = ProcessorBuilder::new()
    .with_monitoring(config.enable_monitoring)
    .with_timeout(config.operation_timeout_ms)
    .with_fallback(config.enable_fallback)
    .with_max_retries(config.max_retries)
    .with_optimization_preference(config.optimization_preference)
    .build()?;
```

### Strategy Selection Options

| Method | Description | Use Case |
|--------|-------------|----------|
| `.force_rust_only()` | Force Rust-only processing | Security audits, deployment constraints |
| `.force_assembly()` | Force assembly if available | Maximum performance requirements |
| `.prefer_rust()` | Prefer Rust with assembly fallback | Predictable behavior with performance backup |
| `.prefer_assembly()` | Prefer assembly with Rust fallback | Performance-first with reliability backup |
| `.with_strategy(strategy)` | Force specific strategy | Precise control over optimization |

### Optimization Preferences

```rust
use vi::OptimizationPreference;

// Available preferences
OptimizationPreference::Auto                    // Automatic selection (default)
OptimizationPreference::ForceRustOnly          // Force Rust-only processing
OptimizationPreference::ForceAssembly          // Force assembly processing
OptimizationPreference::PreferRust             // Prefer Rust over assembly
OptimizationPreference::PreferAssembly         // Prefer assembly over Rust
OptimizationPreference::ForceSpecific(strategy) // Force specific strategy
```

## Best Practices

### Production Deployment

1. **Use Default Configuration**: The default settings are optimized for production use
2. **Enable Monitoring**: Monitor performance in production environments
3. **Handle Errors Gracefully**: Always handle potential errors appropriately
4. **Check Performance**: Verify optimal performance with `is_performing_optimally()`

### Performance Optimization

1. **Batch Processing**: Process multiple strings together when possible
2. **Reuse Processors**: Create processor once and reuse for multiple operations
3. **Monitor Statistics**: Use built-in monitoring to identify performance issues
4. **Configure Timeouts**: Set appropriate timeouts for your use case

### Example: Production Service

```rust
use vi::{ProcessorBuilder, VietnameseTextProcessor, AssemblyError};
use std::sync::Arc;

pub struct VietnameseService {
    processor: Arc<VietnameseTextProcessor>,
}

impl VietnameseService {
    pub fn new() -> Result<Self, AssemblyError> {
        let processor = ProcessorBuilder::new()
            .with_monitoring(true)
            .with_timeout(10000)  // 10 second timeout for production
            .with_fallback(true)
            .build()?;
        
        Ok(VietnameseService {
            processor: Arc::new(processor),
        })
    }
    
    pub fn process_text(&self, input: &str) -> Result<String, AssemblyError> {
        // Clone the processor for thread safety
        let mut processor = ProcessorBuilder::new()
            .with_monitoring(true)
            .build()?;
        
        processor.process_string(input)
    }
    
    pub fn health_check(&self) -> bool {
        // Create a test processor to check system health
        match VietnameseTextProcessor::new() {
            Ok(mut processor) => {
                match processor.process_string("test") {
                    Ok(_) => processor.is_performing_optimally(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}
```

## Migration from Legacy API

### From Direct Functions

```rust
// Old way
use vi::{clean_char, clean_string};
let result = clean_string("Tiếng Việt");

// New way
use vi::VietnameseTextProcessor;
let mut processor = VietnameseTextProcessor::new()?;
let result = processor.process_string("Tiếng Việt")?;
```

### From SafeAssemblyProcessor

```rust
// Old way
use vi::SafeAssemblyProcessor;
let processor = SafeAssemblyProcessor::new();
let result = processor.process_string_safe("Tiếng Việt")?;

// New way (automatic optimization selection)
use vi::VietnameseTextProcessor;
let mut processor = VietnameseTextProcessor::new()?;
let result = processor.process_string("Tiếng Việt")?;
```

The new API provides all the safety guarantees of the old API while adding automatic optimization selection and enhanced monitoring capabilities.
