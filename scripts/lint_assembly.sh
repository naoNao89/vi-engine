#!/bin/bash
# Assembly Linting Script for vi-engine project
# Comprehensive linting for ARM64 and x86_64 assembly files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ASM_DIR="src/asm"
TEMP_DIR="/tmp/vi_rust_asm_lint"
VERBOSE=false
SILENT=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -s|--silent)
            SILENT=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [-v|--verbose] [-s|--silent] [-h|--help]"
            echo "  -v, --verbose    Enable verbose output"
            echo "  -s, --silent     Only show errors, suppress warnings"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Create temp directory
mkdir -p "$TEMP_DIR"

echo -e "${BLUE}=== Assembly Linting for vi-engine ===${NC}"
echo "Linting directory: $ASM_DIR"
echo

# Debug: Check available tools on Ubuntu
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Debug: Checking available LLVM tools on Ubuntu..."
    echo "OSTYPE: $OSTYPE"
    echo "Available llvm commands:"
    ls /usr/bin/llvm* 2>/dev/null || echo "No llvm commands found in /usr/bin/"
    echo "Available clang commands:"
    ls /usr/bin/clang* 2>/dev/null || echo "No clang commands found in /usr/bin/"
    echo "PATH: $PATH"
    echo "which llvm-mc: $(which llvm-mc 2>/dev/null || echo 'not found')"
    echo "which clang: $(which clang 2>/dev/null || echo 'not found')"
    echo
fi

# Function to log messages
log() {
    if [[ "$VERBOSE" == true ]]; then
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check for ARM64 symbol reference issues
check_arm64_symbols() {
    local file="$1"
    local issues=0

    # Check for problematic symbol reference patterns (only real issues)
    if grep -q "adrp.*@PAGE" "$file" && ! grep -q "\.section.*rodata\|\.section.*DATA" "$file"; then
        if [[ "$SILENT" != true ]]; then
            echo -e "${YELLOW}⚠${NC} Warning: ADRP @PAGE references without proper data section"
        fi
        ((issues++))
    fi

    # Check for missing data section definitions (exclude register names and false positives)
    if grep -q "adrp.*," "$file"; then
        local symbols=$(grep -o "adrp [^,@]*" "$file" | sed 's/adrp[[:space:]]*\([^@,[:space:]]*\).*/\1/' | grep -v "^x[0-9]" | sort -u)
        for symbol in $symbols; do
            # Skip register names, empty symbols, and common ARM64 patterns
            if [[ "$symbol" =~ ^x[0-9]+$ ]] || [[ "$symbol" =~ ^w[0-9]+$ ]] || [[ -z "$symbol" ]] || [[ "$symbol" =~ ^[[:space:]]*$ ]]; then
                continue
            fi
            # Only warn if symbol is not defined and looks like a real symbol name
            if [[ "$symbol" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] && ! grep -q "^${symbol}:" "$file"; then
                if [[ "$SILENT" != true ]]; then
                    echo -e "${YELLOW}⚠${NC} Warning: Symbol '$symbol' referenced but not defined"
                fi
                ((issues++))
            fi
        done
    fi

    return $issues
}

# Function to lint ARM64 assembly
lint_arm64() {
    local file="$1"
    local basename=$(basename "$file" .s)
    local temp_obj="$TEMP_DIR/${basename}.o"
    local lint_passed=true

    echo -e "${YELLOW}Linting ARM64:${NC} $file"

    # Check for symbol issues first
    check_arm64_symbols "$file"
    local symbol_issues=$?

    # Method 1: Use LLVM assembler (most reliable for ARM64)
    if command_exists llvm-mc; then
        log "Using LLVM assembler (llvm-mc)"
        if llvm-mc -arch=aarch64 -filetype=obj "$file" -o "$temp_obj" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} LLVM assembler: PASS"
        else
            echo -e "${RED}✗${NC} LLVM assembler: FAIL"
            lint_passed=false
        fi
    fi

    # Method 2: Use clang if available
    if command_exists clang; then
        log "Using clang assembler"
        if clang -target aarch64-apple-macos -c "$file" -o "$temp_obj" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} Clang assembler: PASS"
        else
            echo -e "${RED}✗${NC} Clang assembler: FAIL"
            lint_passed=false
        fi
    fi

    # Method 3: Use system assembler (may have compatibility issues)
    if command_exists as; then
        log "Using system assembler (as)"
        if as -arch arm64 -W -o "$temp_obj" "$file" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} System assembler: PASS"
        else
            echo -e "${YELLOW}⚠${NC} System assembler: FAIL (may be due to syntax differences)"
        fi
    fi

    # Report symbol issues
    if [[ $symbol_issues -gt 0 ]]; then
        echo -e "${YELLOW}⚠${NC} Found $symbol_issues symbol reference issues"
    fi

    if [[ "$lint_passed" == true ]]; then
        return 0
    else
        return 1
    fi
}

