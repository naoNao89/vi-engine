//! Build script for Profile-Guided Optimization and advanced compiler settings
//!
//! This build script configures advanced compiler optimizations including
//! Profile-Guided Optimization (PGO), CPU-specific optimizations, and
//! link-time optimization for maximum performance.
//!
//! ## Assembly Build Messages
//!
//! By default, assembly compilation messages are suppressed to reduce build noise.
//! To see detailed assembly compilation information, set the environment variable:
//! ```bash
//! VI_BUILD_VERBOSE=1 cargo build
//! ```
//! or use cargo's verbose flag:
//! ```bash
//! cargo build -v
//! ```

use std::env;
use std::path::Path;

fn main() {
    // Get the target architecture and CPU features
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let profile = env::var("PROFILE").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=PROFILE");

    // Configure CPU-specific optimizations
    configure_cpu_optimizations(&target_arch, &target_os, &profile);

    // Configure Profile-Guided Optimization if enabled
    configure_pgo(&profile);

    // Configure advanced link-time optimizations
    configure_lto(&profile);

    // Configure target-specific features
    configure_target_features(&target_arch);

    // Auto-enable assembly features based on platform
    configure_auto_assembly(&target_arch, &target_os);

    // Compile assembly kernels for hybrid optimization
    compile_assembly_kernels(&target_arch, &profile);
}

fn configure_cpu_optimizations(target_arch: &str, target_os: &str, profile: &str) {
    // Only apply aggressive optimizations for release builds
    if profile != "release" && profile != "pgo" && profile != "ultra" {
        return;
    }

    match target_arch {
        "x86_64" => {
            // Enable x86_64 specific optimizations
            println!("cargo:rustc-link-arg=-march=native");
            println!("cargo:rustc-link-arg=-mtune=native");

            // Enable specific CPU features for maximum performance
            println!("cargo:rustc-cfg=feature=\"avx2\"");
            println!("cargo:rustc-cfg=feature=\"bmi2\"");
            println!("cargo:rustc-cfg=feature=\"fma\"");

            // Platform-specific optimizations
            match target_os {
                "linux" => {
                    println!("cargo:rustc-link-arg=-Wl,--gc-sections");
                    println!("cargo:rustc-link-arg=-Wl,--strip-all");
                }
                "macos" => {
                    println!("cargo:rustc-link-arg=-Wl,-dead_strip");
                }
                "windows" => {
                    println!("cargo:rustc-link-arg=/OPT:REF");
                    println!("cargo:rustc-link-arg=/OPT:ICF");
                }
                _ => {}
            }
        }
        "aarch64" => {
            // ARM64 specific optimizations
            println!("cargo:rustc-link-arg=-mcpu=native");
            println!("cargo:rustc-cfg=feature=\"neon\"");
        }
        _ => {
            // Generic optimizations for other architectures
            println!("cargo:rustc-link-arg=-O3");
        }
    }
}

fn configure_pgo(profile: &str) {
    if profile == "pgo" {
        // Profile-Guided Optimization configuration
        println!("cargo:rustc-env=RUSTFLAGS=-Cprofile-generate=/tmp/pgo-data");
        println!(
            "cargo:warning=PGO profile generation enabled. Run benchmarks to collect profile data."
        );
        println!("cargo:warning=After collecting data, rebuild with: RUSTFLAGS=\"-Cprofile-use=/tmp/pgo-data\" cargo build --profile=pgo");
    }

    // Check if PGO profile data exists and use it
    if std::path::Path::new("/tmp/pgo-data").exists() && profile == "release" {
        println!("cargo:rustc-env=RUSTFLAGS=-Cprofile-use=/tmp/pgo-data");
        println!("cargo:warning=Using PGO profile data for optimization");
    }
}

fn configure_lto(profile: &str) {
    match profile {
        "release" | "pgo" | "ultra" => {
            // Enable aggressive Link-Time Optimization
            println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_LTO=fat");
            println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1");
            println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_PANIC=abort");
        }
        _ => {}
    }
}

fn configure_target_features(target_arch: &str) {
    match target_arch {
        "x86_64" => {
            // Configure x86_64 specific optimizations
            println!("cargo:rustc-cfg=target_arch_x86_64");
        }
        "aarch64" => {
            // Configure ARM64 specific optimizations
            println!("cargo:rustc-cfg=target_arch_aarch64");
        }
        _ => {}
    }
}

