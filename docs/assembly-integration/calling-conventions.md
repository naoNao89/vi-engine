# Assembly Calling Conventions and ABI Reference

This document provides detailed information about calling conventions, Application Binary Interface (ABI) requirements, and platform-specific considerations for the vi-rust assembly integration.

## Overview

The vi-rust project uses C-compatible calling conventions to interface between Rust and assembly code. This ensures compatibility across different platforms while maintaining performance and safety.

## Platform-Specific Calling Conventions

### ARM64 (AArch64) - AAPCS64

The ARM64 assembly functions follow the ARM Architecture Procedure Call Standard (AAPCS64):

#### Register Usage
- **Argument Registers**: `w0-w7` (32-bit), `x0-x7` (64-bit)
- **Return Register**: `w0` (32-bit), `x0` (64-bit)
- **Caller-Saved**: `x0-x18`, `x30` (LR)
- **Callee-Saved**: `x19-x28`, `x29` (FP), `sp`
- **Special Purpose**: `x29` (Frame Pointer), `x30` (Link Register)

#### Function Signatures

**Single Character Processing:**
```assembly
// ARM64 function signature
// Input: w0 = character (u32)
// Output: w0 = cleaned character (u32)
.global _apple_hybrid_clean_char_optimized
_apple_hybrid_clean_char_optimized:
    // w0 contains input character
    // ... processing logic ...
    // w0 contains output character
    ret
```

**Bulk Character Processing:**
```assembly
// ARM64 bulk processing signature
// Input: x0 = input array pointer (*const u32)
//        x1 = output array pointer (*mut u32)
//        x2 = length (usize)
// Output: x0 = number of characters processed (usize)
.global _apple_hybrid_clean_chars_bulk_neon_optimized
_apple_hybrid_clean_chars_bulk_neon_optimized:
    // x0 = input pointer
    // x1 = output pointer
    // x2 = length
    // ... processing logic ...
    // x0 = processed count
    ret
```

#### Stack Frame Layout
```
High Address
+------------------+
| Caller's Frame   |
+------------------+
| Return Address   | <- x30 (LR)
+------------------+
| Frame Pointer    | <- x29 (FP)
+------------------+
| Callee-Saved     |
| Registers        |
+------------------+
| Local Variables  |
+------------------+
| Spill Area       |
+------------------+ <- sp
Low Address
```

#### Apple Silicon Specific Optimizations

Apple Silicon processors have additional considerations:

```assembly
// Apple Silicon optimized function prologue
_apple_hybrid_clean_char_neon:
    // Standard AAPCS64 prologue
    stp x29, x30, [sp, #-32]!    // Save FP and LR
    mov x29, sp                   // Set up frame pointer
    stp x19, x20, [sp, #16]      // Save callee-saved registers
    
    // Apple Silicon specific optimizations
    prfm pldl1keep, [x0, #64]    // Prefetch for unified memory
    
    // ... function body ...
    
    // Standard epilogue
    ldp x19, x20, [sp, #16]      // Restore callee-saved registers
    ldp x29, x30, [sp], #32      // Restore FP and LR
    ret
```

### x86_64 - System V AMD64 ABI

The x86_64 assembly functions follow the System V AMD64 ABI:

#### Register Usage
- **Argument Registers**: `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
- **Return Register**: `rax`
- **Caller-Saved**: `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8-r11`
- **Callee-Saved**: `rbx`, `rbp`, `r12-r15`
- **Stack Pointer**: `rsp`
- **Frame Pointer**: `rbp`

#### Function Signatures

**Single Character Processing:**
```assembly
# x86_64 function signature
# Input: edi = character (u32)
# Output: eax = cleaned character (u32)
.global hybrid_clean_char_x86_64
hybrid_clean_char_x86_64:
    # edi contains input character
    # ... processing logic ...
    # eax contains output character
    ret
```

**Bulk Character Processing:**
```assembly
# x86_64 bulk processing signature
# Input: rdi = input array pointer (*const u32)
#        rsi = output array pointer (*mut u32)
#        rdx = length (usize)
# Output: rax = number of characters processed (usize)
.global hybrid_clean_chars_bulk_avx512
hybrid_clean_chars_bulk_avx512:
    # rdi = input pointer
    # rsi = output pointer
    # rdx = length
    # ... processing logic ...
    # rax = processed count
    ret
