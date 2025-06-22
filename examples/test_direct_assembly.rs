#!/usr/bin/env cargo
//! Direct Assembly Test
//!
//! This example tests calling the assembly functions directly
//! to debug the linking issue.

// Link the static library directly
#[cfg(feature = "apple_silicon_assembly")]
#[link(name = "apple_silicon_kernels", kind = "static")]
extern "C" {
    fn apple_hybrid_clean_char_optimized(ch: u32) -> u32;
}

fn main() {
    println!("=== Direct Assembly Test ===");

    #[cfg(feature = "apple_silicon_assembly")]
    {
        println!("Apple Silicon assembly feature is enabled");

        // Test direct assembly call
        let test_char = 'ế' as u32;
        println!(
            "Testing character: '{}' (U+{:04X})",
            char::from_u32(test_char).unwrap(),
            test_char
        );

        unsafe {
            let result = apple_hybrid_clean_char_optimized(test_char);
            println!(
                "Assembly result: '{}' (U+{:04X})",
                char::from_u32(result).unwrap(),
                result
            );
        }
    }

    #[cfg(not(feature = "apple_silicon_assembly"))]
    {
        println!("Apple Silicon assembly feature is NOT enabled");
    }

    println!("Test complete");
}
