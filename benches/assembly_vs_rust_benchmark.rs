use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use vi::{
    asm_clean_char_unsafe, asm_clean_string_unsafe, clean_char, clean_string,
    initialize_assembly_safety, is_assembly_available, SafeAssemblyProcessor,
};

/// Performance-optimized benchmark without safety overhead
fn performance_optimized_benchmark(c: &mut Criterion) {
    let test_string = "Tiếng Việt rất đẹp và phong phú ".repeat(100);

    let mut group = c.benchmark_group("performance_optimized");
    group.throughput(Throughput::Bytes(test_string.len() as u64));

    // Rust baseline
    group.bench_function("rust_optimized", |b| {
        b.iter(|| black_box(clean_string(black_box(&test_string))))
    });

    // Assembly without safety overhead (if available)
    if is_assembly_available() {
        group.bench_function("assembly_unsafe", |b| {
            b.iter(|| black_box(asm_clean_string_unsafe(black_box(&test_string))))
        });
    }

    group.finish();
}

/// Assembly vs Rust performance benchmark with detailed analysis
fn assembly_vs_rust_character_benchmark(c: &mut Criterion) {
    // Initialize assembly safety system
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let vietnamese_chars = [
        'á', 'à', 'ả', 'ã', 'ạ', 'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ', 'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'é',
        'è', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ', 'í', 'ì', 'ỉ', 'ĩ', 'ị', 'ó', 'ò', 'ỏ',
        'õ', 'ọ', 'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ú', 'ù', 'ủ', 'ũ',
        'ụ', 'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ', 'đ', 'Đ',
    ];

    let mut group = c.benchmark_group("assembly_vs_rust_character_processing");
    group.throughput(Throughput::Elements(vietnamese_chars.len() as u64));

    // Benchmark Rust implementation
    group.bench_function("rust_single_chars", |b| {
        b.iter(|| {
            for &ch in vietnamese_chars.iter() {
                black_box(clean_char(black_box(ch)));
            }
        })
    });

    // Benchmark Assembly implementation through SafeAssemblyProcessor
    let processor = SafeAssemblyProcessor::new();
    group.bench_function("assembly_single_chars", |b| {
        b.iter(|| {
            let chars_vec: Vec<char> = vietnamese_chars.to_vec();
            black_box(
                processor
                    .process_chars_safe(&chars_vec)
                    .unwrap_or(chars_vec),
            );
        })
    });

    group.finish();
}

/// Benchmark string processing with various sizes
fn assembly_vs_rust_string_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let test_cases = [
        ("small", "Tiếng Việt"),
        ("medium", &"Xin chào thế giới! ".repeat(10)),
        ("large", &"Hà Nội - thủ đô của Việt Nam. ".repeat(100)),
        ("xlarge", &"Đà Nẵng - thành phố đáng sống. ".repeat(1000)),
    ];

    let mut group = c.benchmark_group("assembly_vs_rust_string_processing");
    let processor = SafeAssemblyProcessor::new();

    for (size_name, test_string) in test_cases.iter() {
        group.throughput(Throughput::Bytes(test_string.len() as u64));

        // Rust implementation
        group.bench_with_input(BenchmarkId::new("rust", size_name), test_string, |b, s| {
            b.iter(|| black_box(clean_string(black_box(s))))
        });

        // Assembly implementation through SafeAssemblyProcessor
        group.bench_with_input(
            BenchmarkId::new("assembly", size_name),
            test_string,
            |b, s| {
                b.iter(|| {
                    black_box(
                        processor
                            .process_string_safe(black_box(s))
                            .unwrap_or_else(|_| s.to_string()),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark FFI overhead specifically
fn ffi_overhead_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let test_char = 'ế';
    let processor = SafeAssemblyProcessor::new();

    let mut group = c.benchmark_group("ffi_overhead");
    group.throughput(Throughput::Elements(1));

    // Pure Rust baseline
    group.bench_function("rust_baseline", |b| {
        b.iter(|| black_box(clean_char(black_box(test_char))))
    });

    // Assembly with safety overhead
    group.bench_function("assembly_with_safety", |b| {
        b.iter(|| {
            let chars = vec![test_char];
            black_box(processor.process_chars_safe(&chars).unwrap_or(chars))
        })
    });

    group.finish();
}

/// Benchmark bulk processing performance
fn bulk_processing_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let sizes = [100, 1000, 10000, 100000];
    let vietnamese_chars = [
        'á', 'à', 'ả', 'ã', 'ạ', 'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ', 'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'é',
        'è', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ', 'í', 'ì', 'ỉ', 'ĩ', 'ị', 'ó', 'ò', 'ỏ',
        'õ', 'ọ', 'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ú', 'ù', 'ủ', 'ũ',
        'ụ', 'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ', 'đ', 'Đ',
    ];

    let mut group = c.benchmark_group("bulk_processing");

    for &size in sizes.iter() {
        // Create test data by cycling through Vietnamese characters
        let test_data: Vec<char> = (0..size)
            .map(|i| vietnamese_chars[i % vietnamese_chars.len()])
            .collect();

        group.throughput(Throughput::Elements(size as u64));

        // Rust implementation (character by character)
        group.bench_with_input(
            BenchmarkId::new("rust_char_by_char", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let result: Vec<char> = data.iter().map(|&ch| clean_char(ch)).collect();
                    black_box(result)
                })
            },
        );

        // Rust string processing
        group.bench_with_input(
            BenchmarkId::new("rust_string", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let input: String = data.iter().collect();
                    black_box(clean_string(&input))
                })
            },
        );

        // Assembly implementation through SafeAssemblyProcessor
        let processor = SafeAssemblyProcessor::new();
        group.bench_with_input(
            BenchmarkId::new("assembly_string", size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let input: String = data.iter().collect();
                    black_box(processor.process_string_safe(&input).unwrap_or(input))
                })
            },
        );
    }

    group.finish();
}

