//! Example demonstrating safe assembly processing
//!
//! This example shows how to use the `SafeAssemblyProcessor` for robust
//! Vietnamese text processing with comprehensive safety guarantees.

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use vi::safety::{initialize_assembly_safety, SafeAssemblyProcessor, GLOBAL_SAFETY_METRICS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the safety system
    initialize_assembly_safety()?;
    println!("Assembly safety system initialized");

    // Basic usage example
    basic_usage_example()?;

    // Timeout protection example
    timeout_protection_example()?;

    // Concurrent processing example
    concurrent_processing_example()?;

    // Cancellation example
    cancellation_example()?;

    // Metrics example
    metrics_example();

    println!("\nAll examples completed successfully!");
    Ok(())
}

fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Basic Usage Example ===");

    let processor = SafeAssemblyProcessor::new();

    // Process Vietnamese text
    let test_cases = vec![
        "Xin chào",
        "Tiếng Việt",
        "Hôm nay trời đẹp",
        "Cảm ơn bạn rất nhiều",
        "Chúc mừng năm mới",
    ];

    for input in test_cases {
        let result = processor.process_string_safe(input)?;
        println!("Input:  {input}");
        println!("Output: {result}");
        println!();
    }

    Ok(())
}

fn timeout_protection_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Timeout Protection Example ===");

    // Create processor with 100ms timeout
    let processor = SafeAssemblyProcessor::with_timeout(100);

    // Process normal-sized input (should complete)
    let normal_input = "Tiếng Việt ".repeat(100);
    match processor.process_string_safe(&normal_input) {
        Ok(result) => {
            println!(
                "Normal input processed successfully: {} chars -> {} chars",
                normal_input.len(),
                result.len()
            );
        }
        Err(e) => {
            println!("Normal input failed: {e}");
        }
    }

    // Process very large input (may timeout)
    let large_input = "à".repeat(1_000_000);
    match processor.process_string_safe(&large_input) {
        Ok(result) => {
            println!(
                "Large input processed successfully: {} chars -> {} chars",
                large_input.len(),
                result.len()
            );
        }
        Err(e) => {
            println!("Large input handling: {e}");
        }
    }

    Ok(())
}

fn concurrent_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Concurrent Processing Example ===");

    let processor = Arc::new(SafeAssemblyProcessor::new());
    let mut handles = vec![];

    // Spawn multiple threads processing different texts
    let test_texts = vec![
        "Xin chào thế giới",
        "Hôm nay trời đẹp",
        "Cảm ơn bạn rất nhiều",
        "Chúc mừng năm mới",
        "Tiếng Việt rất hay",
    ];

    for (i, text) in test_texts.into_iter().enumerate() {
        let processor_clone = processor.clone();
        let repeated_text = text.repeat(1000); // Make it substantial

        let handle = thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = processor_clone.process_string_safe(&repeated_text);
            let duration = start.elapsed();
            (i, result, duration)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (thread_id, result, duration) = handle.join().unwrap();
        match result {
            Ok(output) => {
                println!(
                    "Thread {}: Processed {} chars in {:?}",
                    thread_id,
                    output.len(),
                    duration
                );
            }
            Err(e) => {
                println!("Thread {thread_id}: Error - {e}");
            }
        }
    }

    Ok(())
}

fn cancellation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Cancellation Example ===");

    let processor = Arc::new(SafeAssemblyProcessor::new());
    let large_input = "à".repeat(100_000);

    // Start processing in background
    let processor_clone = processor.clone();
    let input_clone = large_input;
    let handle = thread::spawn(move || processor_clone.process_string_safe(&input_clone));

    // Let it run for a bit
    thread::sleep(Duration::from_millis(10));

    // Cancel the operation
    println!("Cancelling operation...");
    processor.cancel();

    // Wait for result
    match handle.join().unwrap() {
        Ok(result) => {
            println!(
                "Operation completed before cancellation: {} chars processed",
                result.len()
            );
        }
        Err(e) => {
            println!("Operation cancelled: {e}");
        }
    }

    Ok(())
}

fn metrics_example() {
    println!("\n=== Metrics Example ===");

    let processor = SafeAssemblyProcessor::new();

    // Process various inputs to generate metrics
    let test_inputs = vec![
        "Short".to_string(),
        "Medium length text with Vietnamese: Tiếng Việt".to_string(),
        "à".repeat(1000),
        format!(
            "Much longer text with lots of Vietnamese characters: {}",
            "àáảãạđĐ".repeat(100)
        ),
    ];

    for input in test_inputs {
        let _ = processor.process_string_safe(&input);
    }

    // Display processor-specific metrics
    let metrics = processor.get_metrics();
    println!("Processor Metrics:");
    println!(
        "  Operations started: {}",
        metrics
            .operations_started
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "  Operations completed: {}",
        metrics
            .operations_completed
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!("  Success rate: {:.2}%", metrics.get_success_rate() * 100.0);
    println!(
        "  Average overhead: {} ns",
        metrics.get_average_overhead_ns()
    );

    // Display global metrics
    let global_metrics = &*GLOBAL_SAFETY_METRICS;
    println!("\nGlobal Safety Metrics:");
    println!(
        "  Total operations started: {}",
        global_metrics
            .operations_started
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "  Total operations completed: {}",
        global_metrics
            .operations_completed
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "  Total operations cancelled: {}",
        global_metrics
            .operations_cancelled
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "  Total operations timed out: {}",
        global_metrics
            .operations_timed_out
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "  Global success rate: {:.2}%",
        global_metrics.get_success_rate() * 100.0
    );
}

/// Helper function to safely convert Duration to microseconds as f64
fn duration_as_micros_f64(duration: std::time::Duration) -> f64 {
    // Use saturating conversion to avoid overflow
    let micros = duration.as_micros();
    if micros > u128::from(u64::MAX) {
        u64::MAX as f64
    } else {
        micros as f64
    }
}

/// Helper function to safely convert Duration to nanoseconds as f64
fn duration_as_nanos_f64(duration: std::time::Duration) -> f64 {
    // Use saturating conversion to avoid overflow
    let nanos = duration.as_nanos();
    if nanos > u128::from(u64::MAX) {
        u64::MAX as f64
    } else {
        nanos as f64
    }
}

#[allow(dead_code)]
fn performance_comparison_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Performance Comparison Example ===");

    let safe_processor = SafeAssemblyProcessor::new();

    let test_input = "Tiếng Việt rất hay và đẹp ".repeat(1000);
    let iterations = 100;

    // Benchmark safe processing
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = safe_processor.process_string_safe(&test_input)?;
    }
    let safe_duration = start.elapsed();

    // Benchmark direct processing
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = vi::clean_string(&test_input);
    }
    let direct_duration = start.elapsed();

    println!("Performance Comparison ({iterations} iterations):");
    println!(
        "  Safe processing:   {:?} ({:.2} μs/op)",
        safe_duration,
        duration_as_micros_f64(safe_duration) / f64::from(iterations)
    );
    println!(
        "  Direct processing: {:?} ({:.2} μs/op)",
        direct_duration,
        duration_as_micros_f64(direct_duration) / f64::from(iterations)
    );

    let overhead_percent =
        ((duration_as_nanos_f64(safe_duration) / duration_as_nanos_f64(direct_duration)) - 1.0)
            * 100.0;
    println!("  Safety overhead:   {overhead_percent:.2}%");

    Ok(())
}