```

#### Stack Frame Layout
```
High Address
+------------------+
| Caller's Frame   |
+------------------+
| Return Address   |
+------------------+
| Frame Pointer    | <- rbp
+------------------+
| Callee-Saved     |
| Registers        |
+------------------+
| Local Variables  |
+------------------+
| Red Zone (128B)  |
+------------------+ <- rsp
Low Address
```

## Rust FFI Interface

### External Function Declarations

The Rust code declares assembly functions using appropriate calling conventions:

```rust
// ARM64 Apple Silicon functions
#[cfg(feature = "apple_silicon_assembly")]
extern "C" {
    /// Process single character with Apple Silicon optimizations
    /// 
    /// # Safety
    /// - Input must be a valid Unicode codepoint (≤ 0x10FFFF)
    /// - Function is stateless and thread-safe
    fn _apple_hybrid_clean_char_optimized(ch: u32) -> u32;
    
    /// Process character array with NEON vectorization
    /// 
    /// # Safety
    /// - input must point to valid array of `len` u32 values
    /// - output must point to writable array of at least `len` u32 values
    /// - Arrays must not overlap
    /// - Function may process fewer than `len` characters if cancelled
    fn _apple_hybrid_clean_chars_bulk_neon_optimized(
        input: *const u32,
        output: *mut u32,
        len: usize,
    ) -> usize;
}

// x86_64 functions
#[cfg(feature = "x86_64_assembly")]
extern "C" {
    /// Process single character with x86_64 optimizations
    fn hybrid_clean_char_x86_64(ch: u32) -> u32;
    
    /// Process character array with AVX-512 vectorization
    fn hybrid_clean_chars_bulk_avx512(
        input: *const u32,
        output: *mut u32,
        len: usize,
    ) -> usize;
    
    /// Process character array with BMI2 optimizations
    fn hybrid_clean_chars_bulk_bmi2(
        input: *const u32,
        output: *mut u32,
        len: usize,
    ) -> usize;
}

// Generic ARM64 functions
#[cfg(feature = "aarch64_assembly")]
extern "C" {
    /// Process single character with generic ARM64 instructions
    fn generic_clean_char_aarch64(ch: u32) -> u32;
    
    /// Process character array with generic ARM64 optimizations
    fn generic_clean_chars_bulk_aarch64(
        input: *const u32,
        output: *mut u32,
        len: usize,
    ) -> usize;
}
```

### Safe Wrapper Implementation

The Rust code provides safe wrappers around unsafe assembly calls:

```rust
impl SafeAssemblyProcessor {
    /// Safely call assembly function with bounds checking and timeout protection
    fn call_assembly_single(&self, ch: char) -> Result<char, AssemblyError> {
        let ch_u32 = ch as u32;
        
        // Validate input
        if ch_u32 > 0x10FFFF {
            return Err(AssemblyError::InvalidInput);
        }
        
        // Call appropriate assembly function based on platform
        let result_u32 = unsafe {
            #[cfg(feature = "apple_silicon_assembly")]
            {
                _apple_hybrid_clean_char_optimized(ch_u32)
            }
            #[cfg(all(feature = "x86_64_assembly", not(feature = "apple_silicon_assembly")))]
            {
                hybrid_clean_char_x86_64(ch_u32)
            }
            #[cfg(all(feature = "aarch64_assembly", not(feature = "apple_silicon_assembly")))]
            {
                generic_clean_char_aarch64(ch_u32)
            }
            #[cfg(feature = "no_assembly")]
            {
                // Fallback to Rust implementation
                return Ok(crate::processor::clean_char_rust_impl(ch));
            }
        };
        
        // Validate output
        char::from_u32(result_u32).ok_or(AssemblyError::ExecutionError(
            format!("Assembly returned invalid Unicode: 0x{:X}", result_u32)
        ))
    }
    
