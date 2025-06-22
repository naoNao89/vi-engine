# Assembly Linting Documentation

This document describes the assembly linting process for the vi-engine project, which ensures code quality and compatibility across different architectures.

## Overview

The vi-engine project includes assembly optimizations for performance-critical operations. To maintain code quality and ensure compatibility across different platforms, we use comprehensive assembly linting.

## Linting Process

### Automated Linting

Assembly linting is performed automatically through:

1. **CI/CD Pipeline**: The `lint_assembly.sh` script runs on every push and pull request
2. **Local Development**: Developers can run linting manually using the provided scripts
3. **Pre-commit Hooks**: Optional integration for catching issues early

### Supported Architectures

- **ARM64/AArch64**: Native Apple Silicon and ARM64 Linux
- **x86_64**: Intel/AMD 64-bit processors

### Linting Tools

The linting process uses multiple tools for comprehensive coverage:

1. **LLVM Assembler (llvm-mc)**: Primary tool for syntax and semantic validation
2. **Clang**: Cross-platform assembly compilation testing
3. **System Assembler (as)**: Platform-specific validation
4. **NASM**: Alternative x86_64 assembler for compatibility testing

## Running Assembly Linting

### Manual Execution

```bash
# Run with default settings
./scripts/lint_assembly.sh

# Run with verbose output
./scripts/lint_assembly.sh --verbose

# Run silently (errors only)
./scripts/lint_assembly.sh --silent
```

### CI/CD Integration

Assembly linting is automatically triggered in the following scenarios:

- Push to main or develop branches
- Pull requests targeting main or develop
- Manual workflow dispatch

## Common Issues and Solutions

### Symbol Reference Issues

**Problem**: ADRP @PAGE references without proper data section
**Solution**: Ensure data sections are properly declared with `.section .rodata`

**Problem**: Undefined symbol references
**Solution**: Define all referenced symbols or use `.extern` directive

### Alignment and Sections

**Problem**: Missing alignment directives
**Solution**: Add `.align` directives for performance optimization

**Problem**: Missing section declarations
**Solution**: Use appropriate `.text`, `.data`, `.rodata` sections

### Cross-Platform Compatibility

**Problem**: Platform-specific syntax differences
**Solution**: Use LLVM tools for better cross-platform compatibility

## Best Practices

1. **Use Standard Sections**: Always declare appropriate sections (`.text`, `.data`, `.rodata`)
2. **Proper Alignment**: Include alignment directives for performance
3. **Symbol Management**: Export symbols with `.global` directive
4. **Documentation**: Comment assembly code thoroughly
5. **Testing**: Test on multiple architectures when possible

## Troubleshooting

### Installation Issues

If linting tools are missing:

```bash
# macOS
brew install llvm nasm

# Ubuntu/Debian
sudo apt-get install llvm-dev clang nasm
```

### Linting Failures

1. Check the specific error messages in the CI logs
2. Run linting locally with `--verbose` flag
3. Verify assembly syntax against target architecture documentation
4. Ensure all symbols are properly defined or declared

## Integration with Development Workflow

### Local Development

1. Write assembly code following project conventions
2. Run local linting before committing
3. Address any warnings or errors
4. Test on target platforms when possible

### Code Review

1. Assembly changes require thorough review
2. Linting must pass before merge
3. Performance implications should be documented
4. Cross-platform compatibility must be verified

## Configuration

The linting configuration can be customized through:

- Script command-line arguments
- Environment variables
- CI/CD workflow parameters

For detailed configuration options, see the `lint_assembly.sh` script documentation.

## Support

For issues with assembly linting:

1. Check this documentation first
2. Review the script source code for detailed behavior
3. Check CI/CD logs for specific error messages
4. Consult platform-specific assembly documentation

## Future Improvements

Planned enhancements to the assembly linting process:

- Integration with more static analysis tools
- Enhanced cross-compilation testing
- Performance regression detection
- Automated optimization suggestions
