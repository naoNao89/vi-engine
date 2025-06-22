//! Force Rust Example - Complete Guide to Rust-Only Processing
//!
//! This example demonstrates various ways to force the vi-rust library to use
//! only Rust implementations, completely avoiding assembly optimizations.
//! Perfect for security audits, deployment constraints, or predictable behavior.

use vi::{
    AssemblyError, OptimizationPreference, OptimizationStrategy, ProcessorBuilder,
    VietnameseTextProcessor,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VI-RUST Force Rust Example ===\n");

    // Method 1: Simple force Rust-only
    println!("1. Simple Force Rust-Only");
    simple_force_rust()?;

    // Method 2: Force Rust with configuration
    println!("\n2. Force Rust with Advanced Configuration");
    force_rust_with_config()?;

    // Method 3: Force specific Rust strategy
    println!("\n3. Force Specific Rust Strategy");
    force_specific_rust_strategy()?;

    // Method 4: Prefer Rust over assembly
    println!("\n4. Prefer Rust Over Assembly");
    prefer_rust_example()?;

    // Method 5: Verification and validation
    println!("\n5. Strategy Verification");
    verify_rust_only()?;

    // Method 6: Performance comparison
    println!("\n6. Performance Comparison");
    performance_comparison()?;

    // Method 7: Production deployment example
    println!("\n7. Production Deployment Example");
    production_deployment_example()?;

    println!("\n✅ Force Rust examples complete!");
    Ok(())
}

