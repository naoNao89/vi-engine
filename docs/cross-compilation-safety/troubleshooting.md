# Troubleshooting Guide

This guide helps resolve common issues with the vi-rust cross-compilation and assembly safety system.

## Quick Diagnostics

### System Check Commands

```bash
# Check current platform and targets
rustc -vV | grep host
rustup target list --installed

# Verify cross-compilation setup
./scripts/cross-compile.sh check

# Test safety system
cargo test --test safety_tests --target aarch64-apple-darwin
cargo test --test safety_tests --target x86_64-apple-darwin
```

### Environment Verification

```bash
# Check Rust toolchain
rustc --version
cargo --version

# Verify Xcode tools (macOS)
xcode-select --print-path
clang --version

# Check available memory
system_profiler SPHardwareDataType | grep Memory
```

## Common Issues and Solutions

### 1. Cross-Compilation Failures

#### Issue: "Assembly compilation disabled for cross-compilation target"
```
warning: vi@0.7.0: Assembly compilation disabled for cross-compilation target
```

**Status**: ✅ Expected behavior  
**Solution**: This is normal. Cross-compilation automatically uses Rust fallback.

**Verification**:
```bash
cargo build --target x86_64-apple-darwin -v
# Should show the warning and complete successfully
```

#### Issue: Target not installed
```
error: the 'x86_64-apple-darwin' target may not be installed
```

**Solution**:
```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

**Verification**:
```bash
rustup target list --installed | grep -E "(x86_64|aarch64)-apple-darwin"
```

### 2. Assembly Compilation Issues

#### Issue: Assembly syntax errors on native compilation
```
error: unknown directive
```

**Cause**: Assembly syntax incompatibility  
**Solution**: The build system should handle this automatically. If it persists:

```bash
# Clean and rebuild
cargo clean
cargo build --target aarch64-apple-darwin

# Check build script output
cargo build --target aarch64-apple-darwin -v 2>&1 | grep -i assembly
```

#### Issue: Missing Xcode command line tools
```
error: linker `cc` not found
```

**Solution**:
```bash
# Install Xcode command line tools
xcode-select --install

# Verify installation
xcode-select --print-path
which clang
```

### 3. Safety System Issues

#### Issue: Tests failing with timeout errors
```
thread 'test_timeout_protection' panicked at 'Unexpected error: Timeout'
```

**Cause**: System under heavy load or very fast execution  
**Solution**: This is often expected behavior. The test validates timeout protection.

**Verification**:
```bash
# Run individual test with verbose output
cargo test --test safety_tests test_timeout_protection -- --nocapture

# Check if it's consistently failing
for i in {1..5}; do cargo test --test safety_tests test_timeout_protection; done
```

#### Issue: Safety tests failing due to global state
```
test test_large_input_processing ... FAILED
```

**Cause**: Shared global state between tests  
**Solution**: Tests are designed to handle this. If persistent:

```bash
# Run tests individually
cargo test --test safety_tests test_large_input_processing

# Reset and retry
cargo clean
cargo test --test safety_tests
```

### 4. Performance Issues

#### Issue: Significantly slower performance on cross-compilation
```
Expected: ~8M chars/sec, Actual: ~2M chars/sec
```

**Cause**: Debug build or suboptimal configuration  
**Solution**:

```bash
# Use release build
cargo build --target x86_64-apple-darwin --release

# Check optimization level
cargo build --target x86_64-apple-darwin --release -v | grep opt-level

# Run benchmarks
cargo bench --target x86_64-apple-darwin --bench safety_benchmark
```

#### Issue: High safety overhead
```
Safety overhead: >5%
```

**Cause**: Frequent safety checks or debug build  
**Solution**:

```bash
# Use release build
cargo build --release

# Check metrics
cargo run --example safe_processing --release

# Profile if needed
cargo bench --bench safety_benchmark
```

### 5. Feature Flag Warnings

#### Issue: Unexpected cfg condition warnings
```
warning: unexpected `cfg` condition value: `no_assembly`
```

**Status**: ⚠️ Harmless warnings  
**Cause**: Dynamic feature flag generation by build script  
**Solution**: These warnings don't affect functionality. To suppress:

```bash
# Build with warnings suppressed
RUSTFLAGS="-A unexpected_cfgs" cargo build --target x86_64-apple-darwin
```

**Long-term fix**: Add feature flags to Cargo.toml (planned improvement).

### 6. Memory and Resource Issues

#### Issue: High memory usage during compilation
```
error: could not compile due to previous error
```

**Cause**: Insufficient memory for parallel compilation  
**Solution**:

```bash
# Reduce parallel jobs
cargo build --target aarch64-apple-darwin -j 2

# Use less memory-intensive profile
cargo build --target x86_64-apple-darwin --profile x86-sim
```

#### Issue: File descriptor limits
```
error: too many open files
```

**Solution**:
```bash
# Check current limits
ulimit -n

# Increase limit (temporary)
ulimit -n 4096

