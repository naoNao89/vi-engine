# Basic Usage Examples

This document provides practical examples for using the vi-rust cross-compilation and assembly safety system.

## Getting Started

### 1. Simple Vietnamese Text Processing

```rust
use vi::safety::SafeAssemblyProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the safety system
    vi::safety::initialize_assembly_safety()?;
    
    // Create a safe processor
    let processor = SafeAssemblyProcessor::new();
    
    // Process Vietnamese text
    let input = "Xin chào thế giới";
    let result = processor.process_string_safe(input)?;
    
    println!("Input:  {}", input);   // "Xin chào thế giới"
    println!("Output: {}", result);  // "Xin chao the gioi"
    
    Ok(())
}
```

### 2. Cross-Platform Development

```rust
use vi::safety::SafeAssemblyProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vi::safety::initialize_assembly_safety()?;
    
    let processor = SafeAssemblyProcessor::new();
    
    // This code works identically on:
    // - aarch64-apple-darwin (native with assembly)
    // - x86_64-apple-darwin (cross-compile with Rust fallback)
    
    let test_cases = vec![
        "Tiếng Việt",
        "Hà Nội",
        "Đà Nẵng", 
        "Hồ Chí Minh",
        "Cần Thơ",
    ];
    
    for input in test_cases {
        let result = processor.process_string_safe(input)?;
        println!("{} -> {}", input, result);
    }
    
    Ok(())
}
```

### 3. Error Handling

```rust
use vi::safety::{SafeAssemblyProcessor, AssemblyError};

fn process_with_error_handling(input: &str) -> Result<String, AssemblyError> {
    let processor = SafeAssemblyProcessor::with_timeout(1000); // 1 second timeout
    
    match processor.process_string_safe(input) {
        Ok(result) => {
            println!("✅ Successfully processed: {} -> {}", input, result);
            Ok(result)
        }
        Err(AssemblyError::Timeout) => {
            println!("⏰ Processing timed out for input: {}", input);
            Err(AssemblyError::Timeout)
        }
        Err(AssemblyError::Cancelled) => {
            println!("🚫 Processing was cancelled for input: {}", input);
            Err(AssemblyError::Cancelled)
        }
        Err(AssemblyError::InvalidInput) => {
            println!("❌ Invalid input: {}", input);
            Err(AssemblyError::InvalidInput)
        }
        Err(e) => {
            println!("💥 Unexpected error: {}", e);
            Err(e)
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vi::safety::initialize_assembly_safety()?;
    
    // Test with various inputs
    let _ = process_with_error_handling("Tiếng Việt")?;
    let _ = process_with_error_handling("Very long text...".repeat(10000).as_str());
    
    Ok(())
}
```

## Build and Test Examples

### 1. Cross-Compilation Workflow

```bash
#!/bin/bash
# build-all-targets.sh

echo "🏗️  Building for all supported targets..."

# Native Mac ARM (with assembly)
echo "Building native Mac ARM..."
cargo build --target aarch64-apple-darwin --release
echo "✅ Native build complete"

# x86_64 simulation (Rust fallback)
echo "Building x86_64 simulation..."
cargo build --target x86_64-apple-darwin --release
echo "✅ Cross-compilation complete"

# Test both targets
echo "🧪 Testing both targets..."
cargo test --target aarch64-apple-darwin --release
cargo test --target x86_64-apple-darwin --release
echo "✅ All tests passed"

# Run safety tests
echo "🔒 Testing safety system..."
cargo test --test safety_tests --target aarch64-apple-darwin
cargo test --test safety_tests --target x86_64-apple-darwin
echo "✅ Safety tests passed"

echo "🎉 All builds and tests completed successfully!"
```

### 2. Performance Comparison

```bash
#!/bin/bash
# compare-performance.sh

echo "📊 Comparing performance across targets..."

echo "Native Mac ARM performance:"
cargo bench --target aarch64-apple-darwin --bench safety_benchmark -- --test

echo "x86_64 simulation performance:"
cargo bench --target x86_64-apple-darwin --bench safety_benchmark -- --test

echo "Performance comparison complete!"
```

### 3. Development Workflow

```bash
#!/bin/bash
# dev-workflow.sh

# Quick development cycle
echo "🔄 Development workflow..."

# 1. Check compilation on both targets
echo "Checking compilation..."
cargo check --target aarch64-apple-darwin
cargo check --target x86_64-apple-darwin

# 2. Run tests on native target
echo "Running native tests..."
cargo test --target aarch64-apple-darwin

# 3. Verify cross-compilation works
echo "Verifying cross-compilation..."
cargo test --target x86_64-apple-darwin --lib

# 4. Quick safety check
echo "Safety system check..."
cargo test --test safety_tests test_safe_processor_creation

echo "✅ Development workflow complete!"
```

## Integration Examples

### 1. Web Service Integration

