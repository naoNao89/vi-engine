//! Safety overhead benchmarks
//!
//! These benchmarks measure the performance impact of safety mechanisms
//! to ensure they add minimal overhead to assembly operations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use vi::safety::{initialize_assembly_safety, SafeAssemblyProcessor};
use vi::{clean_char, clean_string};

fn setup_safety() {
    let _ = initialize_assembly_safety();
}

/// Benchmark single character processing overhead
fn benchmark_single_char_safety_overhead(c: &mut Criterion) {
    setup_safety();

    // Use longer timeout for benchmarking (30 seconds)
    let safe_processor = SafeAssemblyProcessor::with_timeout(30000);

    let test_chars = vec!['à', 'á', 'ả', 'ã', 'ạ', 'đ', 'Đ', 'a', 'z', 'A', 'Z'];

    let mut group = c.benchmark_group("single_char_processing");

    for &ch in &test_chars {
        group.bench_with_input(BenchmarkId::new("safe", ch), &ch, |b, &ch| {
            b.iter(|| {
                let input = vec![ch];
                safe_processor
                    .process_chars_safe(black_box(&input))
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Safe processing failed: {e}");
                        vec![ch] // Fallback to original character
                    })
            });
        });

        group.bench_with_input(BenchmarkId::new("direct", ch), &ch, |b, &ch| {
            b.iter(|| clean_char(black_box(ch)));
        });
    }

    group.finish();
}

/// Benchmark string processing overhead
fn benchmark_string_safety_overhead(c: &mut Criterion) {
    setup_safety();

    // Use longer timeout for benchmarking (30 seconds)
    let safe_processor = SafeAssemblyProcessor::with_timeout(30000);

    let test_strings = vec![
        "Hello World".to_string(),
        "Tiếng Việt".to_string(),
        "Xin chào thế giới".to_string(),
        "à".repeat(100),
        "à".repeat(1000),
        "à".repeat(10000),
    ];

    let mut group = c.benchmark_group("string_processing");

    for test_string in &test_strings {
        let size = test_string.len();

        group.bench_with_input(BenchmarkId::new("safe", size), test_string, |b, s| {
            b.iter(|| {
                safe_processor
                    .process_string_safe(black_box(s))
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Safe processing failed: {e}");
                        s.clone() // Fallback to original string
                    })
            });
        });

        group.bench_with_input(BenchmarkId::new("direct", size), test_string, |b, s| {
            b.iter(|| clean_string(black_box(s)));
        });
    }

    group.finish();
}

/// Benchmark bulk character processing overhead
fn benchmark_bulk_safety_overhead(c: &mut Criterion) {
    setup_safety();

    // Use longer timeout for benchmarking (60 seconds for bulk operations)
    let safe_processor = SafeAssemblyProcessor::with_timeout(60000);

    let sizes = vec![100, 1000, 10000, 100000];

    let mut group = c.benchmark_group("bulk_processing");

    for &size in &sizes {
        let input: Vec<char> = "àáảãạđĐ".repeat(size / 8).chars().collect();

        group.bench_with_input(BenchmarkId::new("safe", size), &input, |b, input| {
            b.iter(|| {
                safe_processor
                    .process_chars_safe(black_box(input))
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Safe processing failed: {e}");
                        input.clone() // Fallback to original input
                    })
            });
        });

        group.bench_with_input(BenchmarkId::new("direct", size), &input, |b, input| {
            b.iter(|| {
                let result: Vec<char> = input.iter().map(|&ch| clean_char(ch)).collect();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark safety check overhead in tight loops
fn benchmark_safety_check_overhead(c: &mut Criterion) {
    setup_safety();

    let control = &*vi::safety::GLOBAL_ASSEMBLY_CONTROL;

    c.bench_function("safety_check_overhead", |b| {
        b.iter(|| {
            control.reset_for_operation(1000);

            // Simulate tight loop with safety checks
            for i in 0..1000 {
                if i % 64 == 0 {
                    // Check every 64 iterations
                    if !control.should_continue() {
                        break;
                    }
                }
                black_box(i);
            }
        });
    });
}

/// Benchmark concurrent safety overhead
fn benchmark_concurrent_safety_overhead(c: &mut Criterion) {
    setup_safety();

    let safe_processor = std::sync::Arc::new(SafeAssemblyProcessor::with_timeout(60000));
    let input = "Tiếng Việt ".repeat(1000);

    c.bench_function("concurrent_safe_processing", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..4 {
                let processor = safe_processor.clone();
                let input_clone = input.clone();
                let handle = std::thread::spawn(move || {
                    processor
                        .process_string_safe(&input_clone)
                        .unwrap_or_else(|e| {
                            eprintln!("Warning: Concurrent processing failed: {e}");
                            input_clone
                        })
                });
                handles.push(handle);
            }

            for handle in handles {
                black_box(handle.join().unwrap());
            }
        });
    });
}

/// Benchmark timeout mechanism overhead
fn benchmark_timeout_overhead(c: &mut Criterion) {
    setup_safety();

    let mut group = c.benchmark_group("timeout_overhead");

    // Test different timeout values
    let timeouts = vec![100, 1000, 5000]; // milliseconds

    for &timeout_ms in &timeouts {
        let processor = SafeAssemblyProcessor::with_timeout(timeout_ms);
        let input = "à".repeat(1000);

        group.bench_with_input(
            BenchmarkId::new("timeout", timeout_ms),
            &input,
            |b, input| {
                b.iter(|| {
                    processor
                        .process_string_safe(black_box(input))
                        .unwrap_or_else(|e| {
                            eprintln!("Warning: Timeout processing failed: {e}");
                            input.clone()
                        })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark metrics collection overhead
fn benchmark_metrics_overhead(c: &mut Criterion) {
    setup_safety();

    let processor = SafeAssemblyProcessor::new();
    let input = "Tiếng Việt";

    c.bench_function("metrics_collection", |b| {
        b.iter(|| {
            let result = processor
                .process_string_safe(black_box(input))
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Metrics processing failed: {e}");
                    input.to_string()
                });
            let metrics = processor.get_metrics();
            black_box((result, metrics.get_success_rate()));
        });
    });
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation_patterns(c: &mut Criterion) {
    setup_safety();

    let processor = SafeAssemblyProcessor::new();

    let mut group = c.benchmark_group("memory_allocation");

    let sizes = vec![10, 100, 1000, 10000];

    for &size in &sizes {
        let input: Vec<char> = "à".repeat(size).chars().collect();

        group.bench_with_input(
            BenchmarkId::new("allocation_pattern", size),
            &input,
            |b, input| {
                b.iter(|| {
                    // Test allocation patterns
                    let result = processor
                        .process_chars_safe(black_box(input))
                        .unwrap_or_else(|e| {
                            eprintln!("Warning: Memory allocation processing failed: {e}");
                            input.clone()
                        });
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    safety_benches,
    benchmark_single_char_safety_overhead,
    benchmark_string_safety_overhead,
    benchmark_bulk_safety_overhead,
    benchmark_safety_check_overhead,
    benchmark_concurrent_safety_overhead,
    benchmark_timeout_overhead,
    benchmark_metrics_overhead,
    benchmark_memory_allocation_patterns
);

criterion_main!(safety_benches);
