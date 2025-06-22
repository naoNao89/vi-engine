# Assembly Safety Model and Isolation

This document details the comprehensive safety model used to isolate assembly code from Rust's memory safety guarantees while maintaining high performance and reliability.

## Safety Architecture Overview

The vi-rust assembly integration implements a multi-layered safety model:

1. **Isolation Boundaries**: Clear separation between Rust and assembly domains
2. **Cooperative Cancellation**: Assembly code can be stopped gracefully
3. **Timeout Protection**: Automatic termination of runaway assembly operations
4. **Panic Integration**: Assembly operations stop when Rust panics
5. **Resource Management**: Automatic cleanup and resource tracking
6. **Signal Handling**: Graceful shutdown on system signals

## Isolation Boundaries

### Memory Isolation

Assembly code operates within strict memory boundaries:

```rust
/// Assembly functions operate only on caller-provided buffers
/// No dynamic allocation or global state modification
pub struct AssemblyMemoryContract {
    /// Input buffer - read-only access
    input: *const u32,
    input_len: usize,
    
    /// Output buffer - write-only access  
    output: *mut u32,
    output_len: usize,
    
    /// Control structure - limited atomic access
    control: *const AssemblyControl,
}

impl AssemblyMemoryContract {
    /// Validate memory contract before assembly call
    pub fn validate(&self) -> Result<(), AssemblyError> {
        // Check pointer validity
        if self.input.is_null() || self.output.is_null() || self.control.is_null() {
            return Err(AssemblyError::InvalidInput);
        }
        
        // Check buffer sizes
        if self.input_len == 0 || self.output_len == 0 {
            return Err(AssemblyError::InvalidInput);
        }
        
        if self.input_len != self.output_len {
            return Err(AssemblyError::InvalidInput);
        }
        
        // Check for reasonable size limits (prevent DoS)
        if self.input_len > 100_000_000 {
            return Err(AssemblyError::InvalidInput);
        }
        
        // Verify buffer alignment for SIMD operations
        if (self.input as usize) % 4 != 0 || (self.output as usize) % 4 != 0 {
            return Err(AssemblyError::InvalidInput);
        }
        
        Ok(())
    }
}
```

### Type Safety Boundaries

All data exchange uses well-defined C-compatible types:

```rust
/// Safe type conversions for assembly interface
pub struct TypeSafetyLayer;

impl TypeSafetyLayer {
    /// Convert Rust char to assembly-compatible u32
    pub fn char_to_u32(ch: char) -> Result<u32, AssemblyError> {
        let code = ch as u32;
        if code > 0x10FFFF {
            return Err(AssemblyError::InvalidInput);
        }
        Ok(code)
    }
    
    /// Convert assembly u32 result back to Rust char
    pub fn u32_to_char(code: u32) -> Result<char, AssemblyError> {
        char::from_u32(code).ok_or_else(|| {
            AssemblyError::ExecutionError(
                format!("Assembly returned invalid Unicode: 0x{:X}", code)
            )
        })
    }
    
    /// Convert string to assembly-compatible u32 array
    pub fn string_to_u32_vec(s: &str) -> Result<Vec<u32>, AssemblyError> {
        s.chars()
            .map(Self::char_to_u32)
            .collect()
    }
    
    /// Convert u32 array back to string
    pub fn u32_vec_to_string(codes: &[u32]) -> Result<String, AssemblyError> {
        codes.iter()
            .map(|&code| Self::u32_to_char(code))
            .collect()
    }
}
```

## Cooperative Cancellation System

### Control Structure Design

The assembly control structure enables safe communication between Rust and assembly:

```rust
/// Cache-line aligned control structure for optimal performance
#[repr(C, align(64))]
pub struct AssemblyControl {
    /// Cooperative cancellation flag - checked by assembly loops
    pub cancel_flag: AtomicBool,
    
    /// Timeout flag - set when operation exceeds time limit
    pub timeout_flag: AtomicBool,
    
    /// Panic flag - set when Rust panic occurs
    pub panic_flag: AtomicBool,
    
    /// Maximum iterations allowed for assembly loops
    pub max_iterations: AtomicUsize,
    
    /// Current iteration count - updated by assembly
    pub current_iteration: AtomicUsize,
    
    /// Heartbeat counter for progress monitoring
    pub heartbeat: AtomicU64,
    
    /// Operation start time (nanoseconds since epoch)
    pub start_time: AtomicU64,
    
    /// Timeout duration in milliseconds
    pub timeout_ms: AtomicU64,
}

impl AssemblyControl {
    /// Reset control structure for new operation
    pub fn reset_for_operation(&self, expected_size: usize) {
        self.cancel_flag.store(false, Ordering::SeqCst);
        self.timeout_flag.store(false, Ordering::SeqCst);
        self.panic_flag.store(false, Ordering::SeqCst);
        self.current_iteration.store(0, Ordering::SeqCst);
        self.max_iterations.store(expected_size * 2, Ordering::SeqCst); // 2x safety margin
        self.heartbeat.store(0, Ordering::SeqCst);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.start_time.store(now, Ordering::SeqCst);
    }
    
    /// Check if assembly should continue processing
    pub fn should_continue(&self) -> bool {
        // Check cancellation flags
        if self.cancel_flag.load(Ordering::Relaxed) ||
           self.timeout_flag.load(Ordering::Relaxed) ||
           self.panic_flag.load(Ordering::Relaxed) {
            return false;
        }
        
        // Check iteration limit
        let current = self.current_iteration.load(Ordering::Relaxed);
        let max = self.max_iterations.load(Ordering::Relaxed);
        if current >= max {
            return false;
        }
        
        true
    }
    
    /// Signal cancellation of all operations
    pub fn cancel_all(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }
    
    /// Check if operation was cancelled
    pub fn was_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }
    
    /// Check if operation timed out
    pub fn timed_out(&self) -> bool {
        self.timeout_flag.load(Ordering::Relaxed)
    }
    
    /// Update heartbeat (called from assembly)
    pub fn update_heartbeat(&self) {
        self.heartbeat.fetch_add(1, Ordering::Relaxed);
    }
}
```

### Assembly Integration Pattern

Assembly code integrates with the control structure:

```assembly
// ARM64 assembly with cooperative cancellation
_apple_hybrid_clean_chars_bulk_neon_optimized:
    // Function prologue
    stp x29, x30, [sp, #-48]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    
    // Save parameters
    mov x19, x0                      // input pointer
    mov x20, x1                      // output pointer
    mov x21, x2                      // length
    mov x22, #0                      // processed count
    
    // Load control structure address
    adrp x9, _GLOBAL_ASSEMBLY_CONTROL@PAGE
    add x9, x9, _GLOBAL_ASSEMBLY_CONTROL@PAGEOFF
    
.processing_loop:
    // Check if we should continue every 1024 iterations
    and x10, x22, #0x3FF             // x22 & 1023
    cbnz x10, .skip_safety_check
    
    // Load cancellation flags
    ldrb w11, [x9, #0]               // cancel_flag
    ldrb w12, [x9, #1]               // timeout_flag  
    ldrb w13, [x9, #2]               // panic_flag
    orr w11, w11, w12
    orr w11, w11, w13
    cbnz w11, .operation_cancelled
    
    // Check iteration limit
    ldr x11, [x9, #16]               // current_iteration
    ldr x12, [x9, #8]                // max_iterations
    cmp x11, x12
    b.ge .iteration_limit_exceeded
    
    // Update heartbeat every 1024 iterations
    ldr x11, [x9, #24]               // heartbeat
    add x11, x11, #1
    str x11, [x9, #24]
    
.skip_safety_check:
    // Process character
    ldr w10, [x19, x22, lsl #2]      // Load input character
    // ... character processing logic ...
    str w10, [x20, x22, lsl #2]      // Store output character
    
    // Update counters
    add x22, x22, #1                 // Increment processed count
    str x22, [x9, #16]               // Update current_iteration
    
    // Continue if more characters to process
    cmp x22, x21
    b.lt .processing_loop
    
    // Normal completion
    mov x0, x22                      // Return processed count
    b .function_exit
    
.operation_cancelled:
    mov x0, x22                      // Return partial count
    b .function_exit
    
.iteration_limit_exceeded:
    mov x0, x22                      // Return partial count
    b .function_exit
    
.function_exit:
    // Function epilogue
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #48
    ret
```

## Timeout Protection

### Watchdog Thread Implementation

A dedicated watchdog thread monitors assembly operations:

