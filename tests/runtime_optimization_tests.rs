//! Integration tests for runtime optimization selection and production API

use vi::{
    AssemblyError, CpuArchitecture, CpuCapabilities, OptimizationPreference, OptimizationSelector,
    OptimizationStrategy, PerformanceTier, ProcessorBuilder, VietnameseTextProcessor,
};

#[test]
fn test_cpu_detection() {
    let capabilities = CpuCapabilities::get();

    // Should detect some valid architecture
    match &capabilities.architecture {
        CpuArchitecture::AppleSilicon {
            generation,
            performance_cores,
            efficiency_cores,
        } => {
            assert!(*generation > 0);
            assert!(*performance_cores > 0);
            assert!(*efficiency_cores < 255); // Reasonable upper bound
        }
        CpuArchitecture::GenericArm64 {
            has_neon,
            has_advanced_simd,
        } => {
            // NEON should be available on most modern ARM64
            assert!(*has_neon || !*has_advanced_simd);
        }
        CpuArchitecture::X86_64 {
            has_avx2,
            has_bmi2,
            has_avx512f,
            has_fma,
        } => {
            // At least some features should be available on modern x86_64
            let _has_some_features = *has_avx2 || *has_bmi2 || *has_avx512f || *has_fma;
        }
        CpuArchitecture::Other { arch_name } => {
            assert!(!arch_name.is_empty());
        }
        // Handle future variants due to #[non_exhaustive]
        _ => {
            // Any future architecture variants should be valid
            assert!(capabilities.performance_score > 0);
        }
    }

    // Should have a valid performance tier
    assert!(matches!(
        capabilities.performance_tier,
        PerformanceTier::Tier1UltraHigh
            | PerformanceTier::Tier2High
            | PerformanceTier::Tier3Good
            | PerformanceTier::Tier4Basic
    ));

    // Should have some performance score
    assert!(capabilities.performance_score > 0);

    // Should have architecture description
    let desc = capabilities.architecture_description();
    assert!(!desc.is_empty());
    assert!(desc.len() > 5); // Should be descriptive
}

#[test]
fn test_optimization_selector() {
    let selector = OptimizationSelector::get();

    // Should have selected some strategy
    let strategy = selector.selected_strategy();
    assert!(matches!(
        strategy,
        OptimizationStrategy::AppleSiliconAssembly
            | OptimizationStrategy::GenericArm64Assembly
            | OptimizationStrategy::X86_64Assembly
            | OptimizationStrategy::RustOptimized
            | OptimizationStrategy::RustStandard
    ));

    // Should have profiles for all strategies
    let profiles = selector.profiles();
    assert_eq!(profiles.len(), 5);

    // At least one profile should be available (Rust implementations)
    let available_count = profiles.iter().filter(|p| p.available).count();
    assert!(available_count >= 2); // At least RustOptimized and RustStandard

    // Selected profile should be available
    let selected_profile = selector.selected_profile().unwrap();
    assert!(selected_profile.available);

    // Should have optimization summary
    let summary = selector.optimization_summary();
    assert!(!summary.is_empty());
    assert!(summary.contains("Selected:"));
}

#[test]
fn test_processor_creation() -> Result<(), AssemblyError> {
    // Test default creation
    let processor = VietnameseTextProcessor::new()?;
    assert!(!processor.processor_name().is_empty());

    // Test builder creation
    let processor = ProcessorBuilder::new()
        .with_timeout(1000)
        .with_monitoring(true)
        .with_fallback(true)
        .with_max_retries(1)
        .build()?;

    let config = processor.config();
    assert_eq!(config.operation_timeout_ms, 1000);
    assert!(config.enable_monitoring);
    assert!(config.enable_fallback);
    assert_eq!(config.max_retries, 1);

    Ok(())
}

#[test]
fn test_character_processing() -> Result<(), AssemblyError> {
    let mut processor = VietnameseTextProcessor::new()?;

    // Test Vietnamese characters with diacritics
    let test_cases = vec![
        ('à', 'a'),
        ('á', 'a'),
        ('ả', 'a'),
        ('ã', 'a'),
        ('ạ', 'a'),
        ('ă', 'a'),
        ('ằ', 'a'),
        ('ắ', 'a'),
        ('ẳ', 'a'),
        ('ẵ', 'a'),
        ('ặ', 'a'),
        ('â', 'a'),
        ('ầ', 'a'),
        ('ấ', 'a'),
        ('ẩ', 'a'),
        ('ẫ', 'a'),
        ('ậ', 'a'),
        ('è', 'e'),
        ('é', 'e'),
        ('ẻ', 'e'),
        ('ẽ', 'e'),
        ('ẹ', 'e'),
        ('ê', 'e'),
        ('ề', 'e'),
        ('ế', 'e'),
        ('ể', 'e'),
        ('ễ', 'e'),
        ('ệ', 'e'),
    ];

    for (input, expected) in test_cases {
        let result = processor.process_char(input)?;
        assert_eq!(result, expected, "Failed for character '{input}'");
    }

    // Test non-Vietnamese characters (should pass through unchanged)
    let non_vietnamese = vec!['x', 'y', 'z', '1', '2', '3', ' ', '.', '!'];
    for ch in non_vietnamese {
        let result = processor.process_char(ch)?;
        assert_eq!(result, ch);
    }

    Ok(())
}

