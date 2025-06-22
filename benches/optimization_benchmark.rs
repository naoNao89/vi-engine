use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use vi::util::clean_char;

/// Optimized clean_char function using Unicode code point ranges
#[inline]
pub const fn clean_char_optimized(ch: char) -> char {
    let code = ch as u32;

    // Fast path for ASCII characters (most common case)
    if code < 128 {
        return ch;
    }

    // Vietnamese character ranges optimization
    match code {
        // Vietnamese lowercase vowels with diacritics
        0x00E0..=0x00E3 => 'a', // à, á, â, ã
        0x00E8..=0x00EB => 'e', // è, é, ê, ë
        0x00EC..=0x00EF => 'i', // ì, í, î, ï
        0x00F2..=0x00F5 => 'o', // ò, ó, ô, õ
        0x00F9..=0x00FC => 'u', // ù, ú, û, ü
        0x00FD | 0x00FF => 'y', // ý, ÿ

        // Vietnamese uppercase vowels with diacritics
        0x00C0..=0x00C3 => 'A', // À, Á, Â, Ã
        0x00C8..=0x00CB => 'E', // È, É, Ê, Ë
        0x00CC..=0x00CF => 'I', // Ì, Í, Î, Ï
        0x00D2..=0x00D5 => 'O', // Ò, Ó, Ô, Õ
        0x00D9..=0x00DC => 'U', // Ù, Ú, Û, Ü
        0x00DD => 'Y',          // Ý

        // Extended Vietnamese characters
        0x0103 | 0x0102 => {
            if code == 0x0103 {
                'a'
            } else {
                'A'
            }
        } // ă, Ă
        0x0111 | 0x0110 => {
            if code == 0x0111 {
                'd'
            } else {
                'D'
            }
        } // đ, Đ
        0x01A1 | 0x01A0 => {
            if code == 0x01A1 {
                'o'
            } else {
                'O'
            }
        } // ơ, Ơ
        0x01B0 | 0x01AF => {
            if code == 0x01B0 {
                'u'
            } else {
                'U'
            }
        } // ư, Ư

        _ => ch,
    }
}

fn benchmark_clean_char_comparison(c: &mut Criterion) {
    let vietnamese_chars = [
        'á', 'à', 'ả', 'ã', 'ạ', 'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ', 'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'é',
        'è', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ', 'í', 'ì', 'ỉ', 'ĩ', 'ị', 'ó', 'ò', 'ỏ',
        'õ', 'ọ', 'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ú', 'ù', 'ủ', 'ũ',
        'ụ', 'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ', 'đ',
    ];

    c.bench_function("clean_char_original_vietnamese", |b| {
        b.iter(|| {
            for &ch in vietnamese_chars.iter() {
                black_box(clean_char(black_box(ch)));
            }
        })
    });

    c.bench_function("clean_char_optimized_vietnamese", |b| {
        b.iter(|| {
            for &ch in vietnamese_chars.iter() {
                black_box(clean_char_optimized(black_box(ch)));
            }
        })
    });
}

criterion_group!(benches, benchmark_clean_char_comparison);
criterion_main!(benches);
