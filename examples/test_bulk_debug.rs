#!/usr/bin/env cargo
//! Bulk Processing Debug Test
//!
//! This example tests the bulk processing function to debug the issue.

// Link the static library directly
#[cfg(feature = "apple_silicon_assembly")]
#[link(name = "aarch64_apple_silicon", kind = "static")]
extern "C" {
    fn apple_hybrid_clean_chars_bulk_neon_optimized(
        input: *const u32,
        output: *mut u32,
        len: usize,
    ) -> usize;
}

fn main() {
    println!("=== Bulk Processing Debug Test ===");

    #[cfg(feature = "apple_silicon_assembly")]
    {
        println!("Apple Silicon assembly feature is enabled");

        // Test with a small array
        let input_chars = ['T', 'i', 'ế', 'n', 'g'];
        let input_u32: Vec<u32> = input_chars.iter().map(|&c| c as u32).collect();
        let mut output_u32 = vec![0u32; input_chars.len()];

        println!("Input characters: {:?}", input_chars);
        println!("Input as u32: {:?}", input_u32);
        println!("Input length: {}", input_u32.len());

        unsafe {
            let processed = apple_hybrid_clean_chars_bulk_neon_optimized(
                input_u32.as_ptr(),
                output_u32.as_mut_ptr(),
                input_u32.len(),
            );

            println!("Assembly returned processed count: {}", processed);
            println!("Output as u32: {:?}", output_u32);

            if processed <= input_u32.len() {
                let output_chars: Vec<char> = output_u32[..processed]
                    .iter()
                    .map(|&u| char::from_u32(u).unwrap_or('?'))
                    .collect();
                println!("Output characters: {:?}", output_chars);
            } else {
                println!("ERROR: Assembly returned invalid count!");
            }
        }
    }

    #[cfg(not(feature = "apple_silicon_assembly"))]
    {
        println!("Apple Silicon assembly feature is NOT enabled");
    }

    println!("Test complete");
}