# Function to lint x86_64 assembly
lint_x86_64() {
    local file="$1"
    local basename=$(basename "$file" .s)
    local temp_obj="$TEMP_DIR/${basename}.o"

    echo -e "${YELLOW}Linting x86_64:${NC} $file"

    # Method 1: Use LLVM assembler (preferred for x86_64)
    # Try different llvm-mc variants (Ubuntu often has versioned names)
    local llvm_mc_cmd=""
    if command_exists llvm-mc; then
        llvm_mc_cmd="llvm-mc"
    elif command_exists llvm-mc-18; then
        llvm_mc_cmd="llvm-mc-18"
    elif command_exists llvm-mc-17; then
        llvm_mc_cmd="llvm-mc-17"
    elif command_exists llvm-mc-16; then
        llvm_mc_cmd="llvm-mc-16"
    elif command_exists llvm-mc-15; then
        llvm_mc_cmd="llvm-mc-15"
    fi

    if [[ -n "$llvm_mc_cmd" ]]; then
        log "Using LLVM assembler ($llvm_mc_cmd)"
        # Preprocess the assembly file first to handle conditional compilation
        local preprocessed_file="$TEMP_DIR/${basename}_preprocessed.s"
        local platform_defines=""
        if [[ "$OSTYPE" == "darwin"* ]]; then
            platform_defines="-D__APPLE__"
        else
            platform_defines="-U__APPLE__ -D__linux__"
        fi

        if clang -E $platform_defines -x assembler-with-cpp "$file" -o "$preprocessed_file" 2>/dev/null; then
            if $llvm_mc_cmd -arch=x86-64 -filetype=obj "$preprocessed_file" -o "$temp_obj" 2>&1; then
                echo -e "${GREEN}✓${NC} LLVM assembler: PASS"
            else
                echo -e "${RED}✗${NC} LLVM assembler: FAIL"
                return 1
            fi
        else
            echo -e "${RED}✗${NC} LLVM assembler: FAIL (preprocessing)"
            return 1
        fi
    else
        log "LLVM assembler (llvm-mc) not found, skipping LLVM test"
    fi

    # Method 2: Use clang if available
    if command_exists clang; then
        log "Using clang assembler"
        if clang -target x86_64-apple-macos -x assembler-with-cpp -c "$file" -o "$temp_obj" 2>&1; then
            echo -e "${GREEN}✓${NC} Clang assembler: PASS"
        else
            echo -e "${RED}✗${NC} Clang assembler: FAIL"
            return 1
        fi
    fi

    # Method 3: Use nasm if available (alternative x86_64 assembler)
    if command_exists nasm; then
        log "Using NASM assembler"
        if nasm -f macho64 "$file" -o "$temp_obj" 2>&1; then
            echo -e "${GREEN}✓${NC} NASM assembler: PASS"
        else
            echo -e "${RED}✗${NC} NASM assembler: FAIL"
            return 1
        fi
    fi

    return 0
}

# Function to perform static analysis
static_analysis() {
    local file="$1"

    if [[ "$SILENT" != true ]]; then
        echo -e "${YELLOW}Static Analysis:${NC} $file"
    fi

    # Check for common issues (only real problems)
    local issues=0

    # Check for proper function alignment (only warn if no alignment at all)
    if ! grep -q "\.align\|\.p2align" "$file"; then
        if [[ "$SILENT" != true ]]; then
            echo -e "${YELLOW}⚠${NC} Warning: No alignment directives found"
        fi
        ((issues++))
    fi

    # Check for proper section declarations (be more flexible)
    if ! grep -q "\.text\|\.data\|\.rodata\|__DATA\|__TEXT" "$file"; then
        if [[ "$SILENT" != true ]]; then
            echo -e "${YELLOW}⚠${NC} Warning: No section declarations found"
        fi
        ((issues++))
    fi

    # Check for global symbol exports (only warn if no exports at all)
    if ! grep -q "\.global" "$file"; then
        if [[ "$SILENT" != true ]]; then
            echo -e "${YELLOW}⚠${NC} Warning: No global symbols exported"
        fi
        ((issues++))
    fi

    # Check for stack frame setup in functions (more specific and accurate check)
    local has_functions=$(grep -c "^[a-zA-Z_][a-zA-Z0-9_]*:" "$file")
    local has_stack_setup=$(grep -c "stp.*sp\|push.*bp\|mov.*bp.*sp\|push rbp\|mov rbp" "$file")

    # Only warn if there are many functions but no stack setup at all
    if [[ $has_functions -gt 3 ]] && [[ $has_stack_setup -eq 0 ]]; then
        if [[ "$SILENT" != true ]]; then
            echo -e "${YELLOW}⚠${NC} Warning: Multiple functions may be missing proper stack frame setup"
        fi
        ((issues++))
    fi

    if [[ "$SILENT" != true ]]; then
        if [[ $issues -eq 0 ]]; then
            echo -e "${GREEN}✓${NC} Static analysis: PASS"
        else
            echo -e "${YELLOW}⚠${NC} Static analysis: $issues warnings"
        fi
    fi

    return 0
}

