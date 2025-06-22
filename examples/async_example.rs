//! Example demonstrating async-compatible Vietnamese text processing
//!
//! This example shows how to use the async interfaces for Vietnamese text processing
//! with proper cancellation and timeout support.

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use tokio::time::timeout;
    use vi::{initialize_assembly_safety, AsyncSafeAssemblyProcessor};

    // Initialize the safety system
    let _ = initialize_assembly_safety();

    // Create an async processor
    let processor = AsyncSafeAssemblyProcessor::new();

    println!("🚀 Async Vietnamese Text Processing Example");
    println!("============================================");

    // Test basic Vietnamese text processing
    let test_text = "Tiếng Việt rất đẹp và phong phú";
    println!("\n📝 Input: {}", test_text);

    let result = processor.process_string_safe(test_text).await?;
    println!("✨ Output: {}", result);

    // Test with timeout
    println!("\n⏱️  Testing with timeout...");
    let large_text = "Xin chào Việt Nam! ".repeat(1000);

    match timeout(
        Duration::from_millis(100),
        processor.process_string_safe(&large_text),
    )
    .await
    {
        Ok(Ok(processed_text)) => {
            println!(
                "✅ Processed {} characters successfully",
                processed_text.len()
            );
        }
        Ok(Err(e)) => {
            println!("❌ Processing failed: {}", e);
        }
        Err(_) => {
            println!("⏰ Operation timed out (as expected for large input)");
        }
    }

    // Test cancellation
    println!("\n🛑 Testing cancellation...");
    let processor2 = AsyncSafeAssemblyProcessor::new();

    // Start a long-running operation
    let handle = tokio::spawn(async move {
        let very_large_text = "à".repeat(100_000);
        processor2.process_string_safe(&very_large_text).await
    });

    // Cancel after a short delay
    tokio::time::sleep(Duration::from_millis(10)).await;
    handle.abort();

    println!("✅ Cancellation test completed");

    // Show metrics
    let metrics = processor.get_metrics();
    println!("\n📊 Processing Metrics:");
    println!(
        "   Operations started: {}",
        metrics
            .operations_started
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "   Operations completed: {}",
        metrics
            .operations_completed
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    println!(
        "   Success rate: {:.1}%",
        metrics.get_success_rate() * 100.0
    );

    println!("\n🎉 Async Vietnamese text processing example completed!");

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --features async --example async_example");
}
