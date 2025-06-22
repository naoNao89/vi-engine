#!/bin/bash
# Cross-compilation and testing script for vi-engine
# Enables building and testing for multiple architectures on Mac ARM

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a target is installed
check_target() {
    local target=$1
    if rustup target list --installed | grep -q "$target"; then
        return 0
    else
        return 1
    fi
}

# Function to install target if not present
install_target() {
    local target=$1
    if ! check_target "$target"; then
        print_status "Installing target: $target"
        rustup target add "$target"
        print_success "Target $target installed"
    else
        print_status "Target $target already installed"
    fi
}

# Function to build for a specific target
build_target() {
    local target=$1
    local profile=${2:-release}
    
    print_status "Building for target: $target (profile: $profile)"
    
    if cargo build --target "$target" --profile "$profile"; then
        print_success "Build successful for $target"
        return 0
    else
        print_error "Build failed for $target"
        return 1
    fi
}

# Function to test for a specific target (if possible)
test_target() {
    local target=$1
    
    print_status "Testing for target: $target"
    
    # Only test targets that can run on the current platform
    case "$target" in
        "aarch64-apple-darwin"|"x86_64-apple-darwin")
            if cargo test --target "$target" --lib; then
                print_success "Tests passed for $target"
                return 0
            else
                print_error "Tests failed for $target"
                return 1
            fi
            ;;
        *)
            print_warning "Skipping tests for $target (cross-compilation target)"
            return 0
            ;;
    esac
}

# Function to check compilation for a target
check_target_compilation() {
    local target=$1
    
    print_status "Checking compilation for target: $target"
    
    if cargo check --target "$target"; then
        print_success "Compilation check passed for $target"
        return 0
    else
        print_error "Compilation check failed for $target"
        return 1
    fi
}

# Function to run benchmarks for a target
benchmark_target() {
    local target=$1
    
    print_status "Running benchmarks for target: $target"
    
    # Only benchmark targets that can run on the current platform
    case "$target" in
        "aarch64-apple-darwin"|"x86_64-apple-darwin")
            if cargo bench --target "$target" --bench safety_benchmark -- --test; then
                print_success "Benchmarks completed for $target"
                return 0
            else
                print_error "Benchmarks failed for $target"
                return 1
            fi
            ;;
        *)
            print_warning "Skipping benchmarks for $target (cross-compilation target)"
            return 0
            ;;
    esac
}

# Main function
main() {
    local command=${1:-all}
    
    print_status "Vi-rust Cross-Compilation Script"
    print_status "Current platform: $(rustc -vV | grep host | cut -d' ' -f2)"
    
    # Define targets (focusing on macOS targets that work out of the box)
    local targets=(
        "aarch64-apple-darwin"
        "x86_64-apple-darwin"
    )

    # Linux targets require cross-compilation toolchains
    local linux_targets=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
    )
    
    case "$command" in
        "install")
            print_status "Installing cross-compilation targets..."
            for target in "${targets[@]}"; do
                install_target "$target"
            done
            print_success "All targets installed"
            ;;
            
        "build")
            local target_filter=${2:-"all"}
            print_status "Building targets..."
            
            local build_failed=0
            for target in "${targets[@]}"; do
                if [[ "$target_filter" == "all" || "$target" == "$target_filter" ]]; then
                    install_target "$target"
                    
                    # Choose appropriate profile
                    local profile="release"
                    case "$target" in
                        "x86_64-apple-darwin") profile="x86-sim" ;;
                        "x86_64-unknown-linux-gnu") profile="release-cross" ;;
                        "aarch64-unknown-linux-gnu") profile="arm-generic" ;;
                    esac
                    
                    if ! build_target "$target" "$profile"; then
                        build_failed=1
                    fi
                fi
            done
            
            if [[ $build_failed -eq 0 ]]; then
                print_success "All builds completed successfully"
            else
                print_error "Some builds failed"
                exit 1
            fi
            ;;
            
        "test")
            local target_filter=${2:-"all"}
            print_status "Testing targets..."
            
            local test_failed=0
            for target in "${targets[@]}"; do
                if [[ "$target_filter" == "all" || "$target" == "$target_filter" ]]; then
                    if ! test_target "$target"; then
                        test_failed=1
                    fi
                fi
            done
            
            if [[ $test_failed -eq 0 ]]; then
                print_success "All tests completed successfully"
            else
                print_error "Some tests failed"
                exit 1
            fi
            ;;
            
        "check")
            print_status "Checking compilation for all targets..."
            
            local check_failed=0
            for target in "${targets[@]}"; do
                install_target "$target"
                if ! check_target_compilation "$target"; then
                    check_failed=1
                fi
            done
            
            if [[ $check_failed -eq 0 ]]; then
                print_success "All compilation checks passed"
            else
                print_error "Some compilation checks failed"
                exit 1
            fi
            ;;
            
        "bench")
            local target_filter=${2:-"all"}
            print_status "Running benchmarks..."
            
            for target in "${targets[@]}"; do
                if [[ "$target_filter" == "all" || "$target" == "$target_filter" ]]; then
                    benchmark_target "$target"
                fi
            done
            ;;
            
        "all")
            print_status "Running complete cross-compilation workflow..."
            
            # Install targets
            main "install"
            
            # Check compilation
            main "check"
            
            # Build all targets
            main "build"
            
            # Test runnable targets
            main "test"
            
            print_success "Complete cross-compilation workflow finished"
            ;;
            
        "help"|"-h"|"--help")
            echo "Usage: $0 [command] [target]"
            echo ""
            echo "Commands:"
            echo "  install    Install cross-compilation targets"
            echo "  build      Build for all or specific target"
            echo "  test       Test all or specific target"
            echo "  check      Check compilation for all targets"
            echo "  bench      Run benchmarks for all or specific target"
            echo "  all        Run complete workflow (default)"
            echo "  help       Show this help message"
            echo ""
            echo "Targets:"
            echo "  aarch64-apple-darwin      Native Mac ARM"
            echo "  x86_64-apple-darwin       Mac x86_64 (simulation)"
            echo "  x86_64-unknown-linux-gnu  Linux x86_64"
            echo "  aarch64-unknown-linux-gnu Linux ARM64"
            echo ""
            echo "Examples:"
            echo "  $0 build x86_64-apple-darwin"
            echo "  $0 test aarch64-apple-darwin"
            echo "  $0 check"
            ;;
            
        *)
            print_error "Unknown command: $command"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