# Main linting logic
main() {
    local exit_code=0
    local total_files=0
    local passed_files=0

    # Debug: Add verbose error handling for Ubuntu
    if [[ "$OSTYPE" != "darwin"* ]]; then
        echo "Debug: Starting main function on Ubuntu..."
        echo "Debug: ASM_DIR = $ASM_DIR"
        echo "Debug: Checking if directory exists..."
        # Don't use set -e as it exits immediately on any error
        # set -x  # Print commands as they execute (for debugging)
    fi

    # Check if assembly directory exists
    if [[ ! -d "$ASM_DIR" ]]; then
        echo -e "${RED}Error:${NC} Assembly directory '$ASM_DIR' not found"
        if [[ "$OSTYPE" != "darwin"* ]]; then
            echo "Debug: Directory check failed, exiting..."
        fi
        exit 1
    fi

    if [[ "$OSTYPE" != "darwin"* ]]; then
        echo "Debug: Directory exists, continuing..."
        echo "Debug: Starting file processing loop..."
    fi

    # Lint each assembly file
    for file in "$ASM_DIR"/*.s; do
        if [[ "$OSTYPE" != "darwin"* ]]; then
            echo "Debug: Found file: $file"
        fi
        if [[ ! -f "$file" ]]; then
            if [[ "$OSTYPE" != "darwin"* ]]; then
                echo "Debug: File $file is not a regular file, skipping..."
            fi
            continue
        fi
        
        ((total_files++))
        echo

        # Debug: Show which file is being processed
        if [[ "$OSTYPE" != "darwin"* ]]; then
            echo "Debug: Processing file: $file"
        fi

        # Determine architecture and lint accordingly
        if [[ "$file" == *"aarch64"* ]] || [[ "$file" == *"arm64"* ]]; then
            if [[ "$OSTYPE" != "darwin"* ]]; then
                echo "Debug: Detected ARM64 file, calling lint_arm64..."
            fi
            if lint_arm64 "$file"; then
                ((passed_files++))
                if [[ "$OSTYPE" != "darwin"* ]]; then
                    echo "Debug: ARM64 linting passed"
                fi
            else
                exit_code=1
                if [[ "$OSTYPE" != "darwin"* ]]; then
                    echo "Debug: ARM64 linting failed"
                fi
            fi
        elif [[ "$file" == *"x86_64"* ]]; then
            if lint_x86_64 "$file"; then
                ((passed_files++))
            else
                exit_code=1
            fi
        else
            echo -e "${YELLOW}⚠${NC} Unknown architecture for file: $file"
        fi
        
        # Perform static analysis
        static_analysis "$file"
    done
    
    # Summary
    echo
    echo -e "${BLUE}=== Linting Summary ===${NC}"
    echo "Total files: $total_files"
    echo "Passed: $passed_files"
    echo "Failed: $((total_files - passed_files))"

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}✓ All assembly files passed linting${NC}"
    else
        echo -e "${RED}✗ Some assembly files failed linting${NC}"
        echo
        echo -e "${BLUE}=== Recommendations ===${NC}"
        echo "1. For ARM64 symbol reference errors:"
        echo "   - Ensure data sections are properly declared with .section .rodata"
        echo "   - Use @PAGE and @PAGEOFF syntax consistently"
        echo "   - Define all referenced symbols in the same file or use .extern"
        echo
        echo "2. For x86_64 assembler compatibility:"
        echo "   - Use LLVM tools (brew install llvm) for better compatibility"
        echo "   - Consider using Intel syntax (.intel_syntax noprefix)"
        echo
        echo "3. For missing alignment/sections:"
        echo "   - Add .align directives for performance"
        echo "   - Use .text, .data, .rodata sections appropriately"
        echo "   - Export symbols with .global directive"
    fi
    
    # Cleanup
    rm -rf "$TEMP_DIR"
    
    exit $exit_code
}

# Run main function
main "$@"
