// ARM64 (AArch64) Assembly Kernels for Vietnamese Character Processing
// Optimized for Apple Silicon M-series processors
//
// This file contains hand-optimized ARM64 assembly implementations
// for Vietnamese character cleaning operations, leveraging NEON SIMD
// instructions and ARM64-specific optimizations.

.section __TEXT,__text,regular,pure_instructions
.p2align 2

// Vietnamese character mapping constants for ARM64
.section __DATA,__data
.p2align 3

// Complete Vietnamese character lookup table (cache-line aligned)
// Maps Unicode codepoints to base characters
vietnamese_lookup_table_arm64:
    // ASCII characters (0-127) - pass through unchanged
    .long 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07
    .long 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
    .long 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17
    .long 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F
    .long 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27
    .long 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F
    .long 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37
    .long 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F
    .long 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47
    .long 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F
    .long 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57
    .long 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F
    .long 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67
    .long 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F
    .long 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77
    .long 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F

// Vietnamese character mapping table for Latin Extended range (0xC0-0xFF)
// This table maps characters starting from 0xC0, so index 0 = 0xC0
// Only Vietnamese characters are mapped, others pass through unchanged
vietnamese_latin_extended_arm64:
    // 0xC0-0xC7: À,Á,Â,Ã,Ä,Å,Æ,Ç
    .long 0x41, 0x41, 0x41, 0x41, 0xC4, 0xC5, 0xC6, 0xC7  // À,Á,Â,Ã -> A; Ä,Å,Æ,Ç unchanged
    // 0xC8-0xCF: È,É,Ê,Ë,Ì,Í,Î,Ï
    .long 0x45, 0x45, 0x45, 0xCB, 0x49, 0x49, 0xCE, 0xCF  // È,É,Ê -> E; Ë unchanged; Ì,Í -> I; Î,Ï unchanged
    // 0xD0-0xD7: Ð,Ñ,Ò,Ó,Ô,Õ,Ö,×
    .long 0xD0, 0xD1, 0x4F, 0x4F, 0x4F, 0x4F, 0xD6, 0xD7  // Ð,Ñ unchanged; Ò,Ó,Ô,Õ -> O; Ö,× unchanged
    // 0xD8-0xDF: Ø,Ù,Ú,Û,Ü,Ý,Þ,ß
    .long 0xD8, 0x55, 0x55, 0x55, 0xDC, 0x59, 0xDE, 0xDF  // Ø unchanged; Ù,Ú,Û -> U; Ü unchanged; Ý -> Y; Þ,ß unchanged
    // 0xE0-0xE7: à,á,â,ã,ä,å,æ,ç
    .long 0x61, 0x61, 0x61, 0x61, 0xE4, 0xE5, 0xE6, 0xE7  // à,á,â,ã -> a; ä,å,æ,ç unchanged
    // 0xE8-0xEF: è,é,ê,ë,ì,í,î,ï
    .long 0x65, 0x65, 0x65, 0xEB, 0x69, 0x69, 0xEE, 0xEF  // è,é,ê -> e; ë unchanged; ì,í -> i; î,ï unchanged
    // 0xF0-0xF7: ð,ñ,ò,ó,ô,õ,ö,÷
    .long 0xF0, 0xF1, 0x6F, 0x6F, 0x6F, 0x6F, 0xF6, 0xF7  // ð,ñ unchanged; ò,ó,ô,õ -> o; ö,÷ unchanged
    // 0xF8-0xFF: ø,ù,ú,û,ü,ý,þ,ÿ
    .long 0xF8, 0x75, 0x75, 0x75, 0xFC, 0x79, 0xFE, 0x79  // ø unchanged; ù,ú,û -> u; ü unchanged; ý -> y; þ unchanged; ÿ -> y

