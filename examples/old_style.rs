//! Example demonstrating old-style Vietnamese accent processing
//!
//! This example shows how to process Vietnamese text with old-style accent placement.

fn main() {
    let inputs = "ra tajp hoas nhaf baf thuyr mua hoa hoef";

    let words = inputs.split(' ');

    let mut result = String::new();
    for word in words {
        vi::transform_buffer_with_style(
            &vi::TELEX,
            vi::processor::AccentStyle::Old,
            word.chars(),
            &mut result,
        );
        result.push(' ');
    }

    println!("{result}"); // prints "ra tạp hóa nhà bà thủy mua hoa hòe"
}
