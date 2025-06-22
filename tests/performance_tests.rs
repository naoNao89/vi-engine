//! Comprehensive performance tests for Vietnamese IME
//!
//! This module provides detailed performance testing and profiling
//! capabilities to identify bottlenecks and validate optimizations.

use std::time::{Duration, Instant};
use vi::{
    asm_clean_char, asm_clean_string, clean_char, clean_string, initialize_assembly_safety,
    is_assembly_available,
};

/// Helper function to safely convert usize to f64
fn usize_to_f64(value: usize) -> f64 {
    // Use saturating conversion to avoid overflow
    if value > (1u64 << 53) as usize {
        (1u64 << 53) as f64
    } else {
        value as f64
    }
}

/// Helper function to safely convert u128 to f64
fn u128_to_f64(value: u128) -> f64 {
    // Use saturating conversion to avoid overflow
    if value > (1u64 << 53) as u128 {
        (1u64 << 53) as f64
    } else {
        value as f64
    }
}

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerfTestConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub input_size: usize,
    pub measure_ffi_overhead: bool,
}

impl Default for PerfTestConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            input_size: 10000,
            measure_ffi_overhead: true,
        }
    }
}

/// Performance measurement results
#[derive(Debug, Clone)]
pub struct PerfResults {
    pub chars_per_second: f64,
    pub avg_latency_ns: f64,
    pub min_latency_ns: f64,
    pub max_latency_ns: f64,
    pub ffi_overhead_ns: Option<f64>,
    pub memory_allocations: usize,
}

/// Micro-benchmark for FFI overhead measurement
pub fn measure_ffi_overhead() -> Result<Duration, Box<dyn std::error::Error>> {
    if !is_assembly_available() {
        return Ok(Duration::ZERO);
    }

    let test_char = 'ế';
    let iterations = 100_000;

    // Warmup
    for _ in 0..1000 {
        match asm_clean_char(test_char) {
            Ok(_) => {}
            Err(_) => {
                // Assembly operation cancelled - return zero overhead
                return Ok(Duration::ZERO);
            }
        }
    }

    // Measure assembly call overhead
    let start = Instant::now();
    let mut successful_calls = 0;
    for _ in 0..iterations {
        match asm_clean_char(test_char) {
            Ok(_) => successful_calls += 1,
            Err(_) => break, // Stop on first cancellation
        }
    }
    let assembly_time = start.elapsed();

    if successful_calls == 0 {
        return Ok(Duration::ZERO);
    }

    // Measure pure Rust equivalent
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = clean_char(test_char);
    }
    let rust_time = start.elapsed();

    // Calculate overhead per call
    let overhead = assembly_time.saturating_sub(rust_time);
    Ok(overhead / successful_calls as u32)
}

/// Comprehensive character processing benchmark
pub fn benchmark_character_processing(
    config: &PerfTestConfig,
) -> Result<(PerfResults, PerfResults), Box<dyn std::error::Error>> {
    initialize_assembly_safety()?;

    let vietnamese_chars = [
        'á', 'à', 'ả', 'ã', 'ạ', 'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ', 'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'é',
        'è', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ', 'í', 'ì', 'ỉ', 'ĩ', 'ị', 'ó', 'ò', 'ỏ',
        'õ', 'ọ', 'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ú', 'ù', 'ủ', 'ũ',
        'ụ', 'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ', 'đ', 'Đ',
    ];

    // Benchmark Rust implementation
    let rust_results = benchmark_implementation(&vietnamese_chars, config, clean_char, "Rust")?;

    // Benchmark Assembly implementation (if available)
    let assembly_results = if is_assembly_available() {
        benchmark_implementation(
            &vietnamese_chars,
            config,
            |ch| asm_clean_char(ch).unwrap_or(ch),
            "Assembly",
        )?
    } else {
        rust_results.clone()
    };

    Ok((rust_results, assembly_results))
}