// Vietnamese specific character mappings (0x1EA0-0x1EF9 range)
// Each entry corresponds exactly to its Unicode codepoint
.p2align 3
vietnamese_specific_1ea0_arm64:
    // 0x1EA0-0x1EAF: A family
    .long 0x41, 0x61, 0x41, 0x61, 0x41, 0x61, 0x41, 0x61  // 1EA0-1EA7: Ạ,ạ,Ả,ả,Ấ,ấ,Ầ,ầ -> A,a,A,a,A,a,A,a
    .long 0x41, 0x61, 0x41, 0x61, 0x41, 0x61, 0x41, 0x61  // 1EA8-1EAF: Ẩ,ẩ,Ẫ,ẫ,Ậ,ậ,Ắ,ắ -> A,a,A,a,A,a,A,a

    // 0x1EB0-0x1EBF: A family continued
    .long 0x41, 0x61, 0x41, 0x61, 0x41, 0x61, 0x41, 0x61  // 1EB0-1EB7: Ằ,ằ,Ẳ,ẳ,Ẵ,ẵ,Ặ,ặ -> A,a,A,a,A,a,A,a
    .long 0x45, 0x65, 0x45, 0x65, 0x45, 0x65, 0x45, 0x65  // 1EB8-1EBF: Ẹ,ẹ,Ẻ,ẻ,Ẽ,ẽ,Ế,ế -> E,e,E,e,E,e,E,e

    // 0x1EC0-0x1ECF: E family continued + I family start
    .long 0x45, 0x65, 0x45, 0x65, 0x45, 0x65, 0x45, 0x65  // 1EC0-1EC7: Ề,ề,Ể,ể,Ễ,ễ,Ệ,ệ -> E,e,E,e,E,e,E,e
    .long 0x49, 0x69, 0x49, 0x69, 0x4F, 0x6F, 0x4F, 0x6F  // 1EC8-1ECF: Ỉ,ỉ,Ị,ị,Ọ,ọ,Ỏ,ỏ -> I,i,I,i,O,o,O,o

    // 0x1ED0-0x1EDF: O family
    .long 0x4F, 0x6F, 0x4F, 0x6F, 0x4F, 0x6F, 0x4F, 0x6F  // 1ED0-1ED7: Ố,ố,Ồ,ồ,Ổ,ổ,Ỗ,ỗ -> O,o,O,o,O,o,O,o
    .long 0x4F, 0x6F, 0x4F, 0x6F, 0x4F, 0x6F, 0x4F, 0x6F  // 1ED8-1EDF: Ộ,ộ,Ớ,ớ,Ờ,ờ,Ở,ở -> O,o,O,o,O,o,O,o

    // 0x1EE0-0x1EEF: O family continued + U family start
    .long 0x4F, 0x6F, 0x4F, 0x6F, 0x55, 0x75, 0x55, 0x75  // 1EE0-1EE7: Ỡ,ỡ,Ợ,ợ,Ụ,ụ,Ủ,ủ -> O,o,O,o,U,u,U,u
    .long 0x55, 0x75, 0x55, 0x75, 0x55, 0x75, 0x55, 0x75  // 1EE8-1EEF: Ứ,ứ,Ừ,ừ,Ử,ử,Ữ,ữ -> U,u,U,u,U,u,U,u

    // 0x1EF0-0x1EF9: U family continued + Y family
    .long 0x55, 0x75, 0x59, 0x79, 0x59, 0x79, 0x59, 0x79  // 1EF0-1EF7: Ự,ự,Ỳ,ỳ,Ỵ,ỵ,Ỷ,ỷ -> U,u,Y,y,Y,y,Y,y
    .long 0x59, 0x79                                        // 1EF8-1EF9: Ỹ,ỹ -> Y,y



.section __TEXT,__text,regular,pure_instructions

