//! Simple Force Rust Example
//!
//! This is the simplest example showing how to force the vi-rust library
//! to use only Rust implementations, avoiding assembly optimizations entirely.

use vi::ProcessorBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple Force Rust Example ===\n");

    // Create a processor that only uses Rust (no assembly)
    let mut processor = ProcessorBuilder::new()
        .force_rust_only() // This is the key line!
        .build()?;

    // Process Vietnamese text
    let vietnamese_texts = vec![
        "Tiếng Việt",
        "Xin chào",
        "Cảm ơn bạn",
        "Hẹn gặp lại",
        "Chúc mừng năm mới",
    ];

    println!("🦀 Processing with Rust-only (no assembly):");
    for text in vietnamese_texts {
        let result = processor.process_string(text)?;
        println!("  '{text}' → '{result}'");
    }

    // Verify we're using Rust
    println!("\n✅ Verification:");
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    println!("\n💡 Why force Rust-only?");
    println!("  • Security audits: No assembly code to review");
    println!("  • Deployment: Works in environments that restrict assembly");
    println!("  • Predictability: Same behavior on all platforms");
    println!("  • Debugging: Easier to debug and profile");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vi::{AssemblyError, OptimizationPreference, OptimizationStrategy};

    #[test]
    fn test_simple_force_rust() -> Result<(), AssemblyError> {
        let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

        // Test basic functionality
        let result = processor.process_string("Tiếng Việt")?;
        assert_eq!(result, "Tieng Viet");

        // Verify it's using Rust
        let strategy = processor.selected_strategy();
        assert!(matches!(
            strategy,
            OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
        ));

        // Verify preference
        assert_eq!(
            *processor.optimization_preference(),
            OptimizationPreference::ForceRustOnly
        );

        Ok(())
    }
}