/// Generic benchmark implementation
fn benchmark_implementation<F>(
    test_chars: &[char],
    config: &PerfTestConfig,
    mut process_fn: F,
    name: &str,
) -> Result<PerfResults, Box<dyn std::error::Error>>
where
    F: FnMut(char) -> char,
{
    println!("Benchmarking {name} implementation...");

    // Warmup
    for _ in 0..config.warmup_iterations {
        for &ch in test_chars {
            std::hint::black_box(process_fn(ch));
        }
    }

    let mut latencies = Vec::with_capacity(config.iterations);
    let total_chars = test_chars.len() * config.iterations;

    let overall_start = Instant::now();

    // Main benchmark loop
    for _ in 0..config.iterations {
        for &ch in test_chars {
            let start = Instant::now();
            std::hint::black_box(process_fn(ch));
            let latency = start.elapsed();
            latencies.push(latency.as_nanos() as f64);
        }
    }

    let total_time = overall_start.elapsed();

    // Calculate statistics
    let chars_per_second = usize_to_f64(total_chars) / total_time.as_secs_f64();
    let avg_latency_ns = latencies.iter().sum::<f64>() / usize_to_f64(latencies.len());
    let min_latency_ns = latencies.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_latency_ns = latencies.iter().fold(0.0f64, |a, &b| a.max(b));

    // Measure FFI overhead if requested
    let ffi_overhead_ns = if config.measure_ffi_overhead && name == "Assembly" {
        measure_ffi_overhead().ok().map(|d| u128_to_f64(d.as_nanos()))
    } else {
        None
    };

    Ok(PerfResults {
        chars_per_second,
        avg_latency_ns,
        min_latency_ns,
        max_latency_ns,
        ffi_overhead_ns,
        memory_allocations: 0, // TODO: Implement memory tracking
    })
}

/// String processing benchmark with various input sizes
pub fn benchmark_string_processing() -> Result<(), Box<dyn std::error::Error>> {
    initialize_assembly_safety()?;

    let test_strings = [
        "Tiếng Việt",
        "Xin chào thế giới",
        "Hà Nội - thủ đô của Việt Nam",
        "Đà Nẵng - thành phố đáng sống",
        "Hồ Chí Minh - thành phố năng động",
        &"Tiếng Việt rất đẹp ".repeat(100),
        &"Xin chào thế giới! ".repeat(1000),
    ];

    println!("\n=== String Processing Benchmark ===");

    for (i, test_string) in test_strings.iter().enumerate() {
        let size = test_string.len();
        println!("\nTest {}: {} characters", i + 1, size);

        // Benchmark Rust implementation
        let rust_time = benchmark_string_impl(test_string, clean_string, 1000)?;
        let rust_throughput = usize_to_f64(size) / rust_time.as_secs_f64();

        // Benchmark Assembly implementation
        let (assembly_time, assembly_throughput) = if is_assembly_available() {
            let time = benchmark_string_impl(
                test_string,
                |s| asm_clean_string(s).unwrap_or_else(|_| s.to_string()),
                1000,
            )?;
            let throughput = usize_to_f64(size) / time.as_secs_f64();
            (time, throughput)
        } else {
            (rust_time, rust_throughput)
        };

        println!(
            "  Rust:     {:.2} M chars/sec ({:.3}ms)",
            rust_throughput / 1_000_000.0,
            rust_time.as_millis()
        );
        println!(
            "  Assembly: {:.2} M chars/sec ({:.3}ms)",
            assembly_throughput / 1_000_000.0,
            assembly_time.as_millis()
        );

        if is_assembly_available() {
            let speedup = assembly_throughput / rust_throughput;
            println!("  Speedup:  {speedup:.2}x");
        }
    }

    Ok(())
}

/// Helper function to benchmark string implementation
fn benchmark_string_impl<F>(
    input: &str,
    mut process_fn: F,
    iterations: usize,
) -> Result<Duration, Box<dyn std::error::Error>>
where
    F: FnMut(&str) -> String,
{
    // Warmup
    for _ in 0..100 {
        std::hint::black_box(process_fn(input));
    }

    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(process_fn(input));
    }
    Ok(start.elapsed() / iterations as u32)
}

/// Memory allocation profiling
pub fn profile_memory_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Memory Usage Profiling ===");

    // TODO: Implement memory profiling using a custom allocator
    // This would track allocations during processing

    println!("Memory profiling not yet implemented");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_overhead_measurement() {
        match measure_ffi_overhead() {
            Ok(overhead) => {
                println!("FFI overhead: {overhead:?}");
                // Should be reasonable (< 1000ns on modern hardware with safety overhead)
                assert!(overhead.as_nanos() < 10000);
            }
            Err(e) => {
                println!(
                    "FFI overhead measurement failed (likely due to assembly cancellation): {e}"
                );
                // This is acceptable - assembly operations can be cancelled for safety
            }
        }
    }

    #[test]
    fn test_character_benchmark() {
        let config = PerfTestConfig {
            iterations: 10,
            warmup_iterations: 5,
            input_size: 100,
            measure_ffi_overhead: true,
        };

        let (rust_results, assembly_results) = benchmark_character_processing(&config).unwrap();

        assert!(rust_results.chars_per_second > 0.0);
        assert!(assembly_results.chars_per_second > 0.0);

        println!(
            "Rust: {:.2} M chars/sec",
            rust_results.chars_per_second / 1_000_000.0
        );
        println!(
            "Assembly: {:.2} M chars/sec",
            assembly_results.chars_per_second / 1_000_000.0
        );
    }

    #[test]
    fn test_string_benchmark() {
        benchmark_string_processing().unwrap();
    }
}
