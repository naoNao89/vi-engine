# Justfile for vi-engine development
# Provides convenient commands for common development tasks

# Default recipe - show available commands
default:
    @just --list

# Development workflow commands
alias dev := check-all
alias ci := ci-check

# Format code using rustfmt
fmt:
    cargo fmt --all

# Check formatting without making changes
fmt-check:
    cargo fmt --all -- --check

# Run clippy with all features
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with pedantic lints
clippy-pedantic:
    cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings

# Check compilation
check:
    cargo check --all-targets --all-features

# Run all tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Run specific test
test-one TEST:
    cargo test --all-features {{TEST}} -- --nocapture

# Run benchmarks
bench:
    cargo bench

# Run specific benchmark
bench-one BENCH:
    cargo bench {{BENCH}}

# Generate documentation
doc:
    cargo doc --all-features --no-deps --open

# Generate documentation with private items
doc-private:
    cargo doc --all-features --no-deps --document-private-items --open

# Security audit
audit:
    cargo audit

# Check dependencies for issues
deny:
    cargo deny check

# Update dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean

# Full development check (format, clippy, test, doc)
check-all: fmt clippy test doc
    @echo "✅ All checks passed!"

# CI-style check (no formatting changes, strict)
ci-check: fmt-check clippy-pedantic test
    @echo "✅ CI checks passed!"

# Release preparation
release-prep: clean check-all audit deny
    @echo "✅ Ready for release!"

# Install development tools
install-tools:
    cargo install cargo-audit
    cargo install cargo-deny
    cargo install cargo-watch
    cargo install cargo-expand
    rustup component add rustfmt clippy

# Watch for changes and run tests
watch:
    cargo watch -x "test --all-features"

# Watch for changes and run clippy
watch-clippy:
    cargo watch -x "clippy --all-targets --all-features"

# Expand macros for debugging
expand FILE:
    cargo expand --bin {{FILE}}

# Profile with perf (Linux only)
profile:
    cargo build --release
    perf record --call-graph=dwarf target/release/examples/production_usage
    perf report

# Memory profiling with valgrind (Linux only)
memcheck:
    cargo build --release
    valgrind --tool=memcheck --leak-check=full target/release/examples/production_usage

# Cross-compilation check
cross-check:
    cargo check --target aarch64-apple-darwin
    cargo check --target x86_64-apple-darwin
    cargo check --target x86_64-unknown-linux-gnu

# Generate coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --all-features --out Html --output-dir coverage

# Lint shell scripts (requires shellcheck)
lint-scripts:
    find scripts -name "*.sh" -exec shellcheck {} \;

# Check for unused dependencies
unused-deps:
    cargo +nightly udeps --all-targets --all-features

# Spell check documentation (requires typos)
spell-check:
    typos

# Generate changelog (requires git-cliff)
changelog:
    git cliff --output CHANGELOG.md

# Publish dry run
publish-dry:
    cargo publish --dry-run --all-features

# Actual publish (use with caution)
publish:
    cargo publish --all-features