/// Method 1: Simplest way to force Rust-only processing
fn simple_force_rust() -> Result<(), AssemblyError> {
    println!("  Creating Rust-only processor...");

    // This is the simplest way to force Rust-only processing
    let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

    // Process Vietnamese text
    let input = "Tiếng Việt đơn giản";
    let result = processor.process_string(input)?;

    println!("  Input:  {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());

    // Verify it's actually using Rust
    match processor.selected_strategy() {
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard => {
            println!("  ✅ Confirmed: Using Rust-only processing");
        }
        _ => {
            println!("  ❌ Error: Not using Rust-only processing!");
        }
    }

    Ok(())
}

/// Method 2: Force Rust with additional configuration
fn force_rust_with_config() -> Result<(), AssemblyError> {
    println!("  Creating configured Rust-only processor...");

    let mut processor = ProcessorBuilder::new()
        .force_rust_only() // Force Rust-only
        .with_monitoring(true) // Enable performance monitoring
        .with_timeout(10000) // 10 second timeout
        .with_fallback(true) // Enable fallback (though not needed for Rust)
        .with_max_retries(2) // Allow retries
        .build()?;

    let input = "Cấu hình nâng cao cho Rust";
    let result = processor.process_string(input)?;

    println!("  Input:  {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    // Show configuration
    let config = processor.config();
    println!("  Configuration:");
    println!("    Monitoring: {}", config.enable_monitoring);
    println!("    Timeout: {}ms", config.operation_timeout_ms);
    println!("    Fallback: {}", config.enable_fallback);
    println!("    Max Retries: {}", config.max_retries);

    Ok(())
}

/// Method 3: Force specific Rust strategy
fn force_specific_rust_strategy() -> Result<(), AssemblyError> {
    println!("  Testing specific Rust strategies...");

    // Force RustOptimized specifically
    let mut processor_optimized = ProcessorBuilder::new()
        .with_strategy(OptimizationStrategy::RustOptimized)
        .build()?;

    // Force RustStandard specifically
    let mut processor_standard = ProcessorBuilder::new()
        .with_strategy(OptimizationStrategy::RustStandard)
        .build()?;

    let input = "Chiến lược Rust cụ thể";

    // Test RustOptimized
    let result_optimized = processor_optimized.process_string(input)?;
    println!("  RustOptimized:");
    println!("    Input:  {}", input);
    println!("    Output: {}", result_optimized);
    println!("    Strategy: {}", processor_optimized.processor_name());

    // Test RustStandard
    let result_standard = processor_standard.process_string(input)?;
    println!("  RustStandard:");
    println!("    Input:  {}", input);
    println!("    Output: {}", result_standard);
    println!("    Strategy: {}", processor_standard.processor_name());

    // Both should produce the same result
    assert_eq!(result_optimized, result_standard);
    println!("  ✅ Both strategies produce identical results");

    Ok(())
}

/// Method 4: Prefer Rust but allow fallback
fn prefer_rust_example() -> Result<(), AssemblyError> {
    println!("  Creating Rust-preferring processor...");

    let mut processor = ProcessorBuilder::new()
        .prefer_rust() // Prefer Rust but allow assembly fallback
        .build()?;

    let input = "Ưu tiên sử dụng Rust";
    let result = processor.process_string(input)?;

    println!("  Input:  {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    // Check what was actually selected
    match processor.selected_strategy() {
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard => {
            println!("  ✅ Rust preference honored");
        }
        _ => {
            println!("  ℹ️  Assembly was selected (Rust not available or failed)");
        }
    }

    Ok(())
}

/// Method 5: Verify Rust-only processing
fn verify_rust_only() -> Result<(), AssemblyError> {
    println!("  Verifying Rust-only processing...");

    let mut processor = ProcessorBuilder::new()
        .force_rust_only()
        .with_monitoring(true)
        .build()?;

    // Process multiple strings to generate statistics
    let test_cases = vec![
        "Kiểm tra xác minh",
        "Chỉ sử dụng Rust",
        "Không có assembly",
        "An toàn và đáng tin cậy",
    ];

    for input in &test_cases {
        let result = processor.process_string(input)?;
        println!("    '{}' → '{}'", input, result);
    }

    // Verify strategy
    let strategy = processor.selected_strategy();
    println!("  Selected Strategy: {:?}", strategy);

    // Verify preference
    let preference = processor.optimization_preference();
    println!("  Optimization Preference: {:?}", preference);

    // Check performance stats
    let stats = processor.stats();
    println!("  Performance Stats:");
    println!("    Characters processed: {}", stats.total_chars_processed);
    println!("    Success rate: {:.1}%", processor.success_rate());
    println!(
        "    Average rate: {:.0} chars/sec",
        processor.avg_processing_rate()
    );

    // Verify it's definitely Rust
    assert!(matches!(
        strategy,
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
    ));
    assert_eq!(*preference, OptimizationPreference::ForceRustOnly);

    println!("  ✅ All verifications passed - definitely using Rust-only");

    Ok(())
}

/// Method 6: Performance comparison between strategies
fn performance_comparison() -> Result<(), AssemblyError> {
    println!("  Comparing performance between strategies...");

    let test_text = "So sánh hiệu suất giữa các chiến lược khác nhau";

    // Test automatic selection
    let mut auto_processor = VietnameseTextProcessor::new()?;
    let auto_result = auto_processor.process_string(test_text)?;

    // Test forced Rust
    let mut rust_processor = ProcessorBuilder::new()
        .force_rust_only()
        .with_monitoring(true)
        .build()?;
    let rust_result = rust_processor.process_string(test_text)?;

    println!("  Test text: {}", test_text);
    println!("  Automatic selection:");
    println!("    Strategy: {}", auto_processor.processor_name());
    println!("    Result: {}", auto_result);

    println!("  Forced Rust:");
    println!("    Strategy: {}", rust_processor.processor_name());
    println!("    Result: {}", rust_result);
    println!(
        "    Rate: {:.0} chars/sec",
        rust_processor.avg_processing_rate()
    );

    // Results should be identical regardless of strategy
    assert_eq!(auto_result, rust_result);
    println!("  ✅ Results are identical across strategies");

    Ok(())
}

/// Method 7: Production deployment example
fn production_deployment_example() -> Result<(), AssemblyError> {
    println!("  Production deployment with Rust-only...");

    // Simulate production configuration
    let mut processor = ProcessorBuilder::new()
        .force_rust_only() // Security requirement: no assembly
        .with_monitoring(true) // Production monitoring
        .with_timeout(5000) // 5 second timeout for production
        .with_fallback(true) // Enable fallback for reliability
        .with_max_retries(3) // Allow retries for production resilience
        .build()?;

    // Simulate production workload
    let production_texts = [
        "Xử lý văn bản trong môi trường sản xuất",
        "Đảm bảo an toàn và bảo mật",
        "Không sử dụng mã assembly",
        "Hiệu suất ổn định và đáng tin cậy",
    ];

    println!("  Processing production workload...");
    for (i, text) in production_texts.iter().enumerate() {
        let result = processor.process_string(text)?;
        println!("    {}. '{}' → '{}'", i + 1, text, result);
    }

    // Production health check
    println!("  Production Health Check:");
    println!("    Strategy: {}", processor.processor_name());
    println!("    Preference: {:?}", processor.optimization_preference());
    println!("    Success Rate: {:.1}%", processor.success_rate());
    println!(
        "    Optimal Performance: {}",
        processor.is_performing_optimally()
    );

    // Verify production requirements
    let strategy = processor.selected_strategy();
    assert!(matches!(
        strategy,
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
    ));

    println!("  ✅ Production deployment verified - Rust-only processing active");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_rust_methods() -> Result<(), AssemblyError> {
        // Test all methods of forcing Rust

        // Method 1: force_rust_only()
        let processor1 = ProcessorBuilder::new().force_rust_only().build()?;
        assert!(matches!(
            processor1.selected_strategy(),
            OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
        ));

        // Method 2: with_strategy(RustOptimized)
        let processor2 = ProcessorBuilder::new()
            .with_strategy(OptimizationStrategy::RustOptimized)
            .build()?;
        assert_eq!(
            processor2.selected_strategy(),
            OptimizationStrategy::RustOptimized
        );

        // Method 3: with_strategy(RustStandard)
        let processor3 = ProcessorBuilder::new()
            .with_strategy(OptimizationStrategy::RustStandard)
            .build()?;
        assert_eq!(
            processor3.selected_strategy(),
            OptimizationStrategy::RustStandard
        );

        Ok(())
    }

    #[test]
    fn test_rust_processing_consistency() -> Result<(), AssemblyError> {
        let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

        // Test that results are consistent
        let test_cases = vec![
            ("Tiếng Việt", "Tieng Viet"),
            ("Xin chào", "Xin chao"),
            ("Cảm ơn", "Cam on"),
        ];

        for (input, expected) in test_cases {
            let result = processor.process_string(input)?;
            assert_eq!(result, expected);
        }

        Ok(())
    }

    #[test]
    fn test_rust_only_preference() -> Result<(), AssemblyError> {
        let processor = ProcessorBuilder::new().force_rust_only().build()?;

        assert_eq!(
            *processor.optimization_preference(),
            OptimizationPreference::ForceRustOnly
        );

        Ok(())
    }
}