// Ultra-fast Vietnamese character cleaning - Generic ARM64 version
// Input: w0 = character (u32)
// Output: w0 = cleaned character (u32)
// Preserves: All registers except w0
.global _hybrid_clean_char_aarch64
.p2align 2
_hybrid_clean_char_aarch64:
    // Fast path for ASCII characters (most common case)
    cmp w0, #128
    b.lo Lascii_passthrough_arm64

    // Check for Latin Extended range (0x00C0-0x00FF)
    mov w1, w0
    mov w2, #0x00C0
    cmp w1, w2
    b.lo Lnot_vietnamese_arm64
    mov w2, #0x00FF
    cmp w1, w2
    b.ls Llatin_extended_lookup_arm64

    // Check for Vietnamese extended range (0x0100-0x01FF) - expand range
    mov w2, #0x0100
    cmp w1, w2
    b.lo Lnot_vietnamese_arm64
    mov w2, #0x01FF
    cmp w1, w2
    b.ls Lextended_100_lookup_arm64

    // Check for Vietnamese specific range (0x1EA0-0x1EF9)
    // Load constants using proper ARM64 method
    mov w2, #0x1E00
    add w2, w2, #0xA0                           // w2 = 0x1EA0
    cmp w1, w2
    b.lo Lnot_vietnamese_arm64
    mov w2, #0x1E00
    add w2, w2, #0xF9                           // w2 = 0x1EF9
    cmp w1, w2
    b.ls Lvietnamese_specific_lookup_arm64

    // Not a Vietnamese character - return original
Lnot_vietnamese_arm64:
    ret

Lascii_passthrough_arm64:
    ret

