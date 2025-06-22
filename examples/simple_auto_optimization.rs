//! Simple Auto-Optimization Example
//!
//! This example demonstrates the new production-ready API with automatic
//! runtime architecture detection and optimization selection.

use vi::VietnameseTextProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VI-RUST Auto-Optimization Demo ===\n");

    // Create processor - automatically detects CPU and selects best optimization
    let mut processor = VietnameseTextProcessor::new()?;

    // Display what optimization was selected
    println!("🚀 Optimization Info:");
    println!("{}\n", processor.optimization_info());

    // Display CPU information
    let cpu_info = processor.cpu_info();
    println!("💻 CPU Information:");
    println!("  Architecture: {}", cpu_info.architecture_description());
    println!("  Performance: {}", cpu_info.performance_description());
    println!("  Score: {}\n", cpu_info.performance_score);

    // Process some Vietnamese text
    println!("📝 Processing Vietnamese Text:");

    let test_cases = vec![
        "Tiếng Việt",
        "Xin chào thế giới",
        "Cảm ơn bạn rất nhiều",
        "Hôm nay trời đẹp quá",
        "Tôi yêu Việt Nam",
    ];

    for input in &test_cases {
        let result = processor.process_string(input)?;
        println!("  '{}' → '{}'", input, result);
    }

    // Show performance statistics
    println!("\n📊 Performance Statistics:");
    let stats = processor.stats();
    println!("  Characters processed: {}", stats.total_chars_processed);
    println!("  Success rate: {:.1}%", processor.success_rate());
    println!(
        "  Average rate: {:.0} chars/sec",
        processor.avg_processing_rate()
    );
    println!(
        "  Optimal performance: {}",
        processor.is_performing_optimally()
    );

    println!("\n✅ Demo complete!");
    Ok(())
}
