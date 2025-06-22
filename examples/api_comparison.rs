#!/usr/bin/env cargo
//! API Comparison Example
//!
//! This example demonstrates the difference between the standard Rust API
//! and the high-performance assembly API, showing performance comparisons
//! and feature differences.

use std::time::Instant;
use vi::{
    // Assembly API
    asm_clean_char,
    asm_clean_string,
    // Standard API
    clean_char,
    clean_string,
    get_assembly_info,
    // Safety infrastructure
    initialize_assembly_safety,
    is_assembly_available,
    is_vowel,
    SafeAssemblyProcessor,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Vietnamese IME API Comparison ===");
    println!();

    // Initialize assembly safety system
    initialize_assembly_safety()?;

    // Display system information
    println!("=== System Information ===");
    println!("Assembly Available: {}", is_assembly_available());
    println!("Assembly Info: {}", get_assembly_info());
    println!();

    // Character processing comparison
    println!("=== Character Processing Comparison ===");
    let test_chars = ['ế', 'ạ', 'ữ', 'đ', 'Đ', 'ă', 'â', 'ê', 'ô', 'ơ', 'ư'];

    println!("Standard API vs Assembly API:");
    for ch in test_chars {
        let rust_result = clean_char(ch);
        let asm_result = asm_clean_char(ch)?;
        let is_viet_vowel = is_vowel(ch);

        println!(
            "'{ch}' -> Rust: '{rust_result}', Assembly: '{asm_result}', Is Vowel: {is_viet_vowel}"
        );

        // Verify results match
        assert_eq!(rust_result, asm_result, "Results should match for '{ch}'");
    }
    println!();

    // String processing comparison
    println!("=== String Processing Comparison ===");
    let test_strings = [
        "Tiếng Việt",
        "Xin chào thế giới",
        "Hà Nội - thủ đô của Việt Nam",
        "Đà Nẵng - thành phố đáng sống",
        "Hồ Chí Minh - thành phố năng động",
    ];

    for input in test_strings {
        let rust_result = clean_string(input);
        let asm_result = asm_clean_string(input)?;

        println!("Input:    '{input}'");
        println!("Rust:     '{rust_result}'");
        println!("Assembly: '{asm_result}'");
        println!();

        // Verify results match
        assert_eq!(
            rust_result, asm_result,
            "Results should match for '{input}'"
        );
    }

    // Performance comparison
    println!("=== Performance Comparison ===");
    let large_text = "Tiếng Việt là ngôn ngữ chính thức của Việt Nam. ".repeat(1000);
    let iterations = 100;

    println!(
        "Testing with {} characters, {} iterations",
        large_text.len(),
        iterations
    );
    println!();

    // Rust implementation benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = clean_string(&large_text);
    }
    let rust_duration = start.elapsed();
    let rust_chars_per_sec = (large_text.len() * iterations) as f64 / rust_duration.as_secs_f64();

    println!("Rust Implementation:");
    println!("  Total time: {rust_duration:?}");
    println!(
        "  Performance: {:.2} M chars/sec",
        rust_chars_per_sec / 1_000_000.0
    );
    println!();

    // Assembly implementation benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = asm_clean_string(&large_text)?;
    }
    let asm_duration = start.elapsed();
    let asm_chars_per_sec = {
        let total_chars = large_text.len() * iterations;
        let total_chars_f64 = if total_chars > (1u64 << 53) as usize {
            (1u64 << 53) as f64
        } else {
            total_chars as f64
        };
        total_chars_f64 / asm_duration.as_secs_f64()
    };

    println!("Assembly Implementation:");
    println!("  Total time: {asm_duration:?}");
    println!(
        "  Performance: {:.2} M chars/sec",
        asm_chars_per_sec / 1_000_000.0
    );
    println!();

    // Calculate speedup
    let speedup = rust_duration.as_secs_f64() / asm_duration.as_secs_f64();
    println!("Assembly Speedup: {speedup:.2}x faster");
    println!();

    // Bulk processing demonstration
    println!("=== Bulk Processing Demonstration ===");
    let input_chars: Vec<char> = "Tiếng Việt rất đẹp".chars().collect();

    let processor = SafeAssemblyProcessor::new();
    let output_chars = processor.process_chars_safe(&input_chars)?;
    let result: String = output_chars.iter().collect();

    println!("Bulk processing:");
    println!("  Input:  {input_chars:?}");
    println!("  Output: {output_chars:?}");
    println!("  Result: '{result}'");
    println!("  Processed: {} characters", output_chars.len());
    println!();

    println!("=== API Comparison Complete ===");
    println!();
    println!("Summary:");
    println!("- Standard API: Always available, pure Rust implementation");
    println!("- Assembly API: Platform-dependent, high-performance optimizations");
    println!("- Both APIs produce identical results");
    println!("- Assembly API provides significant performance improvements when available");

    Ok(())
}