/// Memory allocation benchmark
fn memory_allocation_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let test_string = "Tiếng Việt rất đẹp và phong phú. ".repeat(1000);

    let mut group = c.benchmark_group("memory_allocation");
    group.throughput(Throughput::Bytes(test_string.len() as u64));

    // Rust implementation
    group.bench_function("rust_with_allocation", |b| {
        b.iter(|| {
            let result = clean_string(black_box(&test_string));
            black_box(result)
        })
    });

    // Assembly implementation through SafeAssemblyProcessor
    let processor = SafeAssemblyProcessor::new();
    group.bench_function("assembly_with_allocation", |b| {
        b.iter(|| {
            let result = processor
                .process_string_safe(black_box(&test_string))
                .unwrap_or_else(|_| test_string.clone());
            black_box(result)
        })
    });

    group.finish();
}

/// Cache performance benchmark
fn cache_performance_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    // Create data that will stress different cache levels
    let small_data = "Tiếng Việt ".repeat(10); // L1 cache
    let medium_data = "Tiếng Việt ".repeat(1000); // L2 cache
    let large_data = "Tiếng Việt ".repeat(100000); // L3 cache / main memory

    let test_cases = [
        ("l1_cache", &small_data),
        ("l2_cache", &medium_data),
        ("l3_cache", &large_data),
    ];

    let mut group = c.benchmark_group("cache_performance");

    for (cache_level, test_data) in test_cases.iter() {
        group.throughput(Throughput::Bytes(test_data.len() as u64));

        // Rust implementation
        group.bench_with_input(
            BenchmarkId::new("rust", cache_level),
            test_data,
            |b, data| b.iter(|| black_box(clean_string(black_box(data)))),
        );

        // Assembly implementation through SafeAssemblyProcessor
        let processor = SafeAssemblyProcessor::new();
        group.bench_with_input(
            BenchmarkId::new("assembly", cache_level),
            test_data,
            |b, data| {
                b.iter(|| {
                    black_box(
                        processor
                            .process_string_safe(black_box(data))
                            .unwrap_or_else(|_| data.to_string()),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Concurrent processing benchmark
fn concurrent_benchmark(c: &mut Criterion) {
    initialize_assembly_safety().expect("Failed to initialize assembly safety");

    let test_data = "Xin chào thế giới! Tiếng Việt rất đẹp. ".repeat(100);
    let thread_counts = [1, 2, 4, 8];

    let mut group = c.benchmark_group("concurrent_processing");

    for &thread_count in thread_counts.iter() {
        group.throughput(Throughput::Bytes((test_data.len() * thread_count) as u64));

        // Rust implementation
        group.bench_with_input(
            BenchmarkId::new("rust", thread_count),
            &(test_data.clone(), thread_count),
            |b, (data, threads)| {
                b.iter(|| {
                    let handles: Vec<_> = (0..*threads)
                        .map(|_| {
                            let data_clone = data.clone();
                            std::thread::spawn(move || clean_string(&data_clone))
                        })
                        .collect();

                    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                    black_box(results)
                })
            },
        );

        // Assembly implementation through SafeAssemblyProcessor
        group.bench_with_input(
            BenchmarkId::new("assembly", thread_count),
            &(test_data.clone(), thread_count),
            |b, (data, threads)| {
                b.iter(|| {
                    let handles: Vec<_> = (0..*threads)
                        .map(|_| {
                            let data_clone = data.clone();
                            std::thread::spawn(move || {
                                let processor = SafeAssemblyProcessor::new();
                                processor
                                    .process_string_safe(&data_clone)
                                    .unwrap_or(data_clone)
                            })
                        })
                        .collect();

                    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                    black_box(results)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    assembly_vs_rust_benches,
    performance_optimized_benchmark,
    assembly_vs_rust_character_benchmark,
    assembly_vs_rust_string_benchmark,
    ffi_overhead_benchmark,
    bulk_processing_benchmark,
    memory_allocation_benchmark,
    cache_performance_benchmark,
    concurrent_benchmark
);

criterion_main!(assembly_vs_rust_benches);