    /// Safely call bulk assembly function with comprehensive safety checks
    fn call_assembly_bulk(&self, input: &[u32], output: &mut [u32]) -> Result<usize, AssemblyError> {
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
        self.control.reset_for_operation(input.len());
        
        let processed = unsafe {
            #[cfg(feature = "apple_silicon_assembly")]
            {
                _apple_hybrid_clean_chars_bulk_neon_optimized(
                    input.as_ptr(),
                    output.as_mut_ptr(),
                    input.len(),
                )
            }
            #[cfg(all(feature = "x86_64_assembly", not(feature = "apple_silicon_assembly")))]
            {
                // Try AVX-512 first, fall back to BMI2
                if is_avx512_available() {
                    hybrid_clean_chars_bulk_avx512(
                        input.as_ptr(),
                        output.as_mut_ptr(),
                        input.len(),
                    )
                } else {
                    hybrid_clean_chars_bulk_bmi2(
                        input.as_ptr(),
                        output.as_mut_ptr(),
                        input.len(),
                    )
                }
            }
            #[cfg(all(feature = "aarch64_assembly", not(feature = "apple_silicon_assembly")))]
            {
                generic_clean_chars_bulk_aarch64(
                    input.as_ptr(),
                    output.as_mut_ptr(),
                    input.len(),
                )
            }
            #[cfg(feature = "no_assembly")]
            {
                // Fallback to Rust implementation
                return self.process_chars_rust(input, output);
            }
        };
        
        // Validate return value
        if processed > input.len() {
            return Err(AssemblyError::ExecutionError(
                format!("Assembly returned invalid count: {} > {}", processed, input.len())
            ));
        }
        
        Ok(processed)
    }
}
```

## Memory Layout and Alignment

### Data Structure Alignment

Assembly functions expect specific memory alignment for optimal performance:

```rust
// Cache-line aligned for optimal performance
#[repr(C, align(64))]
pub struct AssemblyControl {
    // Atomic fields for cross-thread communication
    pub cancel_flag: AtomicBool,        // Offset 0
    pub timeout_flag: AtomicBool,       // Offset 1
    pub panic_flag: AtomicBool,         // Offset 2
    // Padding to next 8-byte boundary
    _pad1: [u8; 5],                     // Offset 3-7
    pub max_iterations: AtomicUsize,    // Offset 8
    pub current_iteration: AtomicUsize, // Offset 16
    pub heartbeat: AtomicU64,           // Offset 24
    pub start_time: AtomicU64,          // Offset 32
    pub timeout_ms: AtomicU64,          // Offset 40
    // Remaining bytes pad to 64-byte cache line
    _pad2: [u8; 16],                    // Offset 48-63
}
```

### Buffer Alignment Requirements

Assembly functions may require specific buffer alignment:

```rust
/// Ensure proper alignment for assembly processing
fn create_aligned_buffers(len: usize) -> (Vec<u32>, Vec<u32>) {
    // Align to 64-byte boundaries for optimal SIMD performance
    let mut input = Vec::with_capacity(len + 16);
    let mut output = Vec::with_capacity(len + 16);
    
    // Ensure alignment
    while (input.as_ptr() as usize) % 64 != 0 {
        input.push(0);
    }
    while (output.as_ptr() as usize) % 64 != 0 {
        output.push(0);
    }
    
    input.resize(len, 0);
    output.resize(len, 0);
    
    (input, output)
}
```

## Error Handling Conventions

### Assembly Error Codes

Assembly functions use consistent error reporting:

```assembly
// ARM64 error handling convention
_apple_hybrid_clean_chars_bulk_neon_optimized:
    // ... input validation ...
    cbz x0, .invalid_input          // Check null input pointer
    cbz x1, .invalid_input          // Check null output pointer
    cbz x2, .empty_input_ok         // Check zero length
    
    // ... processing ...
    
    mov x0, x3                      // Return processed count
    ret
    
.invalid_input:
    mov x0, #0xFFFFFFFFFFFFFFFF     // Return SIZE_MAX on error
    ret
    
.empty_input_ok:
    mov x0, #0                      // Return 0 for empty input
    ret
```

### Rust Error Interpretation

The Rust wrapper interprets assembly return values:

```rust
fn interpret_assembly_result(result: usize, expected_max: usize) -> Result<usize, AssemblyError> {
    match result {
        usize::MAX => Err(AssemblyError::ExecutionError("Invalid input parameters".to_string())),
        n if n > expected_max => Err(AssemblyError::ExecutionError(
            format!("Assembly returned impossible count: {} > {}", n, expected_max)
        )),
        n => Ok(n),
    }
}
```

This calling convention documentation ensures consistent and safe interaction between Rust and assembly code across all supported platforms.
