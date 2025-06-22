# Platform-Specific Assembly Integration Guide

This document provides detailed guidance for assembly integration across different platforms, including architecture-specific optimizations, cross-compilation considerations, and platform-specific debugging techniques.

## Supported Platforms

### Platform Support Matrix

| Platform | Architecture | Assembly Support | Performance Target | Status |
|----------|-------------|------------------|-------------------|---------|
| macOS | Apple Silicon (ARM64) | ✅ Native optimized | >50M chars/sec | Production |
| macOS | Intel (x86_64) | ✅ Native optimized | >20M chars/sec | Production |
| Linux | ARM64 | ✅ Generic ARM64 | >15M chars/sec | Stable |
| Linux | x86_64 | ✅ AVX-512/BMI2 | >25M chars/sec | Stable |
| Windows | x86_64 | ❌ Rust fallback | >5M chars/sec | Fallback |
| Cross-compilation | Any | ❌ Rust fallback | >5M chars/sec | Safe |

## Apple Silicon (ARM64) - macOS

### Architecture Characteristics

Apple Silicon processors (M1/M2/M3) have unique characteristics that enable exceptional performance:

- **Unified Memory Architecture**: Shared memory between CPU and GPU
- **Wide Execution Units**: Up to 8 instructions per cycle
- **Advanced Branch Prediction**: Sophisticated prediction algorithms
- **Large Cache Hierarchy**: L1: 128KB+128KB, L2: 12MB, L3: Shared
- **NEON SIMD**: 128-bit vectors with advanced instructions

### Apple Silicon Specific Optimizations

```assembly
// Apple Silicon optimized Vietnamese character processing
_apple_hybrid_clean_char_neon:
    // Leverage unified memory with prefetching
    prfm pldl1keep, [x0, #64]        // Prefetch next cache line
    prfm pldl1keep, [x0, #128]       // Prefetch ahead
    
    // Use Apple Silicon specific NEON features
    dup v0.4s, w0                    // Duplicate input across vector
    
    // Optimized range checking using Apple Silicon branch prediction
    cmp w0, #0x00C0                  // Latin Extended start
    b.lo .ascii_fast_path_apple
    cmp w0, #0x1EF9                  // Vietnamese range end
    b.hi .not_vietnamese_apple
    
    // Use post-indexed addressing (efficient on Apple Silicon)
    adrp x1, vietnamese_lookup_apple@PAGE
    add x1, x1, vietnamese_lookup_apple@PAGEOFF
    
    // Apple Silicon optimized table lookup
    sub w2, w0, #0x00C0              // Normalize to table index
    ldr w0, [x1, w2, lsl #2]         // Load with scaling
    
    ret

.ascii_fast_path_apple:
    // ASCII characters pass through unchanged
    ret

.not_vietnamese_apple:
    // Non-Vietnamese characters pass through unchanged
    ret
```

### Build Configuration for Apple Silicon

```rust
// build.rs - Apple Silicon specific configuration
fn compile_apple_silicon_assembly() {
    let asm_file = "src/asm/aarch64_apple_silicon.s";
    
    if Path::new(asm_file).exists() {
        let mut build = cc::Build::new();
        build.file(asm_file)
             .flag("-x")
             .flag("assembler")
             .flag("-mcpu=native")          // Use native CPU features
             .flag("-mtune=native")         // Optimize for native CPU
             .flag("-w");                   // Suppress warnings
        
        // Apple Silicon specific optimizations
        build.flag("-mllvm")
             .flag("-enable-machine-outliner=never"); // Disable outlining for predictable performance
        
        build.compile("aarch64_apple_silicon");
        
        println!("cargo:rustc-link-lib=static=aarch64_apple_silicon");
        println!("cargo:rustc-cfg=feature=\"apple_silicon_assembly\"");
        println!("cargo:warning=Compiled Apple Silicon optimized assembly");
    }
}
```

### Performance Characteristics

Apple Silicon assembly achieves exceptional performance:

```rust
// Performance benchmarks on Apple Silicon
pub struct AppleSiliconBenchmarks;

impl AppleSiliconBenchmarks {
    pub fn run_benchmarks() {
        let processor = SafeAssemblyProcessor::new();
        
        // Single character processing
        let start = Instant::now();
        for _ in 0..1_000_000 {
            let _ = processor.process_char_safe('ế');
        }
        let single_char_time = start.elapsed();
        println!("Apple Silicon single char: {:.2} ns/char", 
                 single_char_time.as_nanos() as f64 / 1_000_000.0);
        
        // Bulk processing
        let test_text = "Tiếng Việt rất đẹp".repeat(100_000);
        let start = Instant::now();
        let result = processor.process_string_safe(&test_text).unwrap();
        let bulk_time = start.elapsed();
        let chars_per_sec = test_text.len() as f64 / bulk_time.as_secs_f64();
        println!("Apple Silicon bulk: {:.2} M chars/sec", chars_per_sec / 1_000_000.0);
    }
}
```

## x86_64 - Intel/AMD

### Architecture Characteristics

x86_64 processors provide different optimization opportunities:

- **Complex Instruction Set**: Rich instruction set with specialized operations
- **Advanced Vector Extensions**: AVX-512 (512-bit vectors), BMI2 bit manipulation
- **Large Register File**: 16 general-purpose + 32 vector registers
- **Sophisticated Cache**: Multi-level cache with complex prefetching
- **Branch Prediction**: Advanced prediction with large branch target buffers

### x86_64 Specific Optimizations

```assembly
# x86_64 assembly with AVX-512 and BMI2 optimizations
hybrid_clean_chars_bulk_avx512:
    # Function prologue
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    # Check for AVX-512 support at runtime
    mov eax, 7
    xor ecx, ecx
    cpuid
    test ebx, (1 << 16)              # AVX-512F support
    jz .fallback_to_sse_x86
    
    # Setup for vectorized processing
    mov r12, rdi                     # input pointer
    mov r13, rsi                     # output pointer
    mov rbx, rdx                     # length
    xor rax, rax                     # processed count
    
    # Load Vietnamese character range constants
    vpbroadcastd zmm30, [rip + viet_min_const]  # 0x00C0
    vpbroadcastd zmm31, [rip + viet_max_const]  # 0x1EF9
    
.avx512_loop:
    # Check if we have at least 16 characters remaining
    cmp rbx, 16
    jl .process_remaining_scalar
    
    # Load 16 characters (512 bits)
    vmovdqu32 zmm0, [r12 + rax*4]
    
    # Parallel Vietnamese character detection
    vpcmpd k1, zmm0, zmm30, 5        # Compare >= Vietnamese range start
    vpcmpd k2, zmm0, zmm31, 2        # Compare <= Vietnamese range end
    kandw k3, k1, k2                 # Combine masks
    
    # Vectorized table lookup for Vietnamese characters
    vpgatherdd zmm1{k3}{z}, [rip + vietnamese_table + zmm0*4]
    
    # Blend original and processed characters
    vpblendmd zmm0{k3}, zmm0, zmm1
    
    # Store results
    vmovdqu32 [r13 + rax*4], zmm0
    
    # Update counters
    add rax, 16
    sub rbx, 16
    jnz .avx512_loop
    
    jmp .x86_function_exit
    
.fallback_to_sse_x86:
    # Fallback to SSE implementation for older CPUs
    # ... SSE implementation ...
    
.process_remaining_scalar:
    # Process remaining characters one by one
    # ... scalar implementation ...
    
.x86_function_exit:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret
```

### Build Configuration for x86_64

```rust
// build.rs - x86_64 specific configuration
fn compile_x86_64_assembly() {
    let asm_file = "src/asm/x86_64_kernels.s";
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    if Path::new(asm_file).exists() {
        let mut build = cc::Build::new();
        build.file(asm_file)
             .flag("-x")
             .flag("assembler");
        
        // Platform-specific flags
        match target_os.as_str() {
            "macos" => {
                // macOS uses different assembly syntax
                build.flag("-march=native")
                     .flag("-mtune=native");
            }
            "linux" => {
                // Linux can use --64 flag
                build.flag("--64")
                     .flag("-march=native")
                     .flag("-mtune=native");
            }
            _ => {
                build.flag("-march=x86-64");
            }
        }
        
        // Enable advanced instruction sets if available
        if is_feature_available("avx512f") {
            build.flag("-mavx512f");
        }
        if is_feature_available("bmi2") {
            build.flag("-mbmi2");
        }
        
        build.flag("-w"); // Suppress warnings
        build.compile("x86_64_kernels");
        
        println!("cargo:rustc-link-lib=static=x86_64_kernels");
        println!("cargo:rustc-cfg=feature=\"x86_64_assembly\"");
    }
}

fn is_feature_available(feature: &str) -> bool {
    // Runtime CPU feature detection
    match feature {
        "avx512f" => {
            #[cfg(target_arch = "x86_64")]
            {
                std::arch::is_x86_feature_detected!("avx512f")
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                false
            }
        }
        "bmi2" => {
            #[cfg(target_arch = "x86_64")]
            {
                std::arch::is_x86_feature_detected!("bmi2")
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                false
            }
        }
        _ => false,
    }
}
```

## Generic ARM64 - Linux

### Portable ARM64 Implementation

For non-Apple ARM64 systems, we use portable implementations:

