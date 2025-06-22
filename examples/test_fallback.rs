#!/usr/bin/env cargo
//! Test the assembly module with Rust fallback only
//!
//! This example tests the assembly module interface using only
//! the Rust fallback implementation to verify the API works.

use vi::{
    // Assembly API (should fall back to Rust)
    asm_clean_char,
    asm_clean_string,
    // Standard API
    clean_char,
    clean_string,
    get_assembly_info,
    // Safety infrastructure
    initialize_assembly_safety,
    is_assembly_available,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Assembly Fallback Test ===");
    println!();

    // Initialize assembly safety system
    initialize_assembly_safety()?;

    // Display system information
    println!("=== System Information ===");
    println!("Assembly Available: {}", is_assembly_available());
    println!("Assembly Info: {}", get_assembly_info());
    println!();

    // Test character processing
    println!("=== Character Processing Test ===");
    let test_chars = ['ế', 'ạ', 'ữ', 'đ', 'Đ', 'a', 'z'];

    for ch in test_chars {
        let rust_result = clean_char(ch);
        let asm_result = asm_clean_char(ch)?;

        println!("'{ch}' -> Rust: '{rust_result}', Assembly: '{asm_result}'");

        // Verify results match
        assert_eq!(rust_result, asm_result, "Results should match for '{ch}'");
    }
    println!();

    // Test string processing
    println!("=== String Processing Test ===");
    let test_strings = ["Tiếng Việt", "Xin chào", "Hello World"];

    for input in test_strings {
        let rust_result = clean_string(input);
        let asm_result = asm_clean_string(input)?;

        println!("Input:    '{input}'");
        println!("Rust:     '{rust_result}'");
        println!("Assembly: '{asm_result}'");
        println!();

        // Verify results match
        assert_eq!(
            rust_result, asm_result,
            "Results should match for '{input}'"
        );
    }

    println!("=== All Tests Passed! ===");

    Ok(())
}