```rust
pub struct AssemblyWatchdog {
    control: Arc<AssemblyControl>,
    handle: Option<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl AssemblyWatchdog {
    pub fn new(control: Arc<AssemblyControl>) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let handle = Self::spawn_watchdog_thread(control.clone(), shutdown.clone());
        
        Self {
            control,
            handle: Some(handle),
            shutdown,
        }
    }
    
    fn spawn_watchdog_thread(
        control: Arc<AssemblyControl>,
        shutdown: Arc<AtomicBool>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100)); // Check every 100ms
                
                // Check if operation has timed out
                let start_time = control.start_time.load(Ordering::Relaxed);
                if start_time == 0 {
                    continue; // No operation in progress
                }
                
                let timeout_ms = control.timeout_ms.load(Ordering::Relaxed);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                
                let elapsed_ms = (now - start_time) / 1_000_000;
                
                if elapsed_ms > timeout_ms {
                    // Operation has timed out
                    control.timeout_flag.store(true, Ordering::SeqCst);
                    control.cancel_flag.store(true, Ordering::SeqCst);
                    
                    log::warn!("Assembly operation timed out after {}ms", elapsed_ms);
                }
                
                // Check for stalled operations (no heartbeat updates)
                let last_heartbeat = control.heartbeat.load(Ordering::Relaxed);
                thread::sleep(Duration::from_millis(1000)); // Wait 1 second
                let current_heartbeat = control.heartbeat.load(Ordering::Relaxed);
                
                if start_time != 0 && last_heartbeat == current_heartbeat {
                    // No progress in 1 second - possible stall
                    log::warn!("Assembly operation may be stalled (no heartbeat)");
                    control.cancel_flag.store(true, Ordering::SeqCst);
                }
            }
        })
    }
    
    pub fn shutdown(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AssemblyWatchdog {
    fn drop(&mut self) {
        self.shutdown();
    }
}
```

## Panic Integration

### Panic Hook Setup

The safety system integrates with Rust's panic handling:

```rust
/// Global assembly control instance
pub static GLOBAL_ASSEMBLY_CONTROL: Lazy<Arc<AssemblyControl>> = 
    Lazy::new(|| Arc::new(AssemblyControl::new()));

/// Initialize panic hook to stop assembly operations
pub fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Signal all assembly operations to stop immediately
        GLOBAL_ASSEMBLY_CONTROL.panic_flag.store(true, Ordering::SeqCst);
        GLOBAL_ASSEMBLY_CONTROL.cancel_flag.store(true, Ordering::SeqCst);
        
        log::error!("Rust panic detected, stopping all assembly operations");
        
        // Give assembly brief time to see flags before unwinding
        std::thread::sleep(Duration::from_millis(10));
        
        // Call original panic hook
        original_hook(panic_info);
    }));
}
```

### Stack Unwinding Safety

Assembly functions are designed to be unwind-safe:

```rust
/// Ensure assembly operations are unwind-safe
pub struct UnwindSafeAssemblyCall<F, R> 
where
    F: FnOnce() -> R,
{
    operation: F,
    control: Arc<AssemblyControl>,
}

impl<F, R> UnwindSafeAssemblyCall<F, R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    pub fn new(operation: F, control: Arc<AssemblyControl>) -> Self {
        Self { operation, control }
    }
    
    pub fn execute(self) -> Result<R, AssemblyError> {
        // Set up cancellation on panic
        let control = self.control.clone();
        let panic_guard = PanicGuard::new(control);
        
        // Execute operation with panic protection
        let result = std::panic::catch_unwind(|| {
            (self.operation)()
        });
        
        // Clean up panic guard
        drop(panic_guard);
        
        match result {
            Ok(value) => Ok(value),
            Err(_) => Err(AssemblyError::Panic),
        }
    }
}

struct PanicGuard {
    control: Arc<AssemblyControl>,
}

impl PanicGuard {
    fn new(control: Arc<AssemblyControl>) -> Self {
        Self { control }
    }
}

impl Drop for PanicGuard {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Cancel assembly operations if we're unwinding due to panic
            self.control.panic_flag.store(true, Ordering::SeqCst);
            self.control.cancel_flag.store(true, Ordering::SeqCst);
        }
    }
}
```

This comprehensive safety model ensures that assembly code remains isolated and controllable while maintaining high performance for Vietnamese text processing.
