#!/usr/bin/env cargo
//! Direct Assembly Test
//!
//! This example tests calling the assembly functions directly
//! to debug the linking issue.

// Link the static library directly
#[cfg(all(feature = "apple_silicon_assembly", not(feature = "no_assembly")))]
#[link(name = "apple_silicon_kernels", kind = "static")]
extern "C" {
    fn apple_hybrid_clean_char_optimized(ch: u32) -> u32;
}

fn main() {
    println!("=== Direct Assembly Test ===");

    #[cfg(all(feature = "apple_silicon_assembly", not(feature = "no_assembly")))]
    {
        println!("Apple Silicon assembly feature is enabled");

        // Test direct assembly call
        let test_char = 'ế' as u32;
        match char::from_u32(test_char) {
            Some(ch) => println!("Testing character: '{}' (U+{:04X})", ch, test_char),
            None => {
                println!("Invalid test character: U+{:04X}", test_char);
                return;
            }
        }

        unsafe {
            let result = apple_hybrid_clean_char_optimized(test_char);
            match char::from_u32(result) {
                Some(ch) => println!("Assembly result: '{}' (U+{:04X})", ch, result),
                None => println!("Assembly returned invalid Unicode: U+{:04X}", result),
            }
        }
    }

    #[cfg(not(feature = "apple_silicon_assembly"))]
    {
        println!("Apple Silicon assembly feature is NOT enabled");
    }

    println!("Test complete");
}