fn configure_auto_assembly(target_arch: &str, target_os: &str) {
    // Check if no_assembly feature is explicitly enabled
    let no_assembly = env::var("CARGO_FEATURE_NO_ASSEMBLY").is_ok();
    if no_assembly {
        // Only show this message in verbose mode to avoid build warnings
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!("cargo:warning=Assembly explicitly disabled via no_assembly feature");
        }
        return;
    }

    // Check if auto_assembly feature is enabled
    let auto_assembly = env::var("CARGO_FEATURE_AUTO_ASSEMBLY").is_ok();

    if !auto_assembly {
        return;
    }

    // Automatically enable appropriate assembly features based on platform
    match (target_arch, target_os) {
        ("aarch64", "macos") => {
            // Apple Silicon - enable both generic aarch64 and Apple Silicon specific
            println!("cargo:rustc-cfg=feature=\"aarch64_assembly\"");
            println!("cargo:rustc-cfg=feature=\"apple_silicon_assembly\"");
            // Only show this message when explicitly requested via environment variable
            if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
                println!("cargo:warning=Auto-enabled Apple Silicon assembly optimizations");
            }
        }
        ("aarch64", _) => {
            // Generic ARM64 (Linux, Android, etc.)
            println!("cargo:rustc-cfg=feature=\"aarch64_assembly\"");
            if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
                println!("cargo:warning=Auto-enabled generic ARM64 assembly optimizations");
            }
        }
        ("x86_64", _) => {
            // x86_64 on any OS
            println!("cargo:rustc-cfg=feature=\"x86_64_assembly\"");
            if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
                println!("cargo:warning=Auto-enabled x86_64 assembly optimizations");
            }
        }
        _ => {
            // Unsupported architecture - use Rust fallback
            println!("cargo:rustc-cfg=feature=\"no_assembly\"");
            println!(
                "cargo:warning=Assembly not supported for {target_arch}-{target_os}, using Rust fallback"
            );
        }
    }
}

fn compile_assembly_kernels(target_arch: &str, _profile: &str) {
    // Skip assembly compilation if no_assembly feature is enabled
    let no_assembly = env::var("CARGO_FEATURE_NO_ASSEMBLY").is_ok();
    if no_assembly {
        // Only show this message in verbose mode to avoid build warnings
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!("cargo:warning=Skipping assembly compilation due to no_assembly feature");
        }
        return;
    }

    // Always compile assembly for now since we need it for tests
    let should_compile = true;

    if !should_compile {
        return;
    }

    match target_arch {
        "x86_64" => {
            compile_x86_64_assembly();
        }
        "aarch64" => {
            compile_aarch64_assembly();
        }
        _ => {
            println!(
                "cargo:warning=Assembly kernels not available for architecture: {target_arch}"
            );
        }
    }
}

fn compile_x86_64_assembly() {
    let asm_file = "src/asm/x86_64_kernels.s";
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_cross_compile = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default()
        != env::var("HOST")
            .unwrap_or_default()
            .split('-')
            .next()
            .unwrap_or("");

    // For now, disable assembly compilation for cross-compilation to avoid syntax issues
    if is_cross_compile {
        println!("cargo:warning=Assembly compilation disabled for cross-compilation target");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
        return;
    }

    if Path::new(asm_file).exists() {
        println!("cargo:rerun-if-changed={asm_file}");

        // Use cc crate to compile assembly with minimal flags to avoid warnings
        let mut build = cc::Build::new();
        build.file(asm_file).flag("-x").flag("assembler");

        // Use appropriate flags based on target and cross-compilation
        if target_os == "macos" {
            // macOS uses different assembly syntax - don't use --64 flag
            if is_cross_compile {
                build.flag("-march=x86-64");
            } else {
                // Add CPU-specific optimizations only if supported and not cross-compiling
                if is_feature_available("bmi2") {
                    build.flag("-mbmi2");
                }
                if is_feature_available("avx512f") {
                    build.flag("-mavx512f");
                }
            }
        } else {
            // Linux and other platforms use -m64 flag for 64-bit assembly
            build.flag("-m64");
            if is_feature_available("bmi2") {
                build.flag("-mbmi2");
            }
            if is_feature_available("avx512f") {
                build.flag("-mavx512f");
            }
        }

        build.flag("-w"); // Suppress all warnings for assembly compilation

        build.compile("x86_64_kernels");

        println!("cargo:rustc-link-lib=static=x86_64_kernels");
        println!("cargo:rustc-cfg=feature=\"assembly_kernels\"");
        println!("cargo:rustc-cfg=feature=\"x86_64_assembly\"");
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!("cargo:warning=Compiled x86_64 assembly kernels for maximum performance");
        }
    } else {
        println!("cargo:warning=x86_64 assembly file not found: {asm_file}");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
    }
}

