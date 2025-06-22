//! Rust-Only Processing Example
//!
//! This example demonstrates how to force the vi-rust library to use
//! only Rust implementations, avoiding assembly optimizations entirely.
//! This is useful for security audits, deployment constraints, or
//! predictable cross-platform behavior.

use vi::{OptimizationStrategy, ProcessorBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VI-RUST Rust-Only Processing Demo ===\n");

    // Create processor with forced Rust-only processing
    let mut processor = ProcessorBuilder::new()
        .force_rust_only()
        .with_monitoring(true)
        .build()?;

    println!("🦀 Processor Configuration:");
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    // Verify we're actually using Rust
    let selected_strategy = processor.selected_strategy();
    match selected_strategy {
        OptimizationStrategy::RustOptimized => {
            println!("  ✅ Using Rust Optimized implementation");
        }
        OptimizationStrategy::RustStandard => {
            println!("  ✅ Using Rust Standard implementation");
        }
        _ => {
            println!("  ⚠️  Warning: Not using Rust-only implementation!");
        }
    }

    println!("\n📝 Processing Vietnamese Text:");

    // Test various Vietnamese text samples
    let test_cases = [
        "Tiếng Việt",
        "Xin chào thế giới",
        "Cảm ơn bạn rất nhiều",
        "Hôm nay trời đẹp quá",
        "Tôi yêu Việt Nam",
        "Đây là ví dụ về xử lý văn bản",
        "Chỉ sử dụng Rust, không có assembly",
        "An toàn và đáng tin cậy",
    ];

    for (i, input) in test_cases.iter().enumerate() {
        let result = processor.process_string(input)?;
        println!("  {}. '{}' → '{}'", i + 1, input, result);
    }

    // Show performance statistics
    println!("\n📊 Performance Statistics (Rust-Only):");
    let stats = processor.stats();
    println!("  Characters processed: {}", stats.total_chars_processed);
    println!("  Strings processed: {}", stats.total_strings_processed);
    println!("  Success rate: {:.1}%", processor.success_rate());
    println!(
        "  Average rate: {:.0} chars/sec",
        processor.avg_processing_rate()
    );
    println!("  Peak rate: {:.0} chars/sec", stats.peak_processing_rate);
    println!(
        "  Optimal performance: {}",
        processor.is_performing_optimally()
    );

    // Demonstrate character-by-character processing
    println!("\n🔤 Character-by-Character Processing:");
    let sample_chars = vec!['à', 'á', 'ả', 'ã', 'ạ', 'ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ'];

    for ch in sample_chars {
        let result = processor.process_char(ch)?;
        println!("  '{}' → '{}'", ch, result);
    }

    // Show final optimization info
    println!("\n🔍 Final Optimization Info:");
    println!("{}", processor.optimization_info());

    println!("\n✅ Rust-only processing demo complete!");
    println!("\n💡 Benefits of Rust-only processing:");
    println!("  • Security: No assembly code to audit");
    println!("  • Portability: Works on any Rust-supported platform");
    println!("  • Predictability: Consistent behavior across architectures");
    println!("  • Debugging: Easier to debug and profile");
    println!("  • Compliance: Meets strict deployment requirements");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vi::{AssemblyError, OptimizationPreference};

    #[test]
    fn test_rust_only_configuration() -> Result<(), AssemblyError> {
        let processor = ProcessorBuilder::new().force_rust_only().build()?;

        // Verify configuration
        assert_eq!(
            *processor.optimization_preference(),
            OptimizationPreference::ForceRustOnly
        );

        // Verify strategy
        let strategy = processor.selected_strategy();
        assert!(matches!(
            strategy,
            OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
        ));

        Ok(())
    }

    #[test]
    fn test_rust_only_processing() -> Result<(), AssemblyError> {
        let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

        // Test string processing
        let result = processor.process_string("Tiếng Việt")?;
        assert_eq!(result, "Tieng Viet");

        // Test character processing
        let result = processor.process_char('ế')?;
        assert_eq!(result, 'e');

        Ok(())
    }

    #[test]
    fn test_rust_only_performance() -> Result<(), AssemblyError> {
        let mut processor = ProcessorBuilder::new()
            .force_rust_only()
            .with_monitoring(true)
            .build()?;

        // Process some text to generate statistics
        let test_text = "Đây là một đoạn văn bản để kiểm tra hiệu suất Rust";
        let _result = processor.process_string(test_text)?;

        // Verify statistics are collected
        let stats = processor.stats();
        assert!(stats.total_chars_processed > 0);
        assert!(stats.total_strings_processed > 0);
        assert!(stats.successful_operations > 0);
        assert_eq!(stats.failed_operations, 0);

        // Should have good success rate
        assert!(processor.success_rate() >= 100.0);

        Ok(())
    }

    #[test]
    fn test_rust_only_vs_auto_selection() -> Result<(), AssemblyError> {
        // Create both processors
        let mut rust_only = ProcessorBuilder::new().force_rust_only().build()?;

        let mut auto_select = VietnameseTextProcessor::new()?;

        // Both should produce the same results
        let test_text = "Văn bản kiểm tra";
        let rust_result = rust_only.process_string(test_text)?;
        let auto_result = auto_select.process_string(test_text)?;

        assert_eq!(rust_result, auto_result);
        assert_eq!(rust_result, "Van ban kiem tra");

        Ok(())
    }
}
