//! Production Usage Example
//!
//! This example demonstrates how to use the vi-rust library in production
//! applications with automatic optimization selection and comprehensive
//! error handling.

use std::time::Instant;
use vi::{
    AssemblyError, CpuCapabilities, OptimizationSelector, ProcessorBuilder, VietnameseTextProcessor,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VI-RUST Production Usage Example ===\n");

    // Display system information
    display_system_info();

    // Example 1: Simple usage with default configuration
    println!("\n1. Simple Usage (Default Configuration)");
    simple_usage_example()?;

    // Example 2: Advanced configuration
    println!("\n2. Advanced Configuration");
    advanced_configuration_example()?;

    // Example 3: Performance monitoring
    println!("\n3. Performance Monitoring");
    performance_monitoring_example()?;

    // Example 4: Error handling and fallback
    println!("\n4. Error Handling and Fallback");
    error_handling_example()?;

    // Example 5: Batch processing
    println!("\n5. Batch Processing");
    batch_processing_example()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

fn display_system_info() {
    println!("System Information:");

    let cpu_capabilities = CpuCapabilities::get();
    println!("  CPU: {}", cpu_capabilities.architecture_description());
    println!(
        "  Performance: {}",
        cpu_capabilities.performance_description()
    );
    println!("  Score: {}", cpu_capabilities.performance_score);

    let optimization_info = OptimizationSelector::get().optimization_summary();
    println!("  Optimization: {optimization_info}");

    println!("  Available Features:");
    for (feature, available) in &cpu_capabilities.features {
        if *available {
            println!("    ✓ {feature}");
        }
    }
}

fn simple_usage_example() -> Result<(), AssemblyError> {
    println!("Creating processor with default settings...");

    let mut processor = VietnameseTextProcessor::new()?;

    // Process some Vietnamese text
    let test_cases = vec![
        "Tiếng Việt",
        "Xin chào",
        "Cảm ơn bạn",
        "Hẹn gặp lại",
        "Chúc mừng năm mới",
    ];

    println!("Processing Vietnamese text:");
    for input in test_cases {
        let result = processor.process_string(input)?;
        println!("  '{input}' → '{result}'");
    }

    println!("Processor: {}", processor.processor_name());

    Ok(())
}

fn advanced_configuration_example() -> Result<(), AssemblyError> {
    println!("Creating processor with custom configuration...");

    let mut processor = ProcessorBuilder::new()
        .with_timeout(10000) // 10 second timeout
        .with_monitoring(true) // Enable performance monitoring
        .with_fallback(true) // Enable automatic fallback
        .with_max_retries(3) // Allow up to 3 retries
        .build()?;

    // Process text with monitoring
    let input = "Đây là một ví dụ về xử lý văn bản tiếng Việt với cấu hình nâng cao";
    let result = processor.process_string(input)?;

    println!("Input:  {input}");
    println!("Output: {result}");

    // Display configuration
    let config = processor.config();
    println!("Configuration:");
    println!("  Monitoring: {}", config.enable_monitoring);
    println!("  Timeout: {}ms", config.operation_timeout_ms);
    println!("  Fallback: {}", config.enable_fallback);
    println!("  Max Retries: {}", config.max_retries);

    Ok(())
}

fn performance_monitoring_example() -> Result<(), AssemblyError> {
    println!("Demonstrating performance monitoring...");

    let mut processor = ProcessorBuilder::new().with_monitoring(true).build()?;

    // Process various amounts of text to generate statistics
    let test_texts = [
        "Một",
        "Hai ba bốn năm",
        "Sáu bảy tám chín mười mười một mười hai",
        "Đây là một đoạn văn bản dài hơn để kiểm tra hiệu suất xử lý của thư viện vi-rust",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    ];

    let start_time = Instant::now();

    for (i, text) in test_texts.iter().enumerate() {
        let _result = processor.process_string(text)?;
        println!(
            "  Processed text {} ({} chars)",
            i + 1,
            text.chars().count()
        );
    }

    let total_time = start_time.elapsed();

    // Display statistics
    let stats = processor.stats();
    println!("\nPerformance Statistics:");
    println!(
        "  Total characters processed: {}",
        stats.total_chars_processed
    );
    println!(
        "  Total strings processed: {}",
        stats.total_strings_processed
    );
    println!("  Success rate: {:.1}%", processor.success_rate());
    println!(
        "  Average processing rate: {:.0} chars/sec",
        processor.avg_processing_rate()
    );
    println!(
        "  Peak processing rate: {:.0} chars/sec",
        stats.peak_processing_rate
    );
    println!(
        "  Average time per char: {:.1} ns",
        stats.avg_time_per_char_ns
    );
    println!("  Total processing time: {:.2} ms", total_time.as_millis());
    println!(
        "  Performing optimally: {}",
        processor.is_performing_optimally()
    );

    Ok(())
}

fn error_handling_example() -> Result<(), AssemblyError> {
    println!("Demonstrating error handling and fallback...");

    let mut processor = ProcessorBuilder::new()
        .with_fallback(true)
        .with_max_retries(2)
        .build()?;

    // Process normal text (should succeed)
    let normal_text = "Văn bản bình thường";
    match processor.process_string(normal_text) {
        Ok(result) => println!("  Success: '{normal_text}' → '{result}'"),
        Err(e) => println!("  Error: {e}"),
    }

    // Process empty string (should handle gracefully)
    let empty_text = "";
    match processor.process_string(empty_text) {
        Ok(result) => println!("  Empty string handled: '{result}'"),
        Err(e) => println!("  Error with empty string: {e}"),
    }

    // Display fallback statistics
    let stats = processor.stats();
    if stats.fallback_operations > 0 {
        println!("  Fallback operations: {}", stats.fallback_operations);
    } else {
        println!("  No fallback operations needed");
    }

    Ok(())
}

fn batch_processing_example() -> Result<(), AssemblyError> {
    println!("Demonstrating batch processing...");

    let mut processor = ProcessorBuilder::new().with_monitoring(true).build()?;

    // Simulate processing a batch of Vietnamese text
    let batch_data = vec![
        "Hà Nội",
        "Thành phố Hồ Chí Minh",
        "Đà Nẵng",
        "Hải Phòng",
        "Cần Thơ",
        "Biên Hòa",
        "Huế",
        "Nha Trang",
        "Buôn Ma Thuột",
        "Quy Nhon",
    ];

    println!(
        "Processing batch of {} Vietnamese city names:",
        batch_data.len()
    );

    let start_time = Instant::now();
    let mut results = Vec::new();

    for city in &batch_data {
        let processed = processor.process_string(city)?;
        results.push(processed);
    }

    let batch_time = start_time.elapsed();

    // Display results
    for (original, processed) in batch_data.iter().zip(results.iter()) {
        println!("  '{original}' → '{processed}'");
    }

    // Display batch statistics
    println!("\nBatch Processing Statistics:");
    println!("  Items processed: {}", batch_data.len());
    println!("  Total time: {:.2} ms", batch_time.as_millis());
    println!("  Average time per item: {:.2} ms", {
        let millis = batch_time.as_millis();
        let millis_f64 = if millis > (1u64 << 53) as u128 {
            (1u64 << 53) as f64
        } else {
            millis as f64
        };
        let len_f64 = if batch_data.len() > (1u64 << 53) as usize {
            (1u64 << 53) as f64
        } else {
            batch_data.len() as f64
        };
        millis_f64 / len_f64
    });

    let total_chars: usize = batch_data.iter().map(|s| s.chars().count()).sum();
    let chars_per_sec = {
        let chars_f64 = if total_chars > (1u64 << 53) as usize {
            (1u64 << 53) as f64
        } else {
            total_chars as f64
        };
        chars_f64 / batch_time.as_secs_f64()
    };
    println!("  Batch processing rate: {chars_per_sec:.0} chars/sec");

    Ok(())
}