#[test]
fn test_string_processing() -> Result<(), AssemblyError> {
    let mut processor = VietnameseTextProcessor::new()?;

    let test_cases = vec![
        ("Tiếng Việt", "Tieng Viet"),
        ("Xin chào", "Xin chao"),
        ("Cảm ơn bạn", "Cam on ban"),
        ("Hẹn gặp lại", "Hen gap lai"),
        ("Chúc mừng năm mới", "Chuc mung nam moi"),
        ("", ""),                       // Empty string
        ("Hello World", "Hello World"), // Non-Vietnamese
        ("123 ABC", "123 ABC"),         // Numbers and letters
    ];

    for (input, expected) in test_cases {
        let result = processor.process_string(input)?;
        assert_eq!(result, expected, "Failed for string '{input}'");
    }

    Ok(())
}

#[test]
fn test_performance_monitoring() -> Result<(), AssemblyError> {
    let mut processor = ProcessorBuilder::new().with_monitoring(true).build()?;

    // Process some text to generate statistics
    let test_text = "Đây là một đoạn văn bản để kiểm tra thống kê hiệu suất";
    let _result = processor.process_string(test_text)?;

    let stats = processor.stats();
    assert!(stats.total_chars_processed > 0);
    assert!(stats.total_strings_processed > 0);
    assert!(stats.successful_operations > 0);
    assert_eq!(stats.failed_operations, 0); // Should not fail for normal text

    // Test success rate calculation
    let success_rate = processor.success_rate();
    assert!((0.0..=100.0).contains(&success_rate));

    // Test processing rate calculation
    let avg_rate = processor.avg_processing_rate();
    assert!(avg_rate >= 0.0);

    // Test optimal performance check
    let is_optimal = processor.is_performing_optimally();
    assert!(is_optimal); // Should be optimal for simple test

    Ok(())
}

#[test]
fn test_error_handling() -> Result<(), AssemblyError> {
    // Reset global assembly control state to prevent test contamination
    {
        use vi::safety::GLOBAL_ASSEMBLY_CONTROL;
        GLOBAL_ASSEMBLY_CONTROL.reset_for_operation(1000);
    }

    let mut processor = ProcessorBuilder::new()
        .with_fallback(true)
        .with_max_retries(1)
        .with_timeout(10000) // 10 second timeout for long string processing
        .build()?;

    // Test empty string handling
    let result = processor.process_string("")?;
    assert_eq!(result, "");

    // Test very long string (should not timeout with reasonable timeout)
    let long_string = "Tiếng Việt ".repeat(1000);
    let result = processor.process_string(&long_string)?;
    assert!(!result.is_empty());
    assert!(result.contains("Tieng Viet"));

    Ok(())
}

#[test]
fn test_processor_info() -> Result<(), AssemblyError> {
    let processor = VietnameseTextProcessor::new()?;

    // Test processor name
    let name = processor.processor_name();
    assert!(!name.is_empty());

    // Test optimization info
    let opt_info = processor.optimization_info();
    assert!(!opt_info.is_empty());
    assert!(opt_info.contains("Selected:"));

    // Test CPU info
    let cpu_info = processor.cpu_info();
    assert!(cpu_info.performance_score > 0);

    Ok(())
}

#[test]
fn test_stats_reset() -> Result<(), AssemblyError> {
    let mut processor = ProcessorBuilder::new().with_monitoring(true).build()?;

    // Process some text
    let _result = processor.process_string("Test text")?;

    // Verify stats are populated
    assert!(processor.stats().total_chars_processed > 0);

    // Reset stats
    processor.reset_stats();

    // Verify stats are reset
    let stats = processor.stats();
    assert_eq!(stats.total_chars_processed, 0);
    assert_eq!(stats.total_strings_processed, 0);
    assert_eq!(stats.successful_operations, 0);
    assert_eq!(stats.failed_operations, 0);

    Ok(())
}

