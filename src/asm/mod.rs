//! Assembly Module
//!
//! This module provides the interface to hand-optimized assembly kernels
//! for Vietnamese character processing with comprehensive safety integration.

pub mod direct_asm;

// Re-export key types and functions
pub use direct_asm::{
    get_assembly_info, get_assembly_interface, is_assembly_available, AssemblyInterface,
    AssemblyPlatform,
};
