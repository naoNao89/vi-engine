// Generic ARM64 (AArch64) Assembly Kernels for Vietnamese Character Processing
// Compatible with standard ARM64 processors (non-Apple Silicon)
//
// This file contains portable ARM64 assembly implementations that work
// across different ARM64 platforms including Linux, Android, and embedded systems.

.text
.align 4

// Vietnamese character mapping constants for generic ARM64
.section .rodata,"a"
.align 3

// Complete Vietnamese character lookup table (cache-line aligned)
// Maps Unicode codepoints to base characters
vietnamese_lookup_table_generic:
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
vietnamese_latin_extended_generic:
    // 0xC0-0xC7: À,Á,Â,Ã,Ä,Å,Æ,Ç
    .long 0x41, 0x41, 0x41, 0x41, 0xC4, 0xC5, 0xC6, 0xC7  // À,Á,Â,Ã -> A; others unchanged
    // 0xC8-0xCF: È,É,Ê,Ë,Ì,Í,Î,Ï
    .long 0x45, 0x45, 0x45, 0xCB, 0x49, 0x49, 0xCE, 0xCF  // È,É,Ê -> E; Ì,Í -> I; others unchanged
    // 0xD0-0xD7: Ð,Ñ,Ò,Ó,Ô,Õ,Ö,×
    .long 0xD0, 0xD1, 0x4F, 0x4F, 0x4F, 0x4F, 0xD6, 0xD7  // Ò,Ó,Ô,Õ -> O; others unchanged
    // 0xD8-0xDF: Ø,Ù,Ú,Û,Ü,Ý,Þ,ß
    .long 0xD8, 0x55, 0x55, 0x55, 0xDC, 0x59, 0xDE, 0xDF  // Ù,Ú,Û -> U; Ý -> Y; others unchanged
    // 0xE0-0xE7: à,á,â,ã,ä,å,æ,ç
    .long 0x61, 0x61, 0x61, 0x61, 0xE4, 0xE5, 0xE6, 0xE7  // à,á,â,ã -> a; others unchanged
    // 0xE8-0xEF: è,é,ê,ë,ì,í,î,ï
    .long 0x65, 0x65, 0x65, 0xEB, 0x69, 0x69, 0xEE, 0xEF  // è,é,ê -> e; ì,í -> i; others unchanged
    // 0xF0-0xF7: ð,ñ,ò,ó,ô,õ,ö,÷
    .long 0xF0, 0xF1, 0x6F, 0x6F, 0x6F, 0x6F, 0xF6, 0xF7  // ò,ó,ô,õ -> o; others unchanged
    // 0xF8-0xFF: ø,ù,ú,û,ü,ý,þ,ÿ
    .long 0xF8, 0x75, 0x75, 0x75, 0xFC, 0x79, 0xFE, 0x79  // ù,ú,û -> u; ý,ÿ -> y; others unchanged

// Vietnamese specific character mappings (0x1EA0-0x1EF9 range)
.align 3
vietnamese_specific_1ea0_generic:
    // 0x1EA0-0x1EAF: A family
    .long 0x41, 0x61, 0x41, 0x61, 0x41, 0x61, 0x41, 0x61  // 1EA0-1EA7: Ạ,ạ,Ả,ả,Ấ,ấ,Ầ,ầ -> A,a,A,a,A,a,A,a
    .long 0x41, 0x61, 0x41, 0x61, 0x41, 0x61, 0x41, 0x61  // 1EA8-1EAF: Ẩ,ẩ,Ẫ,ẫ,Ậ,ậ,Ắ,ắ -> A,a,A,a,A,a,A,a

    // 0x1EB0-0x1EBF: A family continued + E family start
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

.text

// Generic ARM64 Vietnamese character cleaning
// Input: w0 = character (u32)
// Output: w0 = cleaned character (u32)
// Compatible with all ARM64 processors
.global generic_clean_char_aarch64
.align 2
generic_clean_char_aarch64:
    // Fast path for ASCII characters (most common case)
    cmp w0, #128
    b.lo .Lascii_passthrough_generic

    // Check for Latin Extended range (0x00C0-0x00FF)
    mov w1, w0
    mov w2, #0x00C0
    cmp w1, w2
    b.lo .Lnot_vietnamese_generic
    mov w2, #0x00FF
    cmp w1, w2
    b.ls .Llatin_extended_lookup_generic

    // Check for Vietnamese extended range (0x0100-0x01FF)
    mov w2, #0x0100
    cmp w1, w2
    b.lo .Lnot_vietnamese_generic
    mov w2, #0x01FF
    cmp w1, w2
    b.ls .Lextended_100_lookup_generic

    // Check for Vietnamese specific range (0x1EA0-0x1EF9)
    mov w2, #0x1E00
    add w2, w2, #0xA0                           // w2 = 0x1EA0
    cmp w1, w2
    b.lo .Lnot_vietnamese_generic
    mov w2, #0x1E00
    add w2, w2, #0xF9                           // w2 = 0x1EF9
    cmp w1, w2
    b.ls .Lvietnamese_specific_lookup_generic

    // Not a Vietnamese character - return original