fn compile_aarch64_assembly() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let _target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let asm_file = "src/asm/aarch64_kernels.s";
    let apple_silicon_asm_file = "src/asm/aarch64_apple_silicon.s";
    let generic_asm_file = "src/asm/aarch64_generic.s";
    let optimized_asm_file = "src/asm/optimized_aarch64.s";

    // Determine which assembly files to compile based on target
    let is_apple_silicon = target_os == "macos";
    let is_cross_compile = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default()
        != env::var("HOST")
            .unwrap_or_default()
            .split('-')
            .next()
            .unwrap_or("");

    // For now, disable assembly compilation for cross-compilation to avoid syntax issues
    if is_cross_compile {
        println!("cargo:warning=Assembly compilation disabled for cross-compilation target");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
        return;
    }

    compile_generic_aarch64_assembly(generic_asm_file, is_cross_compile);
    compile_standard_aarch64_assembly(asm_file, is_cross_compile);
    compile_optimized_aarch64_assembly(optimized_asm_file, is_cross_compile);
    compile_apple_silicon_assembly(apple_silicon_asm_file, is_apple_silicon, is_cross_compile);
}

fn compile_generic_aarch64_assembly(generic_asm_file: &str, is_cross_compile: bool) {
    // Compile generic ARM64 assembly (works on all ARM64 platforms)
    // Temporarily disabled due to assembly syntax issues on macOS
    #[allow(clippy::overly_complex_bool_expr)]
    if false && Path::new(generic_asm_file).exists() {
        println!("cargo:rerun-if-changed={generic_asm_file}");

        let mut build = cc::Build::new();
        build.file(generic_asm_file).flag("-x").flag("assembler");

        // Use generic flags for cross-compilation
        if is_cross_compile {
            build.flag("-march=armv8-a");
        } else {
            build.flag("-mcpu=native");
        }

        build.flag("-w"); // Suppress all warnings for assembly compilation

        build.compile("aarch64_generic");

        println!("cargo:rustc-link-lib=static=aarch64_generic");
        println!("cargo:rustc-cfg=feature=\"assembly_kernels\"");
        println!("cargo:rustc-cfg=feature=\"aarch64_generic_assembly\"");
        println!("cargo:warning=Compiled generic aarch64 assembly kernels for cross-platform compatibility");
    } else {
        // Only show this as a warning in verbose mode, otherwise it's just informational
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!(
                "cargo:warning=Generic ARM64 assembly disabled due to syntax compatibility issues"
            );
        }
    }
}

fn compile_standard_aarch64_assembly(asm_file: &str, is_cross_compile: bool) {
    // Compile standard ARM64 assembly
    if Path::new(asm_file).exists() {
        println!("cargo:rerun-if-changed={asm_file}");

        let mut build = cc::Build::new();
        build.file(asm_file).flag("-x").flag("assembler");

        if is_cross_compile {
            build.flag("-march=armv8-a");
        } else {
            build.flag("-mcpu=native");
        }

        build.flag("-w"); // Suppress all warnings for assembly compilation

        build.compile("aarch64_kernels");

        println!("cargo:rustc-link-lib=static=aarch64_kernels");
        println!("cargo:rustc-cfg=feature=\"assembly_kernels\"");
        println!("cargo:rustc-cfg=feature=\"aarch64_assembly\"");
        // Only show compilation success in verbose mode
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!("cargo:warning=Compiled aarch64 assembly kernels for maximum performance");
        }
    } else {
        println!("cargo:warning=aarch64 assembly file not found: {asm_file}");
        println!("cargo:rustc-cfg=feature=\"no_assembly\"");
    }
}

fn compile_optimized_aarch64_assembly(optimized_asm_file: &str, is_cross_compile: bool) {
    // Compile optimized assembly (enhanced SIMD implementation)
    if Path::new(optimized_asm_file).exists() {
        println!("cargo:rerun-if-changed={optimized_asm_file}");

        let mut build = cc::Build::new();
        build.file(optimized_asm_file).flag("-x").flag("assembler");

        if is_cross_compile {
            build.flag("-march=armv8-a");
        } else {
            build.flag("-mcpu=native");
        }

        build.flag("-w"); // Suppress all warnings for assembly compilation

        build.compile("optimized_aarch64");

        println!("cargo:rustc-link-lib=static=optimized_aarch64");
        println!("cargo:rustc-cfg=feature=\"optimized_assembly\"");
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!(
                "cargo:warning=Compiled optimized aarch64 assembly kernels with enhanced SIMD"
            );
        }
    }
}

