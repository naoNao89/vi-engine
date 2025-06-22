//! Strategy Selection Example
//!
//! This example demonstrates how to control optimization strategy selection
//! in the vi-rust library, including forcing Rust-only processing.

use vi::{AssemblyError, OptimizationStrategy, ProcessorBuilder, VietnameseTextProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VI-RUST Strategy Selection Demo ===\n");

    // Example 1: Automatic selection (default behavior)
    println!("1. Automatic Strategy Selection (Default)");
    demo_automatic_selection()?;

    // Example 2: Force Rust-only processing
    println!("\n2. Force Rust-Only Processing");
    demo_force_rust_only()?;

    // Example 3: Force assembly (if available)
    println!("\n3. Force Assembly Processing");
    demo_force_assembly()?;

    // Example 4: Prefer Rust over assembly
    println!("\n4. Prefer Rust Over Assembly");
    demo_prefer_rust()?;

    // Example 5: Force specific strategy
    println!("\n5. Force Specific Strategy");
    demo_force_specific_strategy()?;

    // Example 6: Strategy availability checking
    println!("\n6. Strategy Availability Checking");
    demo_strategy_availability()?;

    println!("\n✅ Strategy selection demo complete!");
    Ok(())
}

fn demo_automatic_selection() -> Result<(), AssemblyError> {
    let mut processor = VietnameseTextProcessor::new()?;

    let input = "Tiếng Việt tự động";
    let result = processor.process_string(input)?;

    println!("  Input: {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    Ok(())
}

fn demo_force_rust_only() -> Result<(), AssemblyError> {
    // Force Rust-only processing - no assembly optimizations
    let mut processor = ProcessorBuilder::new()
        .force_rust_only()
        .with_monitoring(true)
        .build()?;

    let input = "Chỉ sử dụng Rust";
    let result = processor.process_string(input)?;

    println!("  Input: {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    // Verify it's actually using Rust
    let selected_strategy = processor.selected_strategy();
    match selected_strategy {
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard => {
            println!("  ✅ Confirmed: Using Rust-only processing");
        }
        _ => {
            println!("  ⚠️  Warning: Not using Rust-only processing");
        }
    }

    Ok(())
}

fn demo_force_assembly() -> Result<(), AssemblyError> {
    // Try to force assembly processing
    match ProcessorBuilder::new().force_assembly().build() {
        Ok(mut processor) => {
            let input = "Sử dụng assembly";
            let result = processor.process_string(input)?;

            println!("  Input: {}", input);
            println!("  Output: {}", result);
            println!("  Strategy: {}", processor.processor_name());
            println!("  ✅ Assembly processing available and used");
        }
        Err(e) => {
            println!("  ❌ Assembly processing not available: {}", e);
            println!("  This is normal on platforms without assembly optimizations");
        }
    }

    Ok(())
}

fn demo_prefer_rust() -> Result<(), AssemblyError> {
    // Prefer Rust but allow assembly fallback
    let mut processor = ProcessorBuilder::new().prefer_rust().build()?;

    let input = "Ưu tiên Rust";
    let result = processor.process_string(input)?;

    println!("  Input: {}", input);
    println!("  Output: {}", result);
    println!("  Strategy: {}", processor.processor_name());
    println!("  Preference: {:?}", processor.optimization_preference());

    Ok(())
}

fn demo_force_specific_strategy() -> Result<(), AssemblyError> {
    // Try to force a specific strategy
    let strategies_to_try = vec![
        OptimizationStrategy::RustOptimized,
        OptimizationStrategy::RustStandard,
        OptimizationStrategy::AppleSiliconAssembly,
        OptimizationStrategy::X86_64Assembly,
    ];

    for strategy in strategies_to_try {
        match ProcessorBuilder::new()
            .with_strategy(strategy.clone())
            .build()
        {
            Ok(mut processor) => {
                let input = "Chiến lược cụ thể";
                let result = processor.process_string(input)?;

                println!("  Strategy {:?}:", strategy);
                println!("    Input: {}", input);
                println!("    Output: {}", result);
                println!("    Processor: {}", processor.processor_name());
                println!("    ✅ Available and working");
            }
            Err(e) => {
                println!("  Strategy {:?}:", strategy);
                println!("    ❌ Not available: {}", e);
            }
        }
    }

    Ok(())
}

fn demo_strategy_availability() -> Result<(), AssemblyError> {
    use vi::{CpuCapabilities, OptimizationSelector};

    let selector = OptimizationSelector::get();
    let cpu_info = CpuCapabilities::get();

    println!("  CPU: {}", cpu_info.architecture_description());
    println!("  Performance Tier: {}", cpu_info.performance_description());
    println!("  Available Strategies:");

    for profile in selector.profiles() {
        let status = if profile.available { "✅" } else { "❌" };
        let throughput = profile.estimated_throughput as f64 / 1_000_000.0;

        println!(
            "    {} {:?} ({:.0}M chars/sec)",
            status, profile.strategy, throughput
        );

        if !profile.available {
            if let Some(reason) = &profile.unavailable_reason {
                println!("      Reason: {}", reason);
            }
        }
    }

    println!(
        "  Automatically Selected: {:?}",
        selector.selected_strategy()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vi::OptimizationPreference;

    #[test]
    fn test_rust_only_processing() -> Result<(), AssemblyError> {
        let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

        let result = processor.process_string("Test Rust")?;
        assert_eq!(result, "Test Rust");

        // Verify it's using Rust
        let strategy = processor.selected_strategy();
        assert!(matches!(
            strategy,
            OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
        ));

        Ok(())
    }

    #[test]
    fn test_strategy_preference_setting() -> Result<(), AssemblyError> {
        let processor = ProcessorBuilder::new().force_rust_only().build()?;

        assert_eq!(
            *processor.optimization_preference(),
            OptimizationPreference::ForceRustOnly
        );

        Ok(())
    }

    #[test]
    fn test_specific_strategy_selection() {
        // Test that we can request specific strategies
        let result = ProcessorBuilder::new()
            .with_strategy(OptimizationStrategy::RustOptimized)
            .build();

        // Should succeed since RustOptimized is always available
        assert!(result.is_ok());
    }
}
