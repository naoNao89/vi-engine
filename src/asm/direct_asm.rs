//! Direct Assembly Interface
//!
//! This module provides safe Rust interfaces to the hand-optimized assembly kernels
//! for Vietnamese character processing. It handles platform detection, function
//! selection, and safe calling conventions.

use crate::safety::{AssemblyControl, AssemblyError};

// Platform-specific assembly function declarations
// These functions are compiled and linked by build.rs

// Apple Silicon optimized functions
#[cfg(feature = "apple_silicon_assembly")]
extern "C" {
    /// Process single character with Apple Silicon optimizations
    ///
    /// # Safety
    /// - Input must be a valid Unicode codepoint (≤ 0x10FFFF)
    /// - Function is stateless and thread-safe
    #[cfg(target_arch = "x86_64")]
    fn apple_hybrid_clean_char_optimized(ch: u32) -> u32;

    #[cfg(target_arch = "aarch64")]
    fn _apple_hybrid_clean_char_optimized(ch: u32) -> u32;

    // Note: Bulk assembly functions are currently disabled due to stack overflow issues
    // Using character-by-character processing as a workaround
}

// x86_64 optimized functions
#[cfg(feature = "x86_64_assembly")]
extern "C" {
    /// Process single character with `x86_64` optimizations
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_char_x86_64(ch: u32) -> u32;

    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_char_x86_64(ch: u32) -> u32;

    /// Process character array with AVX-512 vectorization
    #[allow(dead_code)]
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_chars_bulk_avx512(input: *const u32, output: *mut u32, len: usize) -> usize;

    #[allow(dead_code)]
    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_chars_bulk_avx512(input: *const u32, output: *mut u32, len: usize) -> usize;

    /// Process character array with BMI2 optimizations
    #[allow(dead_code)]
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_chars_bulk_bmi2(input: *const u32, output: *mut u32, len: usize) -> usize;

    #[allow(dead_code)]
    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_chars_bulk_bmi2(input: *const u32, output: *mut u32, len: usize) -> usize;

    /// Safety-aware bulk processing with control structure
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_chars_bulk_safe(
        input: *const u32,
        output: *mut u32,
        len: usize,
        control: *const AssemblyControl,
    ) -> usize;

    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_chars_bulk_safe(
        input: *const u32,
        output: *mut u32,
        len: usize,
        control: *const AssemblyControl,
    ) -> usize;
}

// Generic ARM64 functions
#[cfg(feature = "aarch64_assembly")]
extern "C" {
    /// Process single character with generic ARM64 instructions
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_char_aarch64(ch: u32) -> u32;

    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_char_aarch64(ch: u32) -> u32;

    /// Process character array with generic ARM64 optimizations
    #[allow(dead_code)]
    #[cfg(target_arch = "x86_64")]
    fn hybrid_clean_chars_bulk_neon(input: *const u32, output: *mut u32, len: usize) -> usize;

    #[allow(dead_code)]
    #[cfg(target_arch = "aarch64")]
    fn _hybrid_clean_chars_bulk_neon(input: *const u32, output: *mut u32, len: usize) -> usize;
}

/// Assembly platform detection and selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssemblyPlatform {
    /// Apple Silicon ARM64 with optimized assembly kernels
    AppleSilicon,
    /// `x86_64` with AVX-512 and BMI2 optimizations
    X86_64,
    /// Generic ARM64 with NEON optimizations
    GenericARM64,
    /// Pure Rust fallback implementation
    RustFallback,
}

impl AssemblyPlatform {
    /// Detect the best available assembly platform
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(feature = "apple_silicon_assembly")]
        {
            Self::AppleSilicon
        }
        #[cfg(all(feature = "x86_64_assembly", not(feature = "apple_silicon_assembly")))]
        {
            Self::X86_64
        }
        #[cfg(all(
            feature = "aarch64_assembly",
            not(feature = "apple_silicon_assembly"),
            not(feature = "x86_64_assembly")
        ))]
        {
            Self::GenericARM64
        }
        #[cfg(not(any(
            feature = "apple_silicon_assembly",
            feature = "x86_64_assembly",
            feature = "aarch64_assembly"
        )))]
        {
            Self::RustFallback
        }
    }

    /// Get platform name for logging/debugging
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::AppleSilicon => "Apple Silicon ARM64",
            Self::X86_64 => "x86_64 with BMI2/AVX-512",
            Self::GenericARM64 => "Generic ARM64",
            Self::RustFallback => "Rust Fallback",
        }
    }
}