fn compile_apple_silicon_assembly(
    apple_silicon_asm_file: &str,
    is_apple_silicon: bool,
    is_cross_compile: bool,
) {
    // Compile Apple Silicon optimized assembly (macOS ARM64 only, not for cross-compilation)
    if is_apple_silicon && !is_cross_compile && Path::new(apple_silicon_asm_file).exists() {
        println!("cargo:rerun-if-changed={apple_silicon_asm_file}");

        let mut build = cc::Build::new();
        build
            .file(apple_silicon_asm_file)
            .flag("-x")
            .flag("assembler")
            .flag("-mcpu=native")
            .flag("-w"); // Suppress all warnings for assembly compilation

        // Use a different library name to avoid conflicts
        build.compile("apple_silicon_kernels");

        println!("cargo:rustc-link-lib=static=apple_silicon_kernels");
        println!("cargo:rustc-cfg=feature=\"apple_silicon_assembly\"");
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!("cargo:warning=Compiled Apple Silicon optimized assembly kernels for ultra-high performance");
        }
    } else if is_apple_silicon && !is_cross_compile {
        // Only show this as a warning in verbose mode
        if env::var("VI_BUILD_VERBOSE").is_ok() || env::var("CARGO_VERBOSE").is_ok() {
            println!(
                "cargo:warning=Apple Silicon assembly file not found: {apple_silicon_asm_file}"
            );
        }
    }
}

fn is_feature_available(feature: &str) -> bool {
    // Simple feature detection - in a real implementation this would
    // check CPU capabilities more thoroughly
    match feature {
        "bmi2" | "avx512f" => true, // Assume available for now
        _ => false,
    }
}

#[allow(dead_code)]
fn generate_build_info() {
    use std::process::Command;

    // Generate build information for performance tracking
    let git_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());

    let build_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=BUILD_TIME={build_time}");

    // Generate optimization report
    let optimization_level = env::var("CARGO_PROFILE_RELEASE_OPT_LEVEL").unwrap_or_default();
    let lto_enabled = env::var("CARGO_PROFILE_RELEASE_LTO").unwrap_or_default();

    println!("cargo:rustc-env=OPTIMIZATION_LEVEL={optimization_level}");
    println!("cargo:rustc-env=LTO_ENABLED={lto_enabled}");
}

// Helper function to detect CPU capabilities at build time
fn _detect_cpu_capabilities() -> Vec<String> {
    // CPU detection would go here in a real implementation
    // For now, return empty vector to avoid compilation issues
    Vec::new()
}

// Generate compile-time optimization constants
#[allow(dead_code)]
fn generate_optimization_constants() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:rustc-cfg=optimization_target=\"x86_64\"");
            println!("cargo:rustc-cfg=simd_width=\"256\""); // AVX2
            println!("cargo:rustc-cfg=cache_line_size=\"64\"");
        }
        "aarch64" => {
            println!("cargo:rustc-cfg=optimization_target=\"aarch64\"");
            println!("cargo:rustc-cfg=simd_width=\"128\""); // NEON
            println!("cargo:rustc-cfg=cache_line_size=\"64\"");
        }
        _ => {
            println!("cargo:rustc-cfg=optimization_target=\"generic\"");
            println!("cargo:rustc-cfg=simd_width=\"0\"");
            println!("cargo:rustc-cfg=cache_line_size=\"64\"");
        }
    }
}

// Configure benchmark-specific optimizations
#[allow(dead_code)]
fn configure_benchmark_optimizations() {
    if env::var("CARGO_CFG_FEATURE")
        .unwrap_or_default()
        .contains("bench")
    {
        // Enable additional optimizations for benchmarking
        println!("cargo:rustc-cfg=benchmark_mode");
        println!("cargo:rustc-link-arg=-flto");
        println!("cargo:rustc-link-arg=-fuse-ld=lld"); // Use LLD linker for faster linking
    }
}

// Generate performance tuning constants based on target
#[allow(dead_code)]
fn generate_performance_constants() {
    let profile = env::var("PROFILE").unwrap_or_default();

    match profile.as_str() {
        "ultra" => {
            println!("cargo:rustc-cfg=performance_mode=\"ultra\"");
            println!("cargo:rustc-cfg=inline_threshold=\"1000\"");
            println!("cargo:rustc-cfg=unroll_loops");
        }
        "pgo" => {
            println!("cargo:rustc-cfg=performance_mode=\"pgo\"");
            println!("cargo:rustc-cfg=inline_threshold=\"500\"");
        }
        "release" => {
            println!("cargo:rustc-cfg=performance_mode=\"release\"");
            println!("cargo:rustc-cfg=inline_threshold=\"100\"");
        }
        _ => {
            println!("cargo:rustc-cfg=performance_mode=\"debug\"");
        }
    }
}
