//! Rust Community Standards Demo
//!
//! This example demonstrates the Rust community standards implemented
//! in the vi-rust library, including:
//! - `#[non_exhaustive]` enums for future-proof APIs
//! - `#[must_use]` attributes for important return values
//! - Optimized syllable types for different use cases

use vi::{ComplexSyllable, OptimizationPreference, ProcessorBuilder, SimpleSyllable, Syllable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Rust Community Standards Demo");
    println!("=================================\n");

    // Demonstrate builder pattern with #[must_use] attributes
    demonstrate_builder_pattern()?;

    // Demonstrate different syllable types
    demonstrate_syllable_types();

    // Demonstrate optimization preferences with #[non_exhaustive] enums
    demonstrate_optimization_preferences()?;

    println!("\n✅ All demonstrations completed successfully!");
    Ok(())
}

/// Demonstrates the builder pattern with #[must_use] attributes
fn demonstrate_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Builder Pattern with #[must_use] Attributes");
    println!("----------------------------------------------");

    // The ProcessorBuilder has #[must_use] attribute, so forgetting .build() would warn
    let mut processor = ProcessorBuilder::new()
        .with_monitoring(true)
        .with_timeout(5000)
        .with_fallback(true)
        .force_rust_only() // This also has #[must_use]
        .build()?; // This has #[must_use] too

    // Process some text
    let input_text = "Tiếng Việt rất đẹp!";
    let result = processor.process_string(input_text)?;
    println!("Processed: '{}' -> '{}'", input_text, result);

    // Get optimization info (has #[must_use])
    let info = processor.optimization_info();
    println!("Optimization: {}", info);

    // Get statistics (has #[must_use])
    let stats = processor.stats();
    println!("Characters processed: {}", stats.total_chars_processed);

    println!();
    Ok(())
}

/// Demonstrates different syllable types optimized for different use cases
fn demonstrate_syllable_types() {
    println!("🔤 Optimized Syllable Types");
    println!("---------------------------");

    // Standard syllable (inline capacity: 2 modifications)
    let standard = Syllable::new("tiếng");
    println!("Standard syllable: {} (len: {})", standard, standard.len());

    // Simple syllable (inline capacity: 1 modification) - for basic words
    let simple = SimpleSyllable::new("an");
    println!("Simple syllable: {} (len: {})", simple, simple.len());

    // Complex syllable (inline capacity: 4 modifications) - for complex words
    let complex = ComplexSyllable::new("nghiêng");
    println!("Complex syllable: {} (len: {})", complex, complex.len());

    println!("📊 Memory optimization: Different syllable types use different");
    println!("   SmallVec capacities to minimize heap allocations based on usage.");

    println!();
}

/// Demonstrates optimization preferences with #[non_exhaustive] enums
fn demonstrate_optimization_preferences() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Optimization Preferences (#[non_exhaustive] Enums)");
    println!("----------------------------------------------------");

    // These enums are marked #[non_exhaustive] for future extensibility
    let preferences = vec![
        ("Auto", OptimizationPreference::Auto),
        ("Force Rust Only", OptimizationPreference::ForceRustOnly),
        ("Prefer Rust", OptimizationPreference::PreferRust),
        ("Prefer Assembly", OptimizationPreference::PreferAssembly),
    ];

    for (name, preference) in preferences {
        match ProcessorBuilder::new()
            .with_optimization_preference(preference)
            .build()
        {
            Ok(mut processor) => {
                let test_text = "Xin chào";
                let result = processor.process_string(test_text)?;
                println!("{}: '{}' -> '{}'", name, test_text, result);
            }
            Err(e) => {
                println!("{}: Error - {}", name, e);
            }
        }
    }

    println!("\n🔮 Future-proof: The #[non_exhaustive] attribute allows adding");
    println!("   new optimization strategies without breaking existing code.");

    println!();
    Ok(())
}