.Lnot_vietnamese_generic:
    ret

.Lascii_passthrough_generic:
    ret

.Llatin_extended_lookup_generic:
    // Use lookup table for Latin Extended range (0x00C0-0x00FF)
    adrp x2, vietnamese_latin_extended_generic@PAGE
    add x2, x2, vietnamese_latin_extended_generic@PAGEOFF
    sub w1, w1, #0x00C0                         // Normalize to table index
    ldr w0, [x2, x1, lsl #2]                    // Load mapped character from table
    ret

.Lextended_100_lookup_generic:
    // Handle specific Vietnamese characters in extended range
    // Check for common Vietnamese characters
    cmp w1, #0x0103  // ă
    b.eq .Lmap_to_a_generic
    cmp w1, #0x0102  // Ă
    b.eq .Lmap_to_A_generic
    cmp w1, #0x0111  // đ
    b.eq .Lmap_to_d_generic
    cmp w1, #0x0110  // Đ
    b.eq .Lmap_to_D_generic
    cmp w1, #0x01A1  // ơ
    b.eq .Lmap_to_o_generic
    cmp w1, #0x01A0  // Ơ
    b.eq .Lmap_to_O_generic
    cmp w1, #0x01B0  // ư
    b.eq .Lmap_to_u_generic
    cmp w1, #0x01AF  // Ư
    b.eq .Lmap_to_U_generic

    // Return original character if no match
    mov w0, w1
    ret

.Lvietnamese_specific_lookup_generic:
    // Use lookup table for Vietnamese specific range (0x1EA0-0x1EF9)
    adrp x2, vietnamese_specific_1ea0_generic@PAGE
    add x2, x2, vietnamese_specific_1ea0_generic@PAGEOFF
    mov w3, #0x1E00
    add w3, w3, #0xA0                           // w3 = 0x1EA0
    sub w1, w1, w3                              // Normalize to table index
    ldr w0, [x2, x1, lsl #2]                    // Load mapped character from table
    ret

// Character mapping targets
.Lmap_to_a_generic:
    mov w0, #0x61  // 'a'
    ret

.Lmap_to_A_generic:
    mov w0, #0x41  // 'A'
    ret

.Lmap_to_d_generic:
    mov w0, #0x64  // 'd'
    ret

.Lmap_to_D_generic:
    mov w0, #0x44  // 'D'
    ret

.Lmap_to_o_generic:
    mov w0, #0x6F  // 'o'
    ret

.Lmap_to_O_generic:
    mov w0, #0x4F  // 'O'
    ret

.Lmap_to_u_generic:
    mov w0, #0x75  // 'u'
    ret

.Lmap_to_U_generic:
    mov w0, #0x55  // 'U'
    ret

// Bulk character processing for generic ARM64
// Input: x0 = input array pointer, x1 = output array pointer, x2 = length
// Output: x0 = number of characters processed
.global generic_clean_chars_bulk_aarch64
.align 2
generic_clean_chars_bulk_aarch64:
    stp x29, x30, [sp, #-32]!
    mov x29, sp
    stp x19, x20, [sp, #16]

    // Save parameters
    mov x19, x0  // input pointer
    mov x20, x1  // output pointer
    mov x3, #0   // processed count

    // Process characters one by one
.Lgeneric_bulk_loop:
    cbz x2, .Lgeneric_bulk_done

    ldr w0, [x19], #4           // Load character
    bl generic_clean_char_aarch64  // Process character
    str w0, [x20], #4           // Store result
    add x3, x3, #1              // Increment processed count
    subs x2, x2, #1             // Decrement remaining count
    b.ne .Lgeneric_bulk_loop

.Lgeneric_bulk_done:
    mov x0, x3  // Return total processed count

    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #32
    ret

// Export symbols
.global generic_clean_char_aarch64
.global generic_clean_chars_bulk_aarch64

// Note: Data sections are defined above in the file