# Permanent fix (add to ~/.zshrc or ~/.bash_profile)
echo "ulimit -n 4096" >> ~/.zshrc
```

## Advanced Troubleshooting

### Debug Build Analysis

```bash
# Enable verbose build output
RUST_LOG=debug cargo build --target x86_64-apple-darwin -v

# Check feature flags
cargo build --target aarch64-apple-darwin -v 2>&1 | grep "rustc-cfg"

# Analyze assembly compilation
cargo build --target aarch64-apple-darwin -v 2>&1 | grep -A5 -B5 assembly
```

### Safety System Debugging

```rust
// Enable debug logging
use vi::safety::{SafeAssemblyProcessor, GLOBAL_ASSEMBLY_CONTROL};

// Check control state
let control = &*GLOBAL_ASSEMBLY_CONTROL;
println!("Cancel flag: {}", control.cancel_flag.load(std::sync::atomic::Ordering::Relaxed));
println!("Timeout flag: {}", control.timeout_flag.load(std::sync::atomic::Ordering::Relaxed));

// Monitor metrics
let processor = SafeAssemblyProcessor::new();
let result = processor.process_string_safe("test")?;
let metrics = processor.get_metrics();
println!("Operations: started={}, completed={}", 
         metrics.operations_started.load(std::sync::atomic::Ordering::Relaxed),
         metrics.operations_completed.load(std::sync::atomic::Ordering::Relaxed));
```

### Performance Profiling

```bash
# Profile compilation time
time cargo build --target x86_64-apple-darwin

# Profile runtime performance
cargo bench --bench safety_benchmark -- --profile-time=5

# Memory profiling (requires additional tools)
cargo install cargo-profdata
cargo profdata -- cargo bench --bench safety_benchmark
```

## Environment-Specific Issues

### macOS Specific

#### Issue: Code signing errors
```
error: failed to sign executable
```

**Solution**:
```bash
# Disable code signing for development
export CODESIGN_ALLOCATE=/usr/bin/codesign_allocate
export CODE_SIGN_IDENTITY=""
```

#### Issue: Rosetta 2 interference
```
Inconsistent performance between runs
```

**Solution**:
```bash
# Check if running under Rosetta
sysctl -n machdep.cpu.brand_string
arch

# Force native execution
arch -arm64 cargo build --target aarch64-apple-darwin
```

### CI/CD Specific

#### Issue: GitHub Actions failures
```
Error: target may not be installed
```

**Solution**:
```yaml
# Add to GitHub Actions workflow
- name: Install targets
  run: |
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

- name: Run cross-compilation tests
  run: ./scripts/cross-compile.sh test
```

#### Issue: Docker build failures
```
Assembly compilation not supported in container
```

**Solution**: Use cross-compilation mode (assembly automatically disabled).

```dockerfile
# Dockerfile example
FROM rust:latest
RUN rustup target add x86_64-apple-darwin
# Assembly will be automatically disabled
```

## Getting Help

### Diagnostic Information to Collect

When reporting issues, include:

```bash
# System information
rustc -vV
cargo --version
uname -a

# Target information
rustup target list --installed

# Build output
cargo build --target x86_64-apple-darwin -v 2>&1 | head -50

# Test results
cargo test --test safety_tests 2>&1

# Performance data
cargo bench --bench safety_benchmark -- --test 2>&1
```

### Log Collection

```bash
# Enable detailed logging
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run with logging
cargo test --test safety_tests 2>&1 | tee debug.log

# Collect build logs
cargo clean
cargo build --target x86_64-apple-darwin -v 2>&1 | tee build.log
```

### Performance Baseline

```bash
# Establish performance baseline
./scripts/cross-compile.sh bench > baseline.txt

# Compare after changes
./scripts/cross-compile.sh bench > current.txt
diff baseline.txt current.txt
```

## Prevention

### Best Practices

1. **Regular Testing**: Run cross-compilation tests frequently
2. **Clean Builds**: Use `cargo clean` when switching targets
3. **Version Pinning**: Pin Rust version for consistent builds
4. **Documentation**: Keep troubleshooting notes for team

### Monitoring

```bash
# Add to development workflow
alias vi-check="./scripts/cross-compile.sh check"
alias vi-test="./scripts/cross-compile.sh test"
alias vi-bench="cargo bench --bench safety_benchmark"

# Regular health check
vi-check && vi-test && echo "✅ All systems operational"
```

### Automation

```bash
#!/bin/bash
# health-check.sh - Add to cron or CI
set -e

echo "🔍 Running vi-rust health check..."

# Check compilation
./scripts/cross-compile.sh check

# Run safety tests
cargo test --test safety_tests --target aarch64-apple-darwin
cargo test --test safety_tests --target x86_64-apple-darwin

# Quick performance check
timeout 30s cargo bench --bench safety_benchmark -- --test

echo "✅ Health check passed"
```

This troubleshooting guide covers the most common issues. For complex problems, refer to the specific documentation sections or create a detailed issue report with the diagnostic information outlined above.
