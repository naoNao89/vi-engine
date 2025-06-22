use std::time::Instant;
#[cfg(feature = "unsafe_performance")]
use vi::asm_clean_char_unsafe;
use vi::{
    asm_clean_char, clean_char, get_assembly_info, initialize_assembly_safety,
    is_assembly_available,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Assembly Debug Information ===");

    // Initialize assembly safety
    initialize_assembly_safety()?;

    // Check assembly availability
    println!("Assembly available: {}", is_assembly_available());
    println!("Assembly info: {}", get_assembly_info());

    // Test single character performance
    let test_char = 'ế';
    let iterations = 10000;

    println!("\n=== Performance Comparison ===");

    // Rust implementation
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = clean_char(test_char);
    }
    let rust_duration = start.elapsed();
    let rust_ns_per_char = rust_duration.as_nanos() as f64 / iterations as f64;

    println!("Rust: {:.2} ns/char", rust_ns_per_char);

    // Assembly implementation with safety (if available)
    if is_assembly_available() {
        let start = Instant::now();
        let mut successful_calls = 0;
        for _ in 0..iterations {
            match asm_clean_char(test_char) {
                Ok(_) => successful_calls += 1,
                Err(e) => {
                    println!("Assembly error: {:?}", e);
                    break;
                }
            }
        }
        let assembly_duration = start.elapsed();

        if successful_calls > 0 {
            let assembly_ns_per_char =
                assembly_duration.as_nanos() as f64 / successful_calls as f64;
            println!(
                "Assembly (safe): {:.2} ns/char ({} successful calls)",
                assembly_ns_per_char, successful_calls
            );

            let speedup = rust_ns_per_char / assembly_ns_per_char;
            println!("Safe speedup: {:.2}x", speedup);

            if speedup < 1.0 {
                println!("⚠️  Safe assembly is slower than Rust (safety overhead)");
            } else {
                println!("✅ Safe assembly is faster than Rust");
            }
        } else {
            println!("❌ All safe assembly calls failed");
        }

        // Test unsafe assembly performance if available
        #[cfg(feature = "unsafe_performance")]
        {
            println!("\n--- Unsafe Assembly Performance ---");
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = asm_clean_char_unsafe(test_char);
            }
            let unsafe_duration = start.elapsed();
            let unsafe_ns_per_char = unsafe_duration.as_nanos() as f64 / iterations as f64;

            println!("Assembly (unsafe): {:.2} ns/char", unsafe_ns_per_char);

            let unsafe_speedup = rust_ns_per_char / unsafe_ns_per_char;
            println!("Unsafe speedup: {:.2}x", unsafe_speedup);

            if unsafe_speedup < 1.0 {
                println!("⚠️  Unsafe assembly is slower than Rust");
            } else {
                println!("✅ Unsafe assembly is faster than Rust");
            }

            // Calculate safety overhead
            if successful_calls > 0 {
                let safety_overhead = assembly_ns_per_char / unsafe_ns_per_char;
                println!("Safety overhead: {:.2}x", safety_overhead);
            }
        }
        #[cfg(not(feature = "unsafe_performance"))]
        {
            println!("\n--- Unsafe Performance Test Disabled ---");
            println!("Enable with: cargo run --release --features unsafe_performance --example debug_assembly");
        }
    } else {
        println!("❌ Assembly not available - using Rust fallback");
    }

    // Test correctness
    println!("\n=== Correctness Test ===");
    let test_chars = ['ế', 'à', 'ộ', 'ư', 'đ', 'a', 'b', 'c'];

    for &ch in &test_chars {
        let rust_result = clean_char(ch);

        if is_assembly_available() {
            match asm_clean_char(ch) {
                Ok(assembly_result) => {
                    if rust_result == assembly_result {
                        println!("✅ '{}' -> '{}' (consistent)", ch, rust_result);
                    } else {
                        println!(
                            "❌ '{}' -> Rust: '{}', Assembly: '{}'",
                            ch, rust_result, assembly_result
                        );
                    }
                }
                Err(e) => {
                    println!("❌ '{}' -> Assembly error: {:?}", ch, e);
                }
            }
        } else {
            println!("✅ '{}' -> '{}' (Rust only)", ch, rust_result);
        }
    }

    Ok(())
}