/// Safe assembly function interface
pub struct AssemblyInterface {
    platform: AssemblyPlatform,
}

impl AssemblyInterface {
    /// Create new assembly interface with automatic platform detection
    #[must_use]
    pub fn new() -> Self {
        Self {
            platform: AssemblyPlatform::detect(),
        }
    }

    /// Get the detected platform
    #[must_use]
    pub fn platform(&self) -> AssemblyPlatform {
        self.platform
    }

    /// Check if assembly is available (not using Rust fallback)
    #[must_use]
    pub fn is_assembly_available(&self) -> bool {
        self.platform != AssemblyPlatform::RustFallback
    }

    /// Process single character using best available assembly
    pub fn process_char_safe(
        &self,
        ch: char,
        control: &AssemblyControl,
    ) -> Result<char, AssemblyError> {
        let ch_u32 = ch as u32;

        // Validate input
        if ch_u32 > 0x0010_FFFF {
            return Err(AssemblyError::InvalidInput);
        }

        // Check if operation should continue
        if !control.should_continue() {
            return Err(AssemblyError::Cancelled);
        }

        match self.platform {
            AssemblyPlatform::AppleSilicon => {
                #[cfg(feature = "apple_silicon_assembly")]
                {
                    // SAFETY: ch_u32 is validated to be ≤ 0x10FFFF above, making it a valid Unicode codepoint.
                    // The assembly function is stateless and thread-safe.
                    #[cfg(target_arch = "x86_64")]
                    let result_u32 = unsafe { apple_hybrid_clean_char_optimized(ch_u32) };

                    #[cfg(target_arch = "aarch64")]
                    let result_u32 = unsafe { _apple_hybrid_clean_char_optimized(ch_u32) };

                    // Validate output
                    char::from_u32(result_u32).ok_or_else(|| {
                        AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid Unicode: 0x{result_u32:X}"
                        ))
                    })
                }
                #[cfg(not(feature = "apple_silicon_assembly"))]
                self.process_char_rust_fallback(ch)
            }
            AssemblyPlatform::X86_64 => {
                #[cfg(feature = "x86_64_assembly")]
                {
                    // SAFETY: ch_u32 is validated to be ≤ 0x10FFFF above, making it a valid Unicode codepoint.
                    // The assembly function is stateless and thread-safe.
                    #[cfg(target_arch = "x86_64")]
                    let result_u32 = unsafe { hybrid_clean_char_x86_64(ch_u32) };

                    #[cfg(target_arch = "aarch64")]
                    let result_u32 = unsafe { _hybrid_clean_char_x86_64(ch_u32) };

                    // Validate output
                    char::from_u32(result_u32).ok_or_else(|| {
                        AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid Unicode: 0x{result_u32:X}"
                        ))
                    })
                }
                #[cfg(not(feature = "x86_64_assembly"))]
                self.process_char_rust_fallback(ch)
            }
            AssemblyPlatform::GenericARM64 => {
                #[cfg(feature = "aarch64_assembly")]
                {
                    // SAFETY: ch_u32 is validated to be ≤ 0x10FFFF above, making it a valid Unicode codepoint.
                    // The assembly function is stateless and thread-safe.
                    #[cfg(target_arch = "x86_64")]
                    let result_u32 = unsafe { hybrid_clean_char_aarch64(ch_u32) };

                    #[cfg(target_arch = "aarch64")]
                    let result_u32 = unsafe { _hybrid_clean_char_aarch64(ch_u32) };

                    // Validate output
                    char::from_u32(result_u32).ok_or_else(|| {
                        AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid Unicode: 0x{result_u32:X}"
                        ))
                    })
                }
                #[cfg(not(feature = "aarch64_assembly"))]
                self.process_char_rust_fallback(ch)
            }
            AssemblyPlatform::RustFallback => self.process_char_rust_fallback(ch),
        }
    }

    /// Process character array using best available assembly with safety
    pub fn process_chars_bulk_safe(
        &self,
        input: &[u32],
        output: &mut [u32],
        control: &AssemblyControl,
    ) -> Result<usize, AssemblyError> {
        if input.len() != output.len() {
            return Err(AssemblyError::InvalidInput);
        }

        if input.is_empty() {
            return Ok(0);
        }

        // Check for reasonable size limits (prevent DoS)
        if input.len() > 100_000_000 {
            return Err(AssemblyError::InvalidInput);
        }

        // Reset control structure for this operation
        control.reset_for_operation(input.len());

        match self.platform {
            AssemblyPlatform::AppleSilicon => {
                #[cfg(feature = "apple_silicon_assembly")]
                {
                    // WORKAROUND: The bulk assembly function has a bug causing segfaults
                    // Use character-by-character processing with the working single-char function
                    let mut processed = 0;
                    for (i, &ch_u32) in input.iter().enumerate() {
                        if !control.should_continue() {
                            break;
                        }

                        // SAFETY: ch_u32 comes from input slice, assembly function is stateless
                        #[cfg(target_arch = "x86_64")]
                        let result = unsafe { apple_hybrid_clean_char_optimized(ch_u32) };

                        #[cfg(target_arch = "aarch64")]
                        let result = unsafe { _apple_hybrid_clean_char_optimized(ch_u32) };

                        output[i] = result;
                        processed += 1;
                    }

                    // Validate return value
                    if processed > input.len() {
                        return Err(AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid count: {} > {}",
                            processed,
                            input.len()
                        )));
                    }

                    Ok(processed)
                }
                #[cfg(not(feature = "apple_silicon_assembly"))]
                self.process_chars_rust_fallback(input, output)
            }
            AssemblyPlatform::X86_64 => {
                #[cfg(feature = "x86_64_assembly")]
                {
                    // SAFETY:
                    // - input and output slices have been validated to have equal length above
                    // - input.as_ptr() points to valid array of input.len() u32 values
                    // - output.as_mut_ptr() points to writable array of input.len() u32 values
                    // - Arrays are guaranteed not to overlap due to Rust's borrowing rules
                    // - control pointer is valid for the duration of this call
                    // - Assembly function respects the control structure for cancellation
                    #[cfg(target_arch = "x86_64")]
                    let processed = unsafe {
                        hybrid_clean_chars_bulk_safe(
                            input.as_ptr(),
                            output.as_mut_ptr(),
                            input.len(),
                            control as *const AssemblyControl,
                        )
                    };

                    #[cfg(target_arch = "aarch64")]
                    let processed = unsafe {
                        _hybrid_clean_chars_bulk_safe(
                            input.as_ptr(),
                            output.as_mut_ptr(),
                            input.len(),
                            control as *const AssemblyControl,
                        )
                    };

                    // Validate return value
                    if processed > input.len() {
                        return Err(AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid count: {} > {}",
                            processed,
                            input.len()
                        )));
                    }

                    Ok(processed)
                }
                #[cfg(not(feature = "x86_64_assembly"))]
                {
                    self.process_chars_rust_fallback(input, output)
                }
            }
            AssemblyPlatform::GenericARM64 => {
                #[cfg(feature = "aarch64_assembly")]
                {
                    // Generic ARM64 doesn't have safety-aware bulk processing
                    // Use the available bulk function with periodic safety checks
                    let mut processed = 0;
                    let chunk_size = 1024; // Process in chunks for safety checks

                    while processed < input.len() {
                        if !control.should_continue() {
                            break;
                        }

                        let remaining = input.len() - processed;
                        let current_chunk = chunk_size.min(remaining);

                        // SAFETY:
                        // - input and output slices have been validated to have equal length above
                        // - Chunk boundaries are validated to be within array bounds
                        // - Arrays are guaranteed not to overlap due to Rust's borrowing rules
                        #[cfg(target_arch = "x86_64")]
                        let chunk_processed = unsafe {
                            hybrid_clean_chars_bulk_neon(
                                input.as_ptr().add(processed),
                                output.as_mut_ptr().add(processed),
                                current_chunk,
                            )
                        };

                        #[cfg(target_arch = "aarch64")]
                        let chunk_processed = unsafe {
                            _hybrid_clean_chars_bulk_neon(
                                input.as_ptr().add(processed),
                                output.as_mut_ptr().add(processed),
                                current_chunk,
                            )
                        };

                        processed += chunk_processed;

                        // If we didn't process the full chunk, something went wrong
                        if chunk_processed < current_chunk {
                            break;
                        }
                    }

                    // Validate return value
                    if processed > input.len() {
                        return Err(AssemblyError::ExecutionError(format!(
                            "Assembly returned invalid count: {} > {}",
                            processed,
                            input.len()
                        )));
                    }

                    Ok(processed)
                }
                #[cfg(not(feature = "aarch64_assembly"))]
                self.process_chars_rust_fallback(input, output)
            }
            AssemblyPlatform::RustFallback => self.process_chars_rust_fallback(input, output),
        }
    }

    /// Rust fallback implementation for single character
    fn process_char_rust_fallback(&self, ch: char) -> Result<char, AssemblyError> {
        // Use the existing Rust implementation from util module
        Ok(crate::util::clean_char(ch))
    }

    /// Rust fallback implementation for bulk processing
    fn process_chars_rust_fallback(
        &self,
        input: &[u32],
        output: &mut [u32],
    ) -> Result<usize, AssemblyError> {
        for (i, &ch_u32) in input.iter().enumerate() {
            let ch = char::from_u32(ch_u32).unwrap_or('\u{FFFD}');
            let result = crate::util::clean_char(ch);
            output[i] = result as u32;
        }
        Ok(input.len())
    }
}