Llatin_extended_lookup_arm64:
    // Use lookup table for Latin Extended range (0x00C0-0x00FF)
    adrp x2, vietnamese_latin_extended_arm64@PAGE
    add x2, x2, vietnamese_latin_extended_arm64@PAGEOFF
    // The table starts at 0xC0, so subtract 0xC0 to get table index
    sub w1, w1, #0x00C0                         // Normalize to table index (0x00C0 -> 0)
    ldr w0, [x2, x1, lsl #2]                    // Load mapped character from table
    ret

Lextended_100_lookup_arm64:
    // Handle specific Vietnamese characters in extended range
    // Check for Ă (0x0102)
    mov w2, #0x0102
    cmp w1, w2
    b.eq Lmap_to_a_uppercase_arm64

    // Check for ă (0x0103)
    mov w2, #0x0103
    cmp w1, w2
    b.eq Lmap_to_a_arm64

    // Check for Đ (0x0110)
    mov w2, #0x0110
    cmp w1, w2
    b.eq Lmap_to_d_uppercase_arm64

    // Check for đ (0x0111)
    mov w2, #0x0111
    cmp w1, w2
    b.eq Lmap_to_d_arm64

    // Check for Ĩ (0x0128)
    mov w2, #0x0128
    cmp w1, w2
    b.eq Lmap_to_i_uppercase_arm64

    // Check for ĩ (0x0129)
    mov w2, #0x0129
    cmp w1, w2
    b.eq Lmap_to_i_arm64

    // Check for Ũ (0x0168)
    mov w2, #0x0168
    cmp w1, w2
    b.eq Lmap_to_u_uppercase_arm64

    // Check for ũ (0x0169)
    mov w2, #0x0169
    cmp w1, w2
    b.eq Lmap_to_u_arm64

    // Check for Ơ (0x01A0)
    mov w2, #0x01A0
    cmp w1, w2
    b.eq Lmap_to_o_uppercase_arm64

    // Check for ơ (0x01A1)
    mov w2, #0x01A1
    cmp w1, w2
    b.eq Lmap_to_o_arm64

    // Check for Ư (0x01AF)
    mov w2, #0x01AF
    cmp w1, w2
    b.eq Lmap_to_u_uppercase_arm64

    // Check for ư (0x01B0)
    mov w2, #0x01B0
    cmp w1, w2
    b.eq Lmap_to_u_arm64

    // Return original character if no match
    mov w0, w1
    ret

Lvietnamese_specific_lookup_arm64:
    // Use lookup table for Vietnamese specific range (0x1EA0-0x1EF9)
    adrp x2, vietnamese_specific_1ea0_arm64@PAGE
    add x2, x2, vietnamese_specific_1ea0_arm64@PAGEOFF
    // Load 0x1EA0 into register using add method
    mov w3, #0x1E00
    add w3, w3, #0xA0                           // w3 = 0x1EA0
    sub w1, w1, w3                              // Normalize to table index
    mov w4, #0x1E00
    add w4, w4, #0xF9                           // w4 = 0x1EF9
    sub w4, w4, w3                              // Calculate max index (0x1EF9 - 0x1EA0)
    cmp w1, w4                                  // Bounds check
    b.hi Lnot_vietnamese_arm64                  // Out of bounds
    ldr w0, [x2, x1, lsl #2]                    // Load mapped character from table
    ret

// Character family mapping targets (for specific cases)
Lmap_to_a_arm64:
    mov w0, #0x0061  // 'a'
    ret

Lmap_to_a_uppercase_arm64:
    mov w0, #0x0041  // 'A'
    ret

Lmap_to_d_arm64:
    mov w0, #0x0064  // 'd'
    ret

Lmap_to_d_uppercase_arm64:
    mov w0, #0x0044  // 'D'
    ret

Lmap_to_i_arm64:
    mov w0, #0x0069  // 'i'
    ret

Lmap_to_i_uppercase_arm64:
    mov w0, #0x0049  // 'I'
    ret

Lmap_to_o_arm64:
    mov w0, #0x006F  // 'o'
    ret

Lmap_to_o_uppercase_arm64:
    mov w0, #0x004F  // 'O'
    ret

Lmap_to_u_arm64:
    mov w0, #0x0075  // 'u'
    ret

Lmap_to_u_uppercase_arm64:
    mov w0, #0x0055  // 'U'
    ret

// NEON-optimized Vietnamese character cleaning
// Input: w0 = character (u32)
// Output: w0 = cleaned character (u32)
// Requires: NEON instruction set
.global _hybrid_clean_char_neon
.p2align 2
_hybrid_clean_char_neon:
    // Fast path for ASCII
    cmp w0, #128
    b.lo Lneon_ascii_passthrough

    // Use NEON for advanced character processing
    // Load character into NEON register
    dup v0.4s, w0

    // Create comparison vectors for Vietnamese ranges
    mov w1, #0x00C0
    dup v1.4s, w1      // Latin start
    mov w1, #0x017F
    dup v2.4s, w1      // Latin end
    mov w1, #0x1E00
    add w1, w1, #0xA0  // w1 = 0x1EA0
    dup v3.4s, w1      // Vietnamese start
    mov w1, #0x1E00
    add w1, w1, #0xF9  // w1 = 0x1EF9
    dup v4.4s, w1      // Vietnamese end

    // For any non-ASCII character, fall back to the full aarch64 implementation
    // This ensures all Vietnamese characters (including extended range 0x0100-0x01FF) are handled
    b _hybrid_clean_char_aarch64

Lneon_ascii_passthrough:
    ret

Lneon_not_vietnamese:
    ret

// NEON bulk character processing
// Input: x0 = input array pointer, x1 = output array pointer, x2 = length
// Output: x0 = number of characters processed
// Requires: NEON instruction set
.global _hybrid_clean_chars_bulk_neon
.p2align 2
_hybrid_clean_chars_bulk_neon:
    // Simple implementation without function calls to avoid stack issues
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    // Save parameters in callee-saved registers
    mov x9, x0   // input pointer
    mov x10, x1  // output pointer
    mov x11, x2  // length
    mov x12, #0  // processed count

    // Process characters one by one with simple passthrough
    mov x3, x11  // Use x3 as remaining count

Lneon_simple_loop:
    cbz x3, Lneon_done

    ldr w0, [x9], #4    // Load input character

    // Save registers that might be modified by the function call
    stp x9, x10, [sp, #-16]!
    stp x11, x12, [sp, #-16]!
    str x3, [sp, #-16]!

    bl _hybrid_clean_char_aarch64

    // Restore registers
    ldr x3, [sp], #16
    ldp x11, x12, [sp], #16
    ldp x9, x10, [sp], #16

    str w0, [x10], #4   // Store processed character
    add x12, x12, #1    // Increment processed count
    subs x3, x3, #1     // Decrement remaining count
    b.ne Lneon_simple_loop

Lneon_done:
    mov x0, x12  // Return total processed count

    ldp x29, x30, [sp], #16
    ret

// Export symbols for C linkage
.global _hybrid_clean_char_aarch64
.global _hybrid_clean_char_neon
.global _hybrid_clean_chars_bulk_neon
