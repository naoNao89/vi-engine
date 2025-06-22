#!/usr/bin/env cargo
//! Single Character Loop Test
//!
//! This example tests processing characters one by one to avoid the bulk function bug.

use vi::{asm_clean_char, clean_char};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Single Character Loop Test ===");

    // Test with the same characters as the bulk test
    let input_chars = ['T', 'i', 'ế', 'n', 'g'];
    let mut output_chars = Vec::new();

    println!("Input characters: {:?}", input_chars);

    // Process each character individually using the assembly API
    for &ch in &input_chars {
        let result = asm_clean_char(ch)?;
        output_chars.push(result);
        println!("'{}' -> '{}'", ch, result);
    }

    println!("Output characters: {:?}", output_chars);

    // Verify results match the standard API
    let expected: Vec<char> = input_chars.iter().map(|&ch| clean_char(ch)).collect();
    println!("Expected: {:?}", expected);

    if output_chars == expected {
        println!("✅ All results match!");
    } else {
        println!("❌ Results don't match!");
    }

    println!("Test complete");
    Ok(())
}
