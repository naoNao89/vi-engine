#!/usr/bin/env cargo
//! Debug character mappings

use vi::{asm_clean_char, clean_char, initialize_assembly_safety};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_assembly_safety()?;

    let test_chars = ['ỉ', 'Ỉ', 'ế', 'Ế', 'ệ', 'Ệ', 'Ă', 'ă', 'ĩ', 'Ĩ'];

    println!("Character mapping comparison:");
    for ch in test_chars {
        let rust_result = clean_char(ch);
        let assembly_result = asm_clean_char(ch)?;

        println!(
            "'{}' (U+{:04X}) -> Rust: '{}' (U+{:04X}), Assembly: '{}' (U+{:04X})",
            ch, ch as u32, rust_result, rust_result as u32, assembly_result, assembly_result as u32
        );
    }

    Ok(())
}