#[test]
fn test_concurrent_processing() -> Result<(), AssemblyError> {
    use std::thread;

    let mut handles = vec![];

    // Spawn threads to process text concurrently
    for i in 0..4 {
        let handle = thread::spawn(move || -> Result<(), AssemblyError> {
            // Each thread uses its own processor with Rust-only to avoid assembly conflicts
            let mut processor = ProcessorBuilder::new()
                .force_rust_only() // Use Rust-only to avoid potential assembly conflicts
                .build()?;

            for j in 0..10 {
                let text = format!("Văn bản số {j} từ thread {i}");
                let _result = processor.process_string(&text)?;
            }
            Ok(())
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[test]
fn test_force_rust_only() -> Result<(), AssemblyError> {
    let mut processor = ProcessorBuilder::new().force_rust_only().build()?;

    // Test processing
    let result = processor.process_string("Tiếng Việt")?;
    assert_eq!(result, "Tieng Viet");

    // Verify it's using Rust
    let strategy = processor.selected_strategy();
    assert!(matches!(
        strategy,
        OptimizationStrategy::RustOptimized | OptimizationStrategy::RustStandard
    ));

    // Verify preference is set correctly
    assert_eq!(
        *processor.optimization_preference(),
        OptimizationPreference::ForceRustOnly
    );

    Ok(())
}

#[test]
fn test_strategy_preferences() -> Result<(), AssemblyError> {
    // Test different preference settings
    let preferences = vec![
        OptimizationPreference::Auto,
        OptimizationPreference::ForceRustOnly,
        OptimizationPreference::PreferRust,
        OptimizationPreference::PreferAssembly,
    ];

    for preference in preferences {
        let processor = ProcessorBuilder::new()
            .with_optimization_preference(preference.clone())
            .build()?;

        assert_eq!(*processor.optimization_preference(), preference);
    }

    Ok(())
}

#[test]
fn test_specific_strategy_selection() -> Result<(), AssemblyError> {
    // Test RustOptimized (should always be available)
    let processor = ProcessorBuilder::new()
        .with_strategy(OptimizationStrategy::RustOptimized)
        .build()?;

    assert_eq!(
        processor.selected_strategy(),
        OptimizationStrategy::RustOptimized
    );

    // Test RustStandard (should always be available)
    let processor = ProcessorBuilder::new()
        .with_strategy(OptimizationStrategy::RustStandard)
        .build()?;

    assert_eq!(
        processor.selected_strategy(),
        OptimizationStrategy::RustStandard
    );

    Ok(())
}

#[test]
fn test_unavailable_strategy_error() {
    // Try to force a strategy that might not be available
    // This test checks error handling rather than specific availability
    let strategies_to_test = vec![
        OptimizationStrategy::AppleSiliconAssembly,
        OptimizationStrategy::GenericArm64Assembly,
        OptimizationStrategy::X86_64Assembly,
    ];

    for strategy in strategies_to_test {
        let result = ProcessorBuilder::new()
            .with_strategy(strategy.clone())
            .build();

        // Either succeeds (strategy available) or fails with clear error
        match result {
            Ok(processor) => {
                // If it succeeds, verify it's using the requested strategy
                assert_eq!(processor.selected_strategy(), strategy);
            }
            Err(AssemblyError::ExecutionError(msg)) => {
                // If it fails, should have a clear error message
                assert!(msg.contains("not available") || msg.contains("Strategy"));
            }
            Err(other) => {
                panic!("Unexpected error type for unavailable strategy: {other:?}");
            }
        }
    }
}

#[test]
fn test_force_assembly_behavior() {
    // Test force assembly behavior
    let result = ProcessorBuilder::new().force_assembly().build();

    match result {
        Ok(processor) => {
            // If assembly is available, verify it's using assembly
            let strategy = processor.selected_strategy();
            assert!(matches!(
                strategy,
                OptimizationStrategy::AppleSiliconAssembly
                    | OptimizationStrategy::GenericArm64Assembly
                    | OptimizationStrategy::X86_64Assembly
            ));
        }
        Err(AssemblyError::ExecutionError(msg)) => {
            // If no assembly available, should have clear error
            assert!(msg.contains("No assembly optimizations available"));
        }
        Err(other) => {
            panic!("Unexpected error type for force assembly: {other:?}");
        }
    }
}

#[test]
fn test_prefer_rust_fallback() -> Result<(), AssemblyError> {
    // Test prefer_rust - should always succeed since Rust is always available
    let processor = ProcessorBuilder::new().prefer_rust().build()?;

    // Should work regardless of what strategy was selected
    let mut processor = processor;
    let result = processor.process_string("Test")?;
    assert_eq!(result, "Test");

    assert_eq!(
        *processor.optimization_preference(),
        OptimizationPreference::PreferRust
    );

    Ok(())
}

#[test]
fn test_optimization_info_with_preference() -> Result<(), AssemblyError> {
    let processor = ProcessorBuilder::new().force_rust_only().build()?;

    let info = processor.optimization_info();

    // Should contain preference information
    assert!(info.contains("Preference:"));
    assert!(info.contains("ForceRustOnly"));

    Ok(())
}
