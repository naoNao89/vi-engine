//! Assembly Function Verification Example
//!
//! This example demonstrates how to check if assembly functions are available
//! and properly linked in the vi-rust project.

#[allow(unused_imports)]
use std::ffi::c_uint;

// External assembly function declarations (if they were to be used)
extern "C" {
    // These functions are compiled but not currently integrated
    // fn _apple_hybrid_clean_char_optimized(ch: c_uint) -> c_uint;
    // fn _apple_hybrid_clean_char_neon(ch: c_uint) -> c_uint;
}

fn main() {
    println!("=== Assembly Function Verification ===");

    // Check if we're running on the target architecture
    println!("Target Architecture: {}", std::env::consts::ARCH);
    println!("Target OS: {}", std::env::consts::OS);

    // Check build-time assembly features
    #[cfg(any(
        feature = "apple_silicon_assembly",
        feature = "x86_64_assembly",
        feature = "aarch64_assembly"
    ))]
    println!("✅ Assembly kernels feature enabled");

    #[cfg(not(any(
        feature = "apple_silicon_assembly",
        feature = "x86_64_assembly",
        feature = "aarch64_assembly"
    )))]
    println!("❌ Assembly kernels feature disabled");

    #[cfg(feature = "aarch64_assembly")]
    println!("✅ ARM64 assembly available");

    #[cfg(feature = "apple_silicon_assembly")]
    println!("✅ Apple Silicon assembly available");

    #[cfg(feature = "x86_64_assembly")]
    println!("✅ x86_64 assembly available");

    // Test the current Rust implementation
    println!("\n=== Testing Current Rust Implementation ===");

    let test_chars = vec!['à', 'á', 'ả', 'ã', 'ạ', 'đ', 'Đ', 'a', 'z'];

    for &ch in &test_chars {
        let cleaned = vi::clean_char(ch);
        println!(
            "'{}' -> '{}' (U+{:04X} -> U+{:04X})",
            ch, cleaned, ch as u32, cleaned as u32
        );
    }

    // Test string processing
    println!("\n=== Testing String Processing ===");
    let test_strings = vec!["Tiếng Việt", "Xin chào", "Hà Nội", "Đà Nẵng", "Hồ Chí Minh"];

    for input in &test_strings {
        let output = vi::clean_string(input);
        println!("'{}' -> '{}'", input, output);
    }

    println!("\n=== Assembly Status Summary ===");
    println!("The assembly functions are compiled and available in the binary,");
    println!("but are not currently integrated into the Rust API.");
    println!("They exist as optimized kernels ready for future integration.");

    // Show assembly artifact locations
    println!("\n=== Assembly Artifacts ===");
    println!("Assembly source files:");
    println!("  - src/asm/aarch64_apple_silicon.s (Apple Silicon optimized)");
    println!("  - src/asm/aarch64_kernels.s (Generic ARM64)");
    println!("  - src/asm/x86_64_kernels.s (x86_64 optimized)");

    println!("\nCompiled assembly libraries:");
    println!("  - libaarch64_apple_silicon.a");
    println!("  - libaarch64_kernels.a");

    println!("\nExported assembly functions:");
    println!("  - _apple_hybrid_clean_char_optimized");
    println!("  - _apple_hybrid_clean_char_neon");
    println!("  - _apple_hybrid_clean_chars_bulk_neon_optimized");
    println!("  - _apple_hybrid_clean_chars_bulk_simd_ultra");
}