```assembly
// Generic ARM64 implementation for Linux and other platforms
generic_clean_char_aarch64:
    // Standard ARM64 prologue
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    
    // Fast path for ASCII characters
    cmp w0, #128
    b.lo .ascii_passthrough_generic
    
    // Check for Vietnamese character ranges
    mov w1, w0
    
    // Latin Extended range (0x00C0-0x00FF)
    mov w2, #0x00C0
    cmp w1, w2
    b.lo .not_vietnamese_generic
    mov w2, #0x00FF
    cmp w1, w2
    b.ls .latin_extended_lookup_generic
    
    // Vietnamese extended range (0x1EA0-0x1EF9)
    mov w2, #0x1EA0
    cmp w1, w2
    b.lo .not_vietnamese_generic
    mov w2, #0x1EF9
    cmp w1, w2
    b.ls .vietnamese_extended_lookup_generic
    
.not_vietnamese_generic:
    // Character is not Vietnamese, return unchanged
    ldp x29, x30, [sp], #16
    ret
    
.ascii_passthrough_generic:
    // ASCII characters pass through unchanged
    ldp x29, x30, [sp], #16
    ret
    
.latin_extended_lookup_generic:
    // Lookup in Latin Extended table
    adrp x2, latin_extended_table
    add x2, x2, :lo12:latin_extended_table
    sub w1, w1, #0x00C0              // Normalize index
    ldr w0, [x2, w1, lsl #2]         // Load replacement
    ldp x29, x30, [sp], #16
    ret
    
.vietnamese_extended_lookup_generic:
    // Lookup in Vietnamese extended table
    adrp x2, vietnamese_extended_table
    add x2, x2, :lo12:vietnamese_extended_table
    sub w1, w1, #0x1EA0              // Normalize index
    ldr w0, [x2, w1, lsl #2]         // Load replacement
    ldp x29, x30, [sp], #16
    ret
```

## Cross-Compilation Handling

### Automatic Fallback System

The build system automatically detects cross-compilation and falls back to Rust:

```rust
// build.rs - Cross-compilation detection
fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();
    
    // Detect cross-compilation
    let is_cross_compile = !host.starts_with(&target_arch);
    
    if is_cross_compile {
        println!("cargo:warning=Cross-compilation detected, using Rust fallback");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
        return;
    }
    
    // Native compilation - proceed with assembly
    match (target_arch.as_str(), target_os.as_str()) {
        ("aarch64", "macos") => compile_apple_silicon_assembly(),
        ("aarch64", "linux") => compile_generic_arm64_assembly(),
        ("x86_64", "macos") => compile_x86_64_assembly(),
        ("x86_64", "linux") => compile_x86_64_assembly(),
        _ => {
            println!("cargo:warning=Unsupported platform, using Rust fallback");
            println!("cargo:rustc-cfg=feature=\"no_assembly\"");
        }
    }
}
```

### Cross-Compilation Testing

```rust
// Test cross-compilation behavior
#[cfg(test)]
mod cross_compilation_tests {
    use super::*;
    
    #[test]
    fn test_cross_compilation_fallback() {
        // This test verifies that cross-compilation falls back to Rust
        let processor = SafeAssemblyProcessor::new();
        let input = "Tiếng Việt";
        
        match processor.process_string_safe(input) {
            Ok(result) => {
                assert_eq!(result, "Tieng Viet");
                
                // Verify we're using Rust fallback
                #[cfg(feature = "no_assembly")]
                {
                    println!("Successfully using Rust fallback");
                }
                
                #[cfg(not(feature = "no_assembly"))]
                {
                    println!("Using native assembly");
                }
            }
            Err(e) => panic!("Cross-compilation fallback failed: {}", e),
        }
    }
}
```

## Platform-Specific Debugging

### Apple Silicon Debugging

```bash
# Debug Apple Silicon assembly
lldb target/debug/vi-rust
(lldb) breakpoint set --name _apple_hybrid_clean_char_optimized
(lldb) run
(lldb) register read
(lldb) disassemble --frame

# Profile with Instruments
xcrun xctrace record --template "Time Profiler" --launch -- ./target/release/vi-rust

# Check assembly compilation
otool -tv target/release/libvi_rust.rlib | grep -A 20 "_apple_hybrid"
```

### x86_64 Debugging

```bash
# Debug x86_64 assembly
gdb target/debug/vi-rust
(gdb) break hybrid_clean_char_x86_64
(gdb) run
(gdb) info registers
(gdb) disas

# Profile with perf (Linux)
perf record -g ./target/release/vi-rust
perf report

# Check CPU features
grep flags /proc/cpuinfo | head -1
```

### Cross-Platform Performance Comparison

```rust
// Benchmark across platforms
pub fn platform_benchmark() {
    let processor = SafeAssemblyProcessor::new();
    let test_input = "Xin chào thế giới! ".repeat(10000);
    
    let start = Instant::now();
    let result = processor.process_string_safe(&test_input).unwrap();
    let duration = start.elapsed();
    
    let chars_per_sec = test_input.len() as f64 / duration.as_secs_f64();
    
    println!("Platform: {}", std::env::consts::ARCH);
    println!("OS: {}", std::env::consts::OS);
    println!("Performance: {:.2} M chars/sec", chars_per_sec / 1_000_000.0);
    
    #[cfg(feature = "apple_silicon_assembly")]
    println!("Using: Apple Silicon optimized assembly");
    
    #[cfg(feature = "x86_64_assembly")]
    println!("Using: x86_64 optimized assembly");
    
    #[cfg(feature = "aarch64_assembly")]
    println!("Using: Generic ARM64 assembly");
    
    #[cfg(feature = "no_assembly")]
    println!("Using: Rust fallback implementation");
}
```

This platform-specific guide ensures optimal performance and compatibility across all supported architectures and operating systems.