```rust
// Example: Actix-web service with vi-rust
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use vi::safety::SafeAssemblyProcessor;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ProcessRequest {
    text: String,
}

#[derive(Serialize)]
struct ProcessResponse {
    original: String,
    processed: String,
    success: bool,
    error: Option<String>,
}

async fn process_vietnamese(req: web::Json<ProcessRequest>) -> Result<HttpResponse> {
    let processor = SafeAssemblyProcessor::with_timeout(5000); // 5 second timeout
    
    match processor.process_string_safe(&req.text) {
        Ok(processed) => Ok(HttpResponse::Ok().json(ProcessResponse {
            original: req.text.clone(),
            processed,
            success: true,
            error: None,
        })),
        Err(e) => Ok(HttpResponse::BadRequest().json(ProcessResponse {
            original: req.text.clone(),
            processed: String::new(),
            success: false,
            error: Some(format!("{}", e)),
        })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize safety system
    vi::safety::initialize_assembly_safety().unwrap();
    
    HttpServer::new(|| {
        App::new()
            .route("/process", web::post().to(process_vietnamese))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### 2. CLI Application

```rust
// Example: Command-line tool
use clap::{Arg, Command};
use vi::safety::SafeAssemblyProcessor;
use std::fs;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("vi-clean")
        .version("1.0")
        .about("Remove Vietnamese diacritics from text")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .value_name("FILE")
            .help("Input file (stdin if not specified)"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FILE")
            .help("Output file (stdout if not specified)"))
        .arg(Arg::new("timeout")
            .short('t')
            .long("timeout")
            .value_name("MS")
            .help("Timeout in milliseconds")
            .default_value("5000"))
        .get_matches();

    // Initialize safety system
    vi::safety::initialize_assembly_safety()?;
    
    let timeout: u64 = matches.get_one::<String>("timeout")
        .unwrap()
        .parse()
        .unwrap_or(5000);
    
    let processor = SafeAssemblyProcessor::with_timeout(timeout);
    
    // Read input
    let input = if let Some(file) = matches.get_one::<String>("input") {
        fs::read_to_string(file)?
    } else {
        let stdin = io::stdin();
        let mut lines = Vec::new();
        for line in stdin.lock().lines() {
            lines.push(line?);
        }
        lines.join("\n")
    };
    
    // Process text
    let output = processor.process_string_safe(&input)?;
    
    // Write output
    if let Some(file) = matches.get_one::<String>("output") {
        fs::write(file, output)?;
    } else {
        println!("{}", output);
    }
    
    Ok(())
}
```

### 3. Batch Processing

```rust
use vi::safety::SafeAssemblyProcessor;
use std::sync::Arc;
use std::thread;

fn batch_process_files(file_paths: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    vi::safety::initialize_assembly_safety()?;
    
    let processor = Arc::new(SafeAssemblyProcessor::with_timeout(10000));
    let mut handles = vec![];
    
    for file_path in file_paths {
        let processor_clone = processor.clone();
        let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            println!("Processing: {}", file_path);
            
            let content = std::fs::read_to_string(&file_path)?;
            let processed = processor_clone.process_string_safe(&content)?;
            
            let output_path = format!("{}.cleaned", file_path);
            std::fs::write(output_path, processed)?;
            
            println!("Completed: {}", file_path);
            Ok(())
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        if let Err(e) = handle.join().unwrap() {
            eprintln!("Error processing file: {}", e);
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "document1.txt".to_string(),
        "document2.txt".to_string(),
        "document3.txt".to_string(),
    ];
    
    batch_process_files(files)?;
    Ok(())
}
```

## Testing Examples

### 1. Unit Test Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vi::safety::{SafeAssemblyProcessor, initialize_assembly_safety};

    #[test]
    fn test_basic_processing() {
        initialize_assembly_safety().unwrap();
        let processor = SafeAssemblyProcessor::new();
        
        let result = processor.process_string_safe("Tiếng Việt").unwrap();
        assert_eq!(result, "Tieng Viet");
    }

    #[test]
    fn test_empty_input() {
        initialize_assembly_safety().unwrap();
        let processor = SafeAssemblyProcessor::new();
        
        let result = processor.process_string_safe("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_non_vietnamese_text() {
        initialize_assembly_safety().unwrap();
        let processor = SafeAssemblyProcessor::new();
        
        let result = processor.process_string_safe("Hello World").unwrap();
        assert_eq!(result, "Hello World");
    }
}
```

### 2. Integration Test

```rust
// tests/integration_test.rs
use vi::safety::{SafeAssemblyProcessor, initialize_assembly_safety};

#[test]
fn test_cross_platform_consistency() {
    initialize_assembly_safety().unwrap();
    let processor = SafeAssemblyProcessor::new();
    
    let test_cases = vec![
        ("Tiếng Việt", "Tieng Viet"),
        ("Hà Nội", "Ha Noi"),
        ("Đà Nẵng", "Da Nang"),
    ];
    
    for (input, expected) in test_cases {
        let result = processor.process_string_safe(input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}
```

### 3. Benchmark Example

```rust
// benches/custom_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};
use vi::safety::{SafeAssemblyProcessor, initialize_assembly_safety};

fn benchmark_vietnamese_processing(c: &mut Criterion) {
    initialize_assembly_safety().unwrap();
    let processor = SafeAssemblyProcessor::new();
    
    let input = "Tiếng Việt rất hay và đẹp ".repeat(1000);
    
    c.bench_function("vietnamese_processing_1k", |b| {
        b.iter(|| {
            processor.process_string_safe(&input).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_vietnamese_processing);
criterion_main!(benches);
```

## Configuration Examples

### 1. Cargo.toml for Applications

```toml
[package]
name = "my-vietnamese-app"
version = "0.1.0"
edition = "2021"

[dependencies]
vi = { path = "../vi-engine" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# For cross-compilation support
[target.x86_64-apple-darwin]
rustflags = ["-C", "target-cpu=x86-64"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=native"]
```

### 2. GitHub Actions Workflow

```yaml
name: Cross-Platform CI

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [aarch64-apple-darwin, x86_64-apple-darwin]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Run tests
      run: |
        cargo test --target ${{ matrix.target }}
        cargo test --test safety_tests --target ${{ matrix.target }}
    
    - name: Run benchmarks
      run: cargo bench --target ${{ matrix.target }} --bench safety_benchmark -- --test
```

These examples demonstrate practical usage patterns for the vi-rust cross-compilation and assembly safety system across different scenarios and platforms.
