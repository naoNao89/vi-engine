use std::time::Instant;
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
    let rust_ns_per_char = rust_duration.as_nanos() as f64 / f64::from(iterations);

    println!("Rust: {rust_ns_per_char:.2} ns/char");

    // Assembly implementation with safety (if available)
    if is_assembly_available() {
        let start = Instant::now();
        let mut successful_calls = 0;
        for _ in 0..iterations {
            match asm_clean_char(test_char) {
                Ok(_) => successful_calls += 1,
                Err(e) => {
                    println!("Assembly error: {e:?}");
                    break;
                }
            }
        }
        let assembly_duration = start.elapsed();

        if successful_calls > 0 {
            let assembly_ns_per_char = {
                let nanos = assembly_duration.as_nanos();
                let nanos_f64 = if nanos > (1u64 << 53) as u128 {
                    (1u64 << 53) as f64
                } else {
                    nanos as f64
                };
                nanos_f64 / f64::from(successful_calls)
            };
            println!(
                "Assembly (safe): {assembly_ns_per_char:.2} ns/char ({successful_calls} successful calls)"
            );

            let speedup = rust_ns_per_char / assembly_ns_per_char;
            println!("Safe speedup: {speedup:.2}x");

            if speedup < 1.0 {
                println!("⚠️  Safe assembly is slower than Rust (safety overhead)");
            } else {
                println!("✅ Safe assembly is faster than Rust");
            }
        } else {
            println!("❌ All safe assembly calls failed");
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
                        println!("✅ '{ch}' -> '{rust_result}' (consistent)");
                    } else {
                        println!(
                            "❌ '{ch}' -> Rust: '{rust_result}', Assembly: '{assembly_result}'"
                        );
                    }
                }
                Err(e) => {
                    println!("❌ '{ch}' -> Assembly error: {e:?}");
                }
            }
        } else {
            println!("✅ '{ch}' -> '{rust_result}' (Rust only)");
        }
    }

    Ok(())
}