impl Default for AssemblyInterface {
    fn default() -> Self {
        Self::new()
    }
}

/// Global assembly interface instance
static ASSEMBLY_INTERFACE: once_cell::sync::Lazy<AssemblyInterface> =
    once_cell::sync::Lazy::new(AssemblyInterface::new);

/// Get the global assembly interface
#[must_use]
pub fn get_assembly_interface() -> &'static AssemblyInterface {
    &ASSEMBLY_INTERFACE
}

/// Check if assembly optimizations are available
#[must_use]
pub fn is_assembly_available() -> bool {
    get_assembly_interface().is_assembly_available()
}

/// Performance-optimized assembly interface that bypasses safety checks
/// WARNING: Only use for benchmarking - no safety guarantees!
#[cfg(any(test, feature = "unsafe_performance"))]
#[must_use]
pub fn process_char_unsafe(ch: char) -> char {
    // Direct assembly call without safety overhead
    match AssemblyPlatform::detect() {
        AssemblyPlatform::AppleSilicon => {
            #[cfg(feature = "apple_silicon_assembly")]
            {
                // Direct Apple Silicon assembly call
                #[cfg(target_arch = "x86_64")]
                let result = unsafe { apple_hybrid_clean_char_optimized(ch as u32) };

                #[cfg(target_arch = "aarch64")]
                let result = unsafe { _apple_hybrid_clean_char_optimized(ch as u32) };

                char::from_u32(result).unwrap_or(ch)
            }
            #[cfg(not(feature = "apple_silicon_assembly"))]
            crate::util::clean_char(ch)
        }
        AssemblyPlatform::X86_64 => {
            #[cfg(feature = "x86_64_assembly")]
            {
                // Direct x86_64 assembly call
                #[cfg(target_arch = "x86_64")]
                let result = unsafe { hybrid_clean_char_x86_64(ch as u32) };

                #[cfg(target_arch = "aarch64")]
                let result = unsafe { _hybrid_clean_char_x86_64(ch as u32) };

                char::from_u32(result).unwrap_or(ch)
            }
            #[cfg(not(feature = "x86_64_assembly"))]
            crate::util::clean_char(ch)
        }
        AssemblyPlatform::GenericARM64 => {
            #[cfg(feature = "aarch64_assembly")]
            {
                // Direct ARM64 assembly call
                #[cfg(target_arch = "x86_64")]
                let result = unsafe { hybrid_clean_char_aarch64(ch as u32) };

                #[cfg(target_arch = "aarch64")]
                let result = unsafe { _hybrid_clean_char_aarch64(ch as u32) };

                char::from_u32(result).unwrap_or(ch)
            }
            #[cfg(not(feature = "aarch64_assembly"))]
            crate::util::clean_char(ch)
        }
        AssemblyPlatform::RustFallback => {
            // Fallback to optimized Rust
            crate::util::clean_char(ch)
        }
    }
}

/// Performance-optimized bulk processing without safety overhead
#[cfg(any(test, feature = "unsafe_performance"))]
#[must_use]
pub fn process_chars_bulk_unsafe(input: &[char]) -> Vec<char> {
    // Use character-by-character processing for maximum performance
    // This avoids the safety overhead of the bulk functions
    input.iter().map(|&ch| process_char_unsafe(ch)).collect()
}

/// Get assembly platform information
#[must_use]
pub fn get_assembly_info() -> String {
    let interface = get_assembly_interface();
    format!(
        "Platform: {} (Available: {})",
        interface.platform().name(),
        interface.is_assembly_available()
    )
}
